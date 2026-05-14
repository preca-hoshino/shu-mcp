#![allow(missing_docs)]
//! 工具注册中心。
//!
//! 使用 `tool_box!` 宏将所有 MCP Tool 汇总为枚举，供 handler 路由分发。

pub mod search_articles;

use rust_mcp_sdk::tool_box;
use search_articles::SearchArticlesTool;

tool_box!(ShuTools, [SearchArticlesTool]);
