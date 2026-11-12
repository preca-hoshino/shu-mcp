//! 上海大学空教室查询与排课日程业务引擎。

use super::classroom_models::{
    BuildingDaySchedule, ClassroomApiResponse, FloorInfo, RoomInfo, STANDARD_SECTIONS,
};
use super::classroom_resolver::BuildingDef;
use std::collections::HashMap;

/// 空教室直连 API 端点。
const CLASSROOM_ENDPOINT: &str = "https://classroom.cc.shu.edu.cn/build/findBuildRoomType";

/// 节次查询范围。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SectionQueryRange {
    /// 起始节次（1-12）。
    pub start: u8,
    /// 结束节次（1-12）。
    pub end: u8,
}

impl SectionQueryRange {
    /// 全天（第 1 节至第 12 节）。
    pub const ALL_DAY: Self = Self { start: 1, end: 12 };

    /// 对应的起止时间描述（如 "08:00-21:40" 或 "10:00-11:40"）。
    #[must_use]
    pub fn time_range_str(&self) -> String {
        let start_time = STANDARD_SECTIONS
            .iter()
            .find(|s| s.index == self.start)
            .map_or("08:00", |s| s.start_time);
        let end_time = STANDARD_SECTIONS
            .iter()
            .find(|s| s.index == self.end)
            .map_or("21:40", |s| s.end_time);
        format!("{start_time}-{end_time}")
    }

    /// 节次范围的可读标题。
    #[must_use]
    pub fn label(&self) -> String {
        if self.start == 1 && self.end == 12 {
            "全天 (第 1-12 节 连续空闲)".to_string()
        } else if self.start == self.end {
            format!("第 {} 节 ({})", self.start, self.time_range_str())
        } else {
            format!("第 {}-{} 节 ({})", self.start, self.end, self.time_range_str())
        }
    }
}

/// 解析用户或模型输入的节次参数。
///
/// 若未传入或输入为空，默认返回 `ALL_DAY`。
/// 支持 "3-4", "5-8", "1..2", "3", "3~4" 等格式。
#[must_use]
pub fn parse_section_range(input: Option<&str>) -> SectionQueryRange {
    let Some(raw) = input else {
        return SectionQueryRange::ALL_DAY;
    };
    let trimmed = raw.trim();
    if trimmed.is_empty() || trimmed.contains("全天") || trimmed.contains("整天") {
        return SectionQueryRange::ALL_DAY;
    }

    // 提取数字并支持常见分隔符
    let clean = trimmed.replace('节', "");
    let parts: Vec<&str> = clean
        .split(['-', '~', '/', ',', '，', '.', '到', '至'])
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect();

    match parts.as_slice() {
        [single] => {
            if let Ok(num) = single.parse::<u8>() {
                let clamped = num.clamp(1, 12);
                SectionQueryRange { start: clamped, end: clamped }
            } else {
                SectionQueryRange::ALL_DAY
            }
        }
        [s1, s2, ..] => {
            let start = s1.parse::<u8>().unwrap_or(1).clamp(1, 12);
            let end = s2.parse::<u8>().unwrap_or(12).clamp(1, 12);
            if start <= end {
                SectionQueryRange { start, end }
            } else {
                SectionQueryRange { start: end, end: start }
            }
        }
        _ => SectionQueryRange::ALL_DAY,
    }
}

/// 从汉字楼层（如 "一层"、"二层"、"负一层"）或英文提取楼层排序序号。
fn extract_floor_number(text: &str) -> i32 {
    if text.contains('一') || text.contains("1F") || text.contains('1') {
        1
    } else if text.contains('二') || text.contains("2F") || text.contains('2') {
        2
    } else if text.contains('三') || text.contains("3F") || text.contains('3') {
        3
    } else if text.contains('四') || text.contains("4F") || text.contains('4') {
        4
    } else if text.contains('五') || text.contains("5F") || text.contains('5') {
        5
    } else if text.contains('六') || text.contains("6F") || text.contains('6') {
        6
    } else if text.contains('七') || text.contains("7F") || text.contains('7') {
        7
    } else if text.contains('八') || text.contains("8F") || text.contains('8') {
        8
    } else if text.contains('九') || text.contains("9F") || text.contains('9') {
        9
    } else if text.contains('十') || text.contains("10") {
        10
    } else {
        100
    }
}

