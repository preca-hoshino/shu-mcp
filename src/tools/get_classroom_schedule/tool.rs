//! `get_classroom_schedule` 工具定义与执行逻辑。

use super::strings::{
    ERR_BUILDING_EMPTY, ERR_NO_VALID_BUILDINGS, WARN_ROOM_OR_QUERY_REQUIRED,
};
use crate::data::classroom_engine::{
    extract_room_schedule, fetch_building_schedule, search_courses_in_schedules,
};
use crate::data::classroom_resolver::resolve_multiple_buildings;
use crate::data::classroom_window::{
    NIGHT_WINDOW_ERROR_MESSAGE, default_query_date, is_external_access_open,
};
use crate::data::client::VsbClient;
use rust_mcp_sdk::macros::{JsonSchema, mcp_tool};
use rust_mcp_sdk::schema::{CallToolResult, TextContent, schema_utils::CallToolError};
use std::fmt::Write;

/// 教室排课日程与课程地点查询。
///
/// 直连上海大学计算中心官方系统。支持查看单间教室全天课表及空闲时段，
/// 或在指定教学楼中模糊反查课程/教师上课地点。夜间外网关闭时段自动拦截并返回提示。
#[mcp_tool(
    name = "get_classroom_schedule",
    description = "<when_to_use>
- 查询指定教室在某一天全天的完整课程安排与全天空闲自习时段（传入 room）
- 在指定教学楼中按课程名、教师名或门牌号反向搜索上课地点（传入 query，支持多楼并排）
</when_to_use>

<when_not_to_use>
- 仅需快速查询某时段有哪些空教室（应使用 query_empty_classrooms）
</when_not_to_use>

<buildings_catalog>
全校支持查询的教学楼列表（共 22 栋有效楼宇，东区已归并至宝山）：
- 宝山校区：A楼/AJ楼, B楼/BJ楼, C楼/CJ楼, D楼/DJ楼, GA楼/GJ楼, EJ楼/FJ楼, J楼, 经管楼, 计算机/通信楼, 材料楼
- 延长校区：行健楼, 第四教学楼, 仪表楼
- 嘉定校区：第一教学楼, 第二教学楼, 文达楼
</buildings_catalog>

<time_restrictions>
- 服务外网开放时间为每日 07:00 至 23:00（北京时间）。
- 夜间 23:00 至次日 07:00 期间调用本工具将被系统直接拦截并返回维护提示，不发起网络请求。
</time_restrictions>

<important>
- 必须指定 building，并且至少提供 room（查具体教室日程）或 query（反查课程地点）其中之一。
- 若传入 room（如 \"A101\"），返回该教室一整天的课程表及全天空闲时段总结。
- 若传入 query（如 \"微积分\"、\"张老师\"），在指定楼宇中检索匹配的所有排课教室与时段。
</important>",
    read_only_hint = true,
    destructive_hint = false,
    idempotent_hint = true,
    open_world_hint = true
)]
#[derive(Debug, ::serde::Deserialize, ::serde::Serialize, JsonSchema)]
pub struct GetClassroomScheduleTool {
    /// 教学楼名称或多栋楼（以逗号/空格分隔，如 \"A楼\"、\"A楼, AJ楼\"、\"行健楼\"）。
    pub building: String,

    /// 教室门牌号（如 \"A101\"、\"201\"、\"AJ204\"）。传入时生成该教室全天排课日历及空闲时段。
    #[serde(default)]
    pub room: Option<String>,

    /// 课程检索关键词。模糊匹配课程名、授课教师名或教室门牌号。
    #[serde(default)]
    pub query: Option<String>,

    /// 查询日期，格式 YYYY-MM-DD。缺省为当天。
    #[serde(default)]
    pub date: Option<String>,
}

