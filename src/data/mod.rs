//! 数据层模块。
//!
//! 职责分离：
//! - [`fetcher`] — 纯数据获取，从 GitHub 拉取 JSON
//! - [`loader`] — 索引管理，协调 fetcher 与全局索引
//! - [`updater`] — 定时调度，周期性触发数据刷新
//! - [`models`] — 数据模型定义
//!
//! 不涉及任何本地文件操作。

pub mod fetcher;
pub mod loader;
pub mod models;
pub mod updater;