/// 请求并解析单栋教学楼在指定日期的完整排课日程。
///
/// # Errors
/// 网络请求失败或官方接口返回非成功状态时返回错误信息字符串。
pub async fn fetch_building_schedule(
    client: &reqwest::Client,
    building: &'static BuildingDef,
    date: &str,
) -> Result<BuildingDaySchedule, String> {
    let mut params = HashMap::new();
    let build_id_str = building.id.to_string();
    params.insert("buildId", build_id_str.as_str());
    params.insert("courseDate", date);

    let response = client
        .post(CLASSROOM_ENDPOINT)
        .header(reqwest::header::USER_AGENT, "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
        .header("X-Requested-With", "XMLHttpRequest")
        .form(&params)
        .send()
        .await
        .map_err(|err| format!("网络请求失败: {err}"))?;

    let status = response.status();
    if !status.is_success() {
        return Err(format!("系统响应异常 (HTTP {status})"));
    }

    let body = response
        .text()
        .await
        .map_err(|err| format!("读取响应数据失败: {err}"))?;

    let decoded: ClassroomApiResponse = serde_json::from_str(&body)
        .map_err(|err| format!("解析排课数据失败: {err}"))?;

    if decoded.code != 200 {
        return Err(format!("接口返回错误: {}", decoded.msg));
    }

    let mut floors = Vec::new();
    if let Some(data) = decoded.data {
        for f in data.floor_list {
            let floor_name = f.text.trim().to_string();
            let floor_num = extract_floor_number(&floor_name);
            let mut rooms = Vec::new();

            for r in f.children {
                let room_name = r.text.trim().to_string();
                if room_name.is_empty() {
                    continue;
                }
                rooms.push(RoomInfo {
                    id: r.id,
                    name: room_name,
                    floor_name: floor_name.clone(),
                    courses: r.room_course_list,
                });
            }

            if !rooms.is_empty() {
                // 房间按门牌字典序排序
                rooms.sort_by(|a, b| a.name.cmp(&b.name));
                floors.push(FloorInfo {
                    name: floor_name,
                    floor_num,
                    rooms,
                });
            }
        }
    }

    // 楼层按层号升序排列
    floors.sort_by_key(|f| f.floor_num);

    Ok(BuildingDaySchedule {
        building_id: building.id,
        building_name: building.name,
        campus_name: building.campus,
        floors,
    })
}

/// 楼层空教室统计。
#[derive(Debug, Clone)]
pub struct FloorEmptyRooms {
    /// 楼层名（如 "一楼" 或 "一层"）。
    pub floor_name: String,
    /// 该楼层空闲教室列表（门牌号）。
    pub available_rooms: Vec<String>,
    /// 该楼层总教室数。
    pub total_rooms: usize,
}

/// 教学楼空教室聚合结果。
#[derive(Debug, Clone)]
pub struct BuildingEmptyResult {
    /// 教学楼标准名。
    pub building_name: &'static str,
    /// 校区名。
    pub campus_name: &'static str,
    /// 各楼层空教室。
    pub floors: Vec<FloorEmptyRooms>,
    /// 全楼空闲教室总数。
    pub total_available: usize,
    /// 全楼总教室数。
    pub total_rooms: usize,
}

/// 从排课数据中计算指定节次范围与楼层的空教室分布。
#[must_use]
pub fn calculate_empty_classrooms(
    schedule: &BuildingDaySchedule,
    range: SectionQueryRange,
    filter_floor: Option<i32>,
) -> BuildingEmptyResult {
    let mut floor_results = Vec::new();
    let mut total_available = 0;
    let mut total_rooms = 0;

    for floor in &schedule.floors {
        if let Some(target_floor) = filter_floor
            && floor.floor_num != target_floor
        {
            continue;
        }

        let floor_total = floor.rooms.len();
        let mut available_rooms = Vec::new();

        for room in &floor.rooms {
            if room.is_free_for(range.start, range.end) {
                available_rooms.push(room.name.clone());
            }
        }

        total_available += available_rooms.len();
        total_rooms += floor_total;

        floor_results.push(FloorEmptyRooms {
            floor_name: floor.name.clone(),
            available_rooms,
            total_rooms: floor_total,
        });
    }

    BuildingEmptyResult {
        building_name: schedule.building_name,
        campus_name: schedule.campus_name,
        floors: floor_results,
        total_available,
        total_rooms,
    }
}

/// 匹配到的单条课程排课地点信息。
#[derive(Debug, Clone)]
pub struct CourseMatchItem {
    /// 教学楼名称。
    pub building_name: &'static str,
    /// 教室门牌。
    pub room_name: String,
    /// 课程名称。
    pub course_name: String,
    /// 教师姓名。
    pub teacher_name: String,
    /// 起始节次。
    pub start_section: u8,
    /// 结束节次。
    pub end_section: u8,
    /// 时间区间。
    pub time_str: String,
}

/// 在教学楼排课数据中按关键词搜索课程与上课地点。
#[must_use]
pub fn search_courses_in_schedules(
    schedules: &[BuildingDaySchedule],
    query: &str,
) -> Vec<CourseMatchItem> {
    let keyword = query.trim().to_lowercase();
    if keyword.is_empty() {
        return Vec::new();
    }

    let mut matches = Vec::new();

    for schedule in schedules {
        for floor in &schedule.floors {
            for room in &floor.rooms {
                for course in &room.courses {
                    let haystack = format!(
                        "{} {} {} {}",
                        schedule.building_name, room.name, course.course_name, course.teacher_name
                    )
                    .to_lowercase();

                    if haystack.contains(&keyword) {
                        let range = SectionQueryRange {
                            start: course.start_section,
                            end: course.end_section,
                        };
                        matches.push(CourseMatchItem {
                            building_name: schedule.building_name,
                            room_name: room.name.clone(),
                            course_name: course.course_name.clone(),
                            teacher_name: course.teacher_name.clone(),
                            start_section: course.start_section,
                            end_section: course.end_section,
                            time_str: range.time_range_str(),
                        });
                    }
                }
            }
        }
    }

    // 按节次先后、楼名、教室排序
    matches.sort_by(|a, b| {
        a.start_section
            .cmp(&b.start_section)
            .then_with(|| a.building_name.cmp(b.building_name))
            .then_with(|| a.room_name.cmp(&b.room_name))
    });

    matches
}

/// 单间教室全天空闲时段总结。
#[derive(Debug, Clone)]
pub struct RoomScheduleDetail {
    /// 教学楼名称。
    pub building_name: &'static str,
    /// 校区名称。
    pub campus_name: &'static str,
    /// 教室门牌号。
    pub room_name: String,
    /// 今日所有课程。
    pub courses: Vec<CourseMatchItem>,
    /// 全天空闲的节次区间总结（例如 [(1, 2), (5, 6)]）。
    pub free_sections: Vec<(u8, u8)>,
}

/// 获取单间教室全天排课日历及计算全天空闲时段。
#[must_use]
pub fn extract_room_schedule(
    schedule: &BuildingDaySchedule,
    room_query: &str,
) -> Option<RoomScheduleDetail> {
    let normalized = room_query.trim().to_lowercase();
    if normalized.is_empty() {
        return None;
    }

    // 寻找匹配门牌号的教室
    let mut found_room: Option<&RoomInfo> = None;
    for floor in &schedule.floors {
        for room in &floor.rooms {
            if room.name.eq_ignore_ascii_case(&normalized)
                || room.name.to_lowercase().contains(&normalized)
                || normalized.contains(&room.name.to_lowercase())
            {
                found_room = Some(room);
                break;
            }
        }
        if found_room.is_some() {
            break;
        }
    }

    let room = found_room?;

    // 组织课程列表
    let mut courses = Vec::new();
    for course in &room.courses {
        let range = SectionQueryRange {
            start: course.start_section,
            end: course.end_section,
        };
        courses.push(CourseMatchItem {
            building_name: schedule.building_name,
            room_name: room.name.clone(),
            course_name: course.course_name.clone(),
            teacher_name: course.teacher_name.clone(),
            start_section: course.start_section,
            end_section: course.end_section,
            time_str: range.time_range_str(),
        });
    }
    courses.sort_by_key(|c| c.start_section);

    // 计算空闲节次时段（检查 1~12 节，合并连续空闲节次）
    let mut free_sections = Vec::new();
    let mut in_free_block = false;
    let mut block_start = 0;

    for sec in 1..=12 {
        let occupied = room.courses.iter().any(|c| c.overlaps(sec, sec));
        if !occupied {
            if !in_free_block {
                in_free_block = true;
                block_start = sec;
            }
        } else if in_free_block {
            free_sections.push((block_start, sec - 1));
            in_free_block = false;
        }
    }
    if in_free_block {
        free_sections.push((block_start, 12));
    }

    Some(RoomScheduleDetail {
        building_name: schedule.building_name,
        campus_name: schedule.campus_name,
        room_name: room.name.clone(),
        courses,
        free_sections,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_section_range_defaults_to_all_day() {
        assert_eq!(parse_section_range(None), SectionQueryRange::ALL_DAY);
        assert_eq!(parse_section_range(Some("")), SectionQueryRange::ALL_DAY);
        assert_eq!(parse_section_range(Some("全天")), SectionQueryRange::ALL_DAY);
        assert_eq!(parse_section_range(Some("整天")), SectionQueryRange::ALL_DAY);
    }

    #[test]
    fn test_parse_section_range_formats() {
        assert_eq!(
            parse_section_range(Some("3-4")),
            SectionQueryRange { start: 3, end: 4 }
        );
        assert_eq!(
            parse_section_range(Some("3-4节")),
            SectionQueryRange { start: 3, end: 4 }
        );
        assert_eq!(
            parse_section_range(Some("5..8")),
            SectionQueryRange { start: 5, end: 8 }
        );
        assert_eq!(
            parse_section_range(Some("5")),
            SectionQueryRange { start: 5, end: 5 }
        );
        // 颠倒自动矫正
        assert_eq!(
            parse_section_range(Some("8-5")),
            SectionQueryRange { start: 5, end: 8 }
        );
    }
}
