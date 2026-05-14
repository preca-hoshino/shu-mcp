//! 数据加载器。
//!
//! 启动时扫描 `shu-mcp-data/output/*.json`，构建内存索引。使用 `OnceLock`
//! 确保全局仅加载一次，后续调用返回缓存引用。

use super::models::DepartmentData;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

/// 全局数据索引：每个元素为 `(部门数据, 文件路径)`。
static DATA_INDEX: OnceLock<Vec<DepartmentData>> = OnceLock::new();

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

/// 获取全局数据索引。首次调用时加载，后续返回缓存。
///
/// # Panics
/// 仅在数据目录完全不存在时打印错误并返回空列表（不 panic）。
pub fn get_index() -> &'static Vec<DepartmentData> {
    DATA_INDEX.get_or_init(|| {
        let dir = data_dir();
        load_all_json(&dir).unwrap_or_else(|e| {
            eprintln!("❌ 数据加载失败: {e}");
            Vec::new()
        })
    })
}