impl GetClassroomScheduleTool {
    /// 执行日程或地点查询。
    ///
    /// # Errors
    /// 系统级处理错误时返回 `CallToolError`。
    pub async fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let client = VsbClient::new();
        self.execute_tool(&client).await
    }

    /// 执行日程或地点查询核心逻辑。
    ///
    /// # Errors
    /// 系统级处理错误时返回 `CallToolError`。
    pub async fn execute_tool(
        &self,
        client: &VsbClient,
    ) -> Result<CallToolResult, CallToolError> {
        // 1. 时间守卫：夜间禁用时段直接短路拦截
        if !is_external_access_open() {
            return Ok(CallToolResult::text_content(vec![TextContent::from(
                NIGHT_WINDOW_ERROR_MESSAGE,
            )]));
        }

        // 2. 校验参数
        let raw_building = self.building.trim();
        if raw_building.is_empty() {
            return Ok(CallToolResult::text_content(vec![TextContent::from(
                ERR_BUILDING_EMPTY,
            )]));
        }

        let target_buildings = resolve_multiple_buildings(raw_building);
        if target_buildings.is_empty() {
            return Ok(CallToolResult::text_content(vec![TextContent::from(
                ERR_NO_VALID_BUILDINGS,
            )]));
        }

        let room_param = self.room.as_deref().map(str::trim).filter(|s| !s.is_empty());
        let query_param = self.query.as_deref().map(str::trim).filter(|s| !s.is_empty());

        if room_param.is_none() && query_param.is_none() {
            return Ok(CallToolResult::text_content(vec![TextContent::from(
                WARN_ROOM_OR_QUERY_REQUIRED,
            )]));
        }

        let date_str = self
            .date
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map_or_else(default_query_date, ToString::to_string);

        // 3. 拉取目标教学楼的排课数据
        let http_client = client.http_client();
        let mut schedules = Vec::new();
        let mut fetch_errors = Vec::new();

        for building in target_buildings {
            match fetch_building_schedule(http_client, building, &date_str).await {
                Ok(sched) => schedules.push(sched),
                Err(err) => {
                    fetch_errors.push(format!("[{}] 排课数据获取失败: {}", building.name, err));
                }
            }
        }

        if schedules.is_empty() {
            let mut output = String::new();
            let _ = writeln!(output, "[错误] 教学楼排课数据拉取失败：");
            for err in fetch_errors {
                let _ = writeln!(output, "- {err}");
            }
            return Ok(CallToolResult::text_content(vec![TextContent::from(
                output,
            )]));
        }

        // 4. 场景分流格式化
        if let Some(target_room) = room_param {
            return Ok(CallToolResult::text_content(vec![TextContent::from(
                format_room_schedule(&schedules, target_room, &date_str),
            )]));
        }

        if let Some(target_query) = query_param {
            return Ok(CallToolResult::text_content(vec![TextContent::from(
                format_course_query(&schedules, target_query, &date_str),
            )]));
        }

        Ok(CallToolResult::text_content(vec![TextContent::from(
            WARN_ROOM_OR_QUERY_REQUIRED,
        )]))
    }
}

/// 格式化单间教室排课日程。
fn format_room_schedule(
    schedules: &[crate::data::classroom_models::BuildingDaySchedule],
    target_room: &str,
    date_str: &str,
) -> String {
    let mut output = String::new();
    let mut found = false;

    for sched in schedules {
        if let Some(detail) = extract_room_schedule(sched, target_room) {
            found = true;
            let _ = writeln!(
                output,
                "[{}校区 - {} - {}] {date_str} 教室日程\n",
                detail.campus_name, detail.building_name, detail.room_name
            );

            if detail.courses.is_empty() {
                let _ = writeln!(output, "今日暂无排课记录（全天完全空闲）。\n");
            } else {
                let _ = writeln!(output, "已排课程：");
                let _ = writeln!(output, "| 节次 | 时间 | 课程名称 | 授课教师 |");
                let _ = writeln!(output, "|------|------|----------|----------|");
                for c in detail.courses {
                    let _ = writeln!(
                        output,
                        "| {}-{} 节 | {} | {} | {} |",
                        c.start_section,
                        c.end_section,
                        c.time_str,
                        c.course_name,
                        if c.teacher_name.is_empty() { "-" } else { &c.teacher_name }
                    );
                }
                output.push('\n');
            }

            let _ = writeln!(output, "全天空闲时段：");
            if detail.free_sections.is_empty() {
                let _ = writeln!(output, "- 今日全天均有课程占用。");
            } else {
                for (s, e) in detail.free_sections {
                    let time_range = crate::data::classroom_engine::SectionQueryRange {
                        start: s,
                        end: e,
                    };
                    let _ = writeln!(output, "- 第 {s}-{e} 节 ({})", time_range.time_range_str());
                }
            }
            break;
        }
    }

    if !found {
        let _ = writeln!(
            output,
            "[提示] 在选定教学楼（{}）中未找到门牌号为 \"{target_room}\" 的教室。",
            schedules
                .iter()
                .map(|s| s.building_name)
                .collect::<Vec<_>>()
                .join(", ")
        );
    }

    output
}

/// 格式化课程地点反查结果。
fn format_course_query(
    schedules: &[crate::data::classroom_models::BuildingDaySchedule],
    target_query: &str,
    date_str: &str,
) -> String {
    let mut output = String::new();
    let matches = search_courses_in_schedules(schedules, target_query);
    let _ = writeln!(
        output,
        "{date_str} 关键词 \"{target_query}\" 匹配课程：\n"
    );

    if matches.is_empty() {
        let _ = writeln!(output, "未在指定教学楼中匹配到包含该关键词的课程或排课信息。");
    } else {
        let _ = writeln!(output, "| 教学楼 | 教室 | 节次 | 时间 | 课程名称 | 授课教师 |");
        let _ = writeln!(output, "|--------|------|------|------|----------|----------|");
        for m in matches {
            let _ = writeln!(
                output,
                "| {} | {} | {}-{} 节 | {} | {} | {} |",
                m.building_name,
                m.room_name,
                m.start_section,
                m.end_section,
                m.time_str,
                m.course_name,
                if m.teacher_name.is_empty() { "-" } else { &m.teacher_name }
            );
        }
    }

    output
}
