//! 从 GitHub 远程仓库拉取 JSON 数据。
//!
//! 纯数据获取模块，不涉及任何索引管理。通过 GitHub REST API
//! 拉取 `preca-hoshino/shu-mcp-data` 仓库 `output/` 目录下的所有 `.json` 文件，
//! 解析后返回结构化数据。

use super::models::DepartmentData;

/// GitHub 仓库 owner/repo 标识。
const REPO: &str = "preca-hoshino/shu-mcp-data";

/// 数据文件所在目录（仓库内相对路径）。
const OUTPUT_DIR: &str = "output";

/// 默认分支名。
const DEFAULT_BRANCH: &str = "master";

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

/// 从 GitHub 拉取 output 目录下所有 JSON 文件并解析。
///
/// 返回解析成功的部门数据列表，按部门名称倒序排列。
///
/// # Errors
/// - 网络请求失败（GitHub API 或 raw 文件下载）
/// - JSON 解析失败
/// - 目录为空或全部文件解析失败
pub async fn fetch_from_github() -> Result<Vec<DepartmentData>, String> {
    let branch = std::env::var("SHU_DATA_BRANCH").unwrap_or_else(|_| DEFAULT_BRANCH.to_string());
    let client = reqwest::Client::builder()
        .user_agent("shu-mcp/0.1.0")
        .build()
        .map_err(|e| format!("HTTP 客户端构建失败: {e}"))?;

    // ── 1. 获取目录文件列表 ──
    let filenames = list_json_filenames(&client, &branch).await?;

    eprintln!("📦 发现 {} 个 JSON 文件，开始下载...", filenames.len());

    // ── 2. 并发下载所有 JSON 文件 ──
    let departments = download_all(&client, &branch, &filenames).await?;

    if departments.is_empty() {
        return Err(format!("全部 {} 个文件下载失败", filenames.len()));
    }

    eprintln!("✅ 远程数据拉取完成：{} 个部门", departments.len());

    Ok(departments)
}

/// 获取 GitHub API 返回的目录中所有 .json 文件名。
async fn list_json_filenames(
    client: &reqwest::Client,
    branch: &str,
) -> Result<Vec<String>, String> {
    let url = contents_api_url(branch);
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

    let filenames: Vec<String> = entries
        .iter()
        .filter_map(|entry| {
            let name = entry.get("name")?.as_str()?;
            if name
                .rsplit('.')
                .next()
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

    Ok(filenames)
}

/// 并发下载所有 JSON 文件并解析。
async fn download_all(
    client: &reqwest::Client,
    branch: &str,
    filenames: &[String],
) -> Result<Vec<DepartmentData>, String> {
    let mut handles = Vec::with_capacity(filenames.len());

    for filename in filenames {
        let download_url = raw_url(branch, filename);
        let client = client.clone();
        let filename = filename.clone();
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

    let mut departments = Vec::with_capacity(handles.len());

    for handle in handles {
        match handle.await {
            Ok(Ok(data)) => departments.push(data),
            Ok(Err(e)) => eprintln!("⚠ {e}"),
            Err(e) => eprintln!("⚠ 任务执行失败: {e}"),
        }
    }

    if !departments.is_empty() {
        departments.sort_by(|a, b| b.department.cmp(&a.department));
    }

    Ok(departments)
}
