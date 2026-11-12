#![allow(missing_docs)]
//! `search_articles` 工具模块。
//!
//! 提供搜索上海大学各部门新闻公告的 MCP Tool。

pub mod date_parser;
pub mod tool;

pub(super) mod strings;

pub use tool::SearchArticlesTool;
