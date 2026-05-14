//! 数据加载器与内存索引管理。
//!
//! 启动时扫描 `shu-mcp-data/output/*.json` 构建内存索引，后续通过
//! [`refresh_index`] 支持定时热刷新。使用 `RwLock` 保护索引的并发读写。

use super::models::DepartmentData;
use std::path::{Path, PathBuf};
use tokio::sync::RwLock;

/// 全局可刷新数据索引。
static DATA_INDEX: RwLock<Vec<DepartmentData>> = RwLock::const_new(Vec::new());

/// 默认数据目录（相对于 shu-mcp 项目根目录）。
const DEFAULT_DATA_DIR: &str = "../shu-mcp-data/output";

/// 获取数据目录路径。优先使用环境变量 `SHU_DATA_PATH`，否则用默认相对路径。
fn data_dir() -> PathBuf {
    if let Ok(custom) = std::env::var("SHU_DATA_PATH") {
        PathBuf::from(custom)
    } else {
        PathBuf::from(DEFAULT_DATA_DIR)
    }
}

/// 扫描目录下所有 `.json` 文件并解析为 `DepartmentData` 列表。
///
/// # Errors
/// 若 glob 模式编译失败或目录不可读，返回错误信息。
fn load_all_json(dir: &Path) -> Result<Vec<DepartmentData>, String> {
    let pattern = dir.join("*.json").to_string_lossy().into_owned();

    let paths = glob::glob(&pattern).map_err(|e| format!("glob 模式编译失败: {e}"))?;

    let mut departments = Vec::new();

    for entry in paths {
        let path = match entry {
            Ok(p) => p,
            Err(e) => {
                eprintln!("⚠ 跳过不可读的目录条目: {e}");
                continue;
            }
        };

        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("⚠ 跳过文件 {}: {e}", path.display());
                continue;
            }
        };

        match serde_json::from_str::<DepartmentData>(&content) {
            Ok(data) => departments.push(data),
            Err(e) => {
                eprintln!("⚠ 跳过文件 {}: JSON 解析失败 - {e}", path.display());
            }
        }
    }

    departments.sort_by(|a, b| b.department.cmp(&a.department));
    Ok(departments)
}

/// 从本地数据目录加载并刷新全局索引。
///
/// 若加载失败，保留旧索引不变并打印错误。
pub async fn refresh_from_local() {
    let dir = data_dir();
    match load_all_json(&dir) {
        Ok(new_data) => {
            let mut index = DATA_INDEX.write().await;
            *index = new_data;
            eprintln!("✅ 本地数据加载完成，共 {} 个部门", index.len());
        }
        Err(e) => {
            eprintln!("❌ 本地数据加载失败: {e}");
        }
    }
}

/// 用新的数据列表替换全局索引。
pub async fn replace_index(new_data: Vec<DepartmentData>) {
    let mut index = DATA_INDEX.write().await;
    *index = new_data;
}

/// 获取全局数据索引的只读快照。调用方须持有读锁期间使用返回值。
///
/// 首次调用前应确保已执行 [`refresh_from_local`] 或 [`replace_index`]。
pub async fn get_index_snapshot() -> Vec<DepartmentData> {
    let index = DATA_INDEX.read().await;
    index.clone()
}

/// 获取全局数据索引的读锁引用。持有期间可安全读取。
pub async fn get_index_ref() -> tokio::sync::RwLockReadGuard<'static, Vec<DepartmentData>> {
    DATA_INDEX.read().await
}
