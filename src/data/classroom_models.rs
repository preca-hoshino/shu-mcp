//! 上海大学空教室系统数据模型与时间守卫。

use serde::{Deserialize, Serialize};

/// 节次作息时间常量（1~12 节）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SectionTime {
    /// 节次序号（1-12）。
    pub index: u8,
    /// 开始时间（例如 "08:00"）。
    pub start_time: &'static str,
    /// 结束时间（例如 "08:45"）。
    pub end_time: &'static str,
}

/// 全校标准 12 节课节次时刻表。
pub const STANDARD_SECTIONS: [SectionTime; 12] = [
    SectionTime { index: 1, start_time: "08:00", end_time: "08:45" },
    SectionTime { index: 2, start_time: "08:55", end_time: "09:40" },
    SectionTime { index: 3, start_time: "10:00", end_time: "10:45" },
    SectionTime { index: 4, start_time: "10:55", end_time: "11:40" },
    SectionTime { index: 5, start_time: "13:00", end_time: "13:45" },
    SectionTime { index: 6, start_time: "13:55", end_time: "14:40" },
    SectionTime { index: 7, start_time: "15:00", end_time: "15:45" },
    SectionTime { index: 8, start_time: "15:55", end_time: "16:40" },
    SectionTime { index: 9, start_time: "18:00", end_time: "18:45" },
    SectionTime { index: 10, start_time: "18:55", end_time: "19:40" },
    SectionTime { index: 11, start_time: "20:00", end_time: "20:45" },
    SectionTime { index: 12, start_time: "20:55", end_time: "21:40" },
];

/// 教学楼排课响应最外层。
#[derive(Debug, Clone, Deserialize)]
pub struct ClassroomApiResponse {
    /// 响应状态码（200 为成功）。
    pub code: i32,
    /// 响应消息。
    #[serde(default)]
    pub msg: String,
    /// 数据体。
    pub data: Option<ClassroomApiData>,
}

/// 教学楼排课数据体。
#[derive(Debug, Clone, Deserialize)]
pub struct ClassroomApiData {
    /// 楼层列表。
    #[serde(rename = "floorList", default)]
    pub floor_list: Vec<ClassroomFloorRaw>,
}

/// 楼层原始数据。
#[derive(Debug, Clone, Deserialize)]
pub struct ClassroomFloorRaw {
    /// 楼层 ID。
    #[serde(default)]
    pub id: i64,
    /// 楼层文本（如 "一层"、"二层"）。
    #[serde(default)]
    pub text: String,
    /// 教室列表。
    #[serde(default)]
    pub children: Vec<ClassroomRoomRaw>,
}

/// 教室原始数据。
#[derive(Debug, Clone, Deserialize)]
pub struct ClassroomRoomRaw {
    /// 教室 ID。
    #[serde(default)]
    pub id: i64,
    /// 教室门牌名称（如 "A101"）。
    #[serde(default)]
    pub text: String,
    /// 完整路径。
    #[serde(rename = "fullName", default)]
    pub full_name: String,
    /// 当日课程排课列表。
    #[serde(rename = "roomCourseList", default)]
    pub room_course_list: Vec<ClassroomCourseRaw>,
}

/// 单门课程排课原始记录。
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ClassroomCourseRaw {
    /// 课程名称。
    #[serde(rename = "courseName", default)]
    pub course_name: String,
    /// 授课教师姓名。
    #[serde(rename = "teacherName", default)]
    pub teacher_name: String,
    /// 起始节次（1-12）。
    #[serde(rename = "startSection", default)]
    pub start_section: u8,
    /// 结束节次（1-12）。
    #[serde(rename = "endSection", default)]
    pub end_section: u8,
    /// 数据来源。
    #[serde(default)]
    pub sjly: String,
}

impl ClassroomCourseRaw {
    /// 检查该课程排课是否与指定的节次范围 [start, end] 存在时间重叠。
    #[must_use]
    pub fn overlaps(&self, start: u8, end: u8) -> bool {
        self.start_section <= end && self.end_section >= start
    }
}

/// 规范化后的单间教室信息与课程。
#[derive(Debug, Clone)]
pub struct RoomInfo {
    /// 教室 ID。
    pub id: i64,
    /// 门牌号（如 "A101"）。
    pub name: String,
    /// 所属楼层（如 "一层"）。
    pub floor_name: String,
    /// 当日所有排课。
    pub courses: Vec<ClassroomCourseRaw>,
}

impl RoomInfo {
    /// 判断该教室内在指定节次区间 [start, end] 内是否完全空闲。
    #[must_use]
    pub fn is_free_for(&self, start: u8, end: u8) -> bool {
        self.courses.iter().all(|c| !c.overlaps(start, end))
    }
}

/// 规范化后的楼层与可用教室信息。
#[derive(Debug, Clone)]
pub struct FloorInfo {
    /// 楼层名（如 "一层"）。
    pub name: String,
    /// 楼层数值排序权重（例如 1, 2, 3）。
    pub floor_num: i32,
    /// 该楼层的所有教室。
    pub rooms: Vec<RoomInfo>,
}

/// 某一栋教学楼拉取并规范化后的全天排课日程。
#[derive(Debug, Clone)]
pub struct BuildingDaySchedule {
    /// 教学楼 ID。
    pub building_id: i32,
    /// 教学楼标准名（如 "A楼"）。
    pub building_name: &'static str,
    /// 所属校区（"宝山"、"延长"、"嘉定"）。
    pub campus_name: &'static str,
    /// 各楼层数据。
    pub floors: Vec<FloorInfo>,
}
