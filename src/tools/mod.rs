#![allow(missing_docs)]
//! 工具注册中心。
//!
//! 使用 `tool_box!` 宏将所有 MCP Tool 汇总为枚举，供 handler 路由分发。

pub mod get_article_detail;
pub mod get_classroom_schedule;
pub mod query_empty_classrooms;
pub mod search_articles;

use get_article_detail::GetArticleDetailTool;
use get_classroom_schedule::GetClassroomScheduleTool;
use query_empty_classrooms::QueryEmptyClassroomsTool;
use rust_mcp_sdk::tool_box;
use search_articles::SearchArticlesTool;

tool_box!(
    ShuTools,
    [
        SearchArticlesTool,
        GetArticleDetailTool,
        QueryEmptyClassroomsTool,
        GetClassroomScheduleTool
    ]
);
