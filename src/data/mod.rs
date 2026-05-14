//! 数据层模块。
//!
//! 负责从 `shu-mcp-data/output/*.json` 加载爬虫数据并提供内存索引访问，
//! 并支持从 GitHub 远程仓库定时拉取最新数据。

pub mod loader;
pub mod models;
pub mod updater;
