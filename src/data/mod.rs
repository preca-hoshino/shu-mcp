//! 数据层模块。
//!
//! 从 GitHub 远程仓库 `preca-hoshino/shu-mcp-data` 定时拉取 JSON 数据，
//! 维护全局内存索引供搜索工具并发读取。不涉及任何本地文件操作。

pub mod loader;
pub mod models;
pub mod updater;
