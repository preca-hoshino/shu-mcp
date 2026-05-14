//! 数据索引管理。
//!
//! 职责：
//! - 维护全局内存索引，供搜索工具并发读取
//! - 提供数据刷新接口，协调 [`super::fetcher`] 从 GitHub 拉取数据
//!
//! 使用 `RwLock` 保护索引的并发读写。

use super::models::DepartmentData;
use tokio::sync::RwLock;

/// 全局可刷新数据索引。
static DATA_INDEX: RwLock<Vec<DepartmentData>> = RwLock::const_new(Vec::new());

/// 用新的数据列表替换全局索引。
///
/// 使用 `std::mem::swap` 实现瞬时替换，写锁持有时间极短，
/// 不会阻塞并发的读操作（`get_index_snapshot`）。
async fn replace_index(new_data: Vec<DepartmentData>) {
    let mut slot = DATA_INDEX.write().await;
    let mut incoming = new_data;
    std::mem::swap(&mut *slot, &mut incoming);
    // incoming 现持有旧数据，slot 持有新数据；函数结束时旧数据被释放
}

/// 获取全局数据索引的快照（克隆）。
///
/// 首次调用前应确保已通过 [`refresh_from_remote`] 写入数据，
/// 否则返回空 `Vec`。
pub async fn get_index_snapshot() -> Vec<DepartmentData> {
    let index = DATA_INDEX.read().await;
    index.clone()
}

/// 从 GitHub 远程仓库拉取最新数据并更新全局索引。
///
/// 此函数是数据获取与索引管理的协调层：
/// 1. 调用 [`super::fetcher::fetch_from_github`] 获取远程数据
/// 2. 成功后调用 [`replace_index`] 更新全局索引
///
/// # Errors
/// 网络请求失败、JSON 解析失败时返回错误描述。
pub async fn refresh_from_remote() -> Result<(), String> {
    let departments = super::fetcher::fetch_from_github().await?;
    let count = departments.len();
    replace_index(departments).await;
    eprintln!("✅ 数据索引已更新：{count} 个部门");
    Ok(())
}
