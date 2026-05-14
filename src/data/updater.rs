//! 定时拉取远程仓库最新 JSON 数据。
//!
//! 通过 GitHub REST API 拉取 `preca-hoshino/shu-mcp-data` 仓库 `output/` 目录
//! 下的所有 `.json` 文件，解析后替换内存索引。默认每 1 小时执行一次。

use super::loader::replace_index;
use super::models::DepartmentData;

/// GitHub 仓库 owner/repo 标识。
const REPO: &str = "preca-hoshino/shu-mcp-data";

/// 数据文件所在目录（仓库内相对路径）。
const OUTPUT_DIR: &str = "output";

/// 默认分支名。
const DEFAULT_BRANCH: &str = "master";

/// 拉取间隔（秒）。
const REFRESH_INTERVAL_SECS: u64 = 3600;

/// GitHub API 列举目录内容的 URL 模板。
///
/// 使用 `contents` API 返回 JSON 数组，每个元素含 `name`、`download_url` 等字段。
fn contents_api_url(branch: &str) -> String {
    format!("https://api.github.com/repos/{REPO}/contents/{OUTPUT_DIR}?ref={branch}")
}

/// GitHub raw 文件下载 URL 模板。
fn raw_url(branch: &str, filename: &str) -> String {
    let encoded = urlencoding::encode(filename);
    format!("https://raw.githubusercontent.com/{REPO}/{branch}/{OUTPUT_DIR}/{encoded}")
}

/// 启动后台定时拉取任务。
///
/// 首次立即执行一次拉取，之后每 [`REFRESH_INTERVAL_SECS`] 秒重复执行。
/// 拉取失败时保留旧索引并打印错误，不中断任务循环。
pub fn spawn_updater() {
    tokio::spawn(async move {
        loop {
            if let Err(e) = fetch_and_update().await {
                eprintln!("⚠ 远程数据拉取失败: {e}");
            }
            tokio::time::sleep(std::time::Duration::from_secs(REFRESH_INTERVAL_SECS)).await;
        }
    });
}

/// 从 GitHub 拉取 output 目录文件列表，逐个下载并解析，最后替换全局索引。
///
/// # Errors
/// 网络请求失败、JSON 解析失败时返回错误描述。
pub async fn fetch_and_update() -> Result<(), String> {
    let branch = std::env::var("SHU_DATA_BRANCH").unwrap_or_else(|_| DEFAULT_BRANCH.to_string());
    let client = reqwest::Client::builder()
        .user_agent("shu-mcp/0.1.0")
        .build()
        .map_err(|e| format!("HTTP 客户端构建失败: {e}"))?;

    // ── 1. 获取目录文件列表 ──
    let url = contents_api_url(&branch);
    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("GitHub API 请求失败: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!("GitHub API 返回状态 {}", resp.status()));
    }

    let entries: Vec<serde_json::Value> = resp
        .json()
        .await
        .map_err(|e| format!("GitHub API 响应解析失败: {e}"))?;

    // 筛选 .json 文件名
    let filenames: Vec<String> = entries
        .iter()
        .filter_map(|entry| {
            let name = entry.get("name")?.as_str()?;
            if std::path::Path::new(name)
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("json"))
            {
                Some(name.to_string())
            } else {
                None
            }
        })
        .collect();

    if filenames.is_empty() {
        return Err("GitHub API 返回空文件列表".to_string());
    }

    eprintln!("📦 发现 {} 个 JSON 文件，开始下载...", filenames.len());

    // ── 2. 并发下载所有 JSON 文件 ──
    let mut handles = Vec::with_capacity(filenames.len());
    for filename in filenames {
        let download_url = raw_url(&branch, &filename);
        let client = client.clone();
        handles.push(tokio::spawn(async move {
            let resp = client
                .get(&download_url)
                .send()
                .await
                .map_err(|e| format!("{filename}: 下载失败 - {e}"))?;

            if !resp.status().is_success() {
                return Err(format!("{filename}: HTTP {}", resp.status()));
            }

            let text = resp
                .text()
                .await
                .map_err(|e| format!("{filename}: 读取响应失败 - {e}"))?;

            serde_json::from_str::<DepartmentData>(&text)
                .map_err(|e| format!("{filename}: JSON 解析失败 - {e}"))
        }));
    }

    // ── 3. 收集结果 ──
    let mut departments = Vec::with_capacity(handles.len());
    let mut errors = 0usize;

    for handle in handles {
        match handle.await {
            Ok(Ok(data)) => departments.push(data),
            Ok(Err(e)) => {
                eprintln!("⚠ {e}");
                errors += 1;
            }
            Err(e) => {
                eprintln!("⚠ 任务执行失败: {e}");
                errors += 1;
            }
        }
    }

    if departments.is_empty() {
        return Err(format!("全部 {errors} 个文件下载失败"));
    }

    departments.sort_by(|a, b| b.department.cmp(&a.department));

    eprintln!(
        "✅ 远程数据拉取完成：{} 个部门成功，{} 个失败",
        departments.len(),
        errors
    );

    // ── 4. 替换全局索引 ──
    replace_index(departments).await;

    Ok(())
}
