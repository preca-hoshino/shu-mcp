//! 数据层模块。
//!
//! 包含：
//! - [`client`] — 官方 VSB 搜索接口 HTTP 客户端
//! - [`models`] — 搜索请求与响应数据模型
//! - [`sources`] — 71 个全校部门元数据与全局查询常量
//!
//! 直接实时直连上海大学官方接口，不涉及任何本地文件与定时爬虫。

pub mod client;
pub mod detail_parser;
pub mod markdown;
pub mod models;
pub mod sources;
