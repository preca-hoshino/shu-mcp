#![allow(missing_docs)]
//! `search_articles` 工具模块。
//!
//! 提供按关键词搜索上海大学各部门新闻文章的 MCP Tool。

pub mod search;
pub mod tool;

pub(super) mod strings;

pub use tool::SearchArticlesTool;
