//! 数据索引管理。
//!
//! 维护全局内存索引，供搜索工具并发读取。数据由 [`super::updater`]
//! 从 GitHub 远程仓库拉取后通过 [`replace_index`] 写入。使用 `RwLock`
//! 保护索引的并发读写。

use super::models::DepartmentData;
use tokio::sync::RwLock;

/// 全局可刷新数据索引。
static DATA_INDEX: RwLock<Vec<DepartmentData>> = RwLock::const_new(Vec::new());

/// 用新的数据列表替换全局索引。
pub async fn replace_index(new_data: Vec<DepartmentData>) {
    let mut index = DATA_INDEX.write().await;
    *index = new_data;
}

/// 获取全局数据索引的快照（克隆）。
///
/// 首次调用前应确保已通过 [`replace_index`] 写入数据，
/// 否则返回空 `Vec`。
pub async fn get_index_snapshot() -> Vec<DepartmentData> {
    let index = DATA_INDEX.read().await;
    index.clone()
}
