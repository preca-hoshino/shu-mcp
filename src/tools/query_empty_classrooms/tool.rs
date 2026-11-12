//! `query_empty_classrooms` 工具定义与执行逻辑。

use super::strings::{ERR_BUILDING_EMPTY, ERR_NO_VALID_BUILDINGS};
use crate::data::classroom_engine::{
    calculate_empty_classrooms, fetch_building_schedule, parse_section_range,
};
use crate::data::classroom_resolver::resolve_multiple_buildings;
use crate::data::classroom_window::{
    NIGHT_WINDOW_ERROR_MESSAGE, default_query_date, is_external_access_open,
};
use crate::data::client::VsbClient;
use rust_mcp_sdk::macros::{JsonSchema, mcp_tool};
use rust_mcp_sdk::schema::{CallToolResult, TextContent, schema_utils::CallToolError};
use std::fmt::Write;

/// 查询上海大学空闲自习教室。
///
/// 直连上海大学计算中心官方系统。支持传入单栋或多栋教学楼进行横向并排对比，
/// 支持指定节次或默认查询全天无课空闲教室。夜间外网关闭时段自动拦截并返回提示。
#[mcp_tool(
    name = "query_empty_classrooms",
    description = "<when_to_use>
- 查询上海大学指定教学楼在指定日期的空闲自习教室
- 支持同时传入多栋教学楼进行并排横向对比（如 \"A楼, AJ楼\"）
- 支持查询特定节次（如 \"3-4\"），未传节次时默认查询全天无课的空教室
</when_to_use>

<when_not_to_use>
- 查看某间具体教室全天的完整课表日历（使用 get_classroom_schedule）
- 根据课程名称或教师姓名查找上课地点（使用 get_classroom_schedule）
</when_not_to_use>

<buildings_catalog>
全校支持查询的教学楼列表（共 22 栋有效楼宇，东区已归并至宝山）：
- 宝山校区：
  • 主楼与对应阶梯楼：A楼 / AJ楼、B楼 / BJ楼、C楼 / CJ楼、D楼 / DJ楼、GA楼 / GJ楼、EJ楼 / FJ楼
  • 独立教学楼：J楼
  • 东区楼宇：经管楼、计算机/通信楼、材料楼
- 延长校区：行健楼、第四教学楼、仪表楼
- 嘉定校区：第一教学楼、第二教学楼、文达楼
（注：计算机大楼未开放数据已剔除；\"A\"、\"AJ\"、\"经管\" 等别名均可自动识别）
</buildings_catalog>

<time_restrictions>
- 服务外网开放时间为每日 07:00 至 23:00（北京时间）。
- 夜间 23:00 至次日 07:00 为外网禁用时段，在此期间调用本工具将被系统直接拦截并返回维护提示，不发起网络请求。
</time_restrictions>

<important>
- 若不提供 sections 参数，默认查询「全天」（第 1-12 节，08:00-21:40）整日完全空闲的教室。
- 日期缺省时默认为当天。
</important>",
    read_only_hint = true,
    destructive_hint = false,
    idempotent_hint = true,
    open_world_hint = true
)]
#[derive(Debug, ::serde::Deserialize, ::serde::Serialize, JsonSchema)]
pub struct QueryEmptyClassroomsTool {
    /// 教学楼名称或以逗号/空格分隔的多栋楼（如 \"A楼\"、\"A楼, AJ楼\"、\"行健楼 第四教学楼\"、\"经管楼\"）。
    pub building: String,

    /// 查询日期，格式 YYYY-MM-DD（如 \"2026-11-13\"）。缺省时默认为当天。
    #[serde(default)]
    pub date: Option<String>,

    /// 节次或节次区间（如 \"3-4\"、\"5-8\"、\"1..2\"、单节 \"3\"）。缺省时默认查询全天（1-12节）连续空闲的教室。
    #[serde(default)]
    pub sections: Option<String>,

    /// 指定楼层过滤（例如传入 2 仅返回 2 楼教室）。缺省返回整栋楼全部楼层。
    #[serde(default)]
    pub floor: Option<i32>,
}

impl QueryEmptyClassroomsTool {
    /// 执行空教室查询。
    ///
    /// # Errors
    /// 系统级处理错误时返回 `CallToolError`。
    pub async fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let client = VsbClient::new();
        self.execute_tool(&client).await
    }

    /// 执行空教室查询逻辑。
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

        let date_str = self
            .date
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map_or_else(default_query_date, ToString::to_string);

        let range = parse_section_range(self.sections.as_deref());

        // 3. 并发/批量拉取各栋教学楼日课表数据
        let http_client = client.http_client();
        let mut results = Vec::new();
        let mut fetch_errors = Vec::new();

        for building in target_buildings {
            match fetch_building_schedule(http_client, building, &date_str).await {
                Ok(sched) => {
                    let empty_res = calculate_empty_classrooms(&sched, range, self.floor);
                    results.push(empty_res);
                }
                Err(err) => {
                    fetch_errors.push(format!("[{}] 排课数据获取失败: {}", building.name, err));
                }
            }
        }

        // 4. 组装输出 Markdown
        let mut output = String::new();
        let _ = writeln!(output, "{} {} 空教室查询结果：\n", date_str, range.label());

        if results.is_empty() && !fetch_errors.is_empty() {
            for err in fetch_errors {
                let _ = writeln!(output, "{err}");
            }
            return Ok(CallToolResult::text_content(vec![TextContent::from(output)]));
        }

        for res in results {
            let _ = writeln!(
                output,
                "[{}校区 - {}] 空闲: {} / {} 间",
                res.campus_name, res.building_name, res.total_available, res.total_rooms
            );

            if res.total_available == 0 {
                let _ = writeln!(output, "- 当前时段该楼暂无空闲教室（全部排课占用）。");
            } else {
                for fl in res.floors {
                    if fl.available_rooms.is_empty() {
                        continue;
                    }
                    let rooms_joined = fl.available_rooms.join(", ");
                    let _ = writeln!(
                        output,
                        "- {} ({}/{}): {}",
                        fl.floor_name,
                        fl.available_rooms.len(),
                        fl.total_rooms,
                        rooms_joined
                    );
                }
            }
            output.push('\n');
        }

        if range == crate::data::classroom_engine::SectionQueryRange::ALL_DAY {
            let _ = writeln!(
                output,
                "> 提示：当前默认展示全天完全空闲教室。若需查看特定节次（如当前时段），可指定 sections 参数（如 \"3-4\"、\"5-6\"）。"
            );
        }

        if !fetch_errors.is_empty() {
            output.push_str("\n部分请求提示：\n");
            for err in fetch_errors {
                let _ = writeln!(output, "- {err}");
            }
        }

        Ok(CallToolResult::text_content(vec![TextContent::from(
            output,
        )]))
    }
}
