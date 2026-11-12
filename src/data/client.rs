//! 上海大学官方 VSB 搜索接口 HTTP 客户端实现。

use super::models::{
    CleanedArticle, VsbArticleRecord, VsbPageParam, VsbSearchRequest, VsbSearchResponse,
};
use super::sources::{GLOBAL_COLUMN_IDS, GLOBAL_OWNERS, get_source_by_owner, resolve_article_url};
use std::time::Duration;

/// VSB 官方搜索 API 端点。
const VSB_SEARCH_ENDPOINT: &str =
    "https://www.shu.edu.cn/aop_component/webber/search/search/search/queryPage";

/// HTTP 请求超时时间（10 秒）。
const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);

/// 最大重试次数。
const MAX_RETRIES: usize = 2;

/// 重试基础延迟（300 毫秒）。
const RETRY_BASE_DELAY: Duration = Duration::from_millis(300);

/// 摘要最大字符长度。
const SNIPPET_MAX_CHARS: usize = 70;

/// 上海大学官方 VSB 搜索客户端。
#[derive(Debug, Clone)]
pub struct VsbClient {
    /// 底层 HTTP 客户端。
    client: reqwest::Client,
}

impl Default for VsbClient {
    fn default() -> Self {
        Self::new()
    }
}

impl VsbClient {
    /// 创建新的 VSB 客户端实例。
    #[must_use]
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        Self { client }
    }

    /// 实时搜索文章列表。
    ///
    /// # 参数
    /// - `query`: 搜索关键词（AND 逻辑，支持空格分隔）
    /// - `scope`: 检索范围（1=仅标题，2=仅正文，3=标题与正文）
    /// - `begin_date`: 起始时间（`YYYY-MM-DD HH:mm:ss`）
    /// - `end_date`: 截止时间（`YYYY-MM-DD HH:mm:ss`）
    /// - `page`: 页码（从 1 开始）
    /// - `limit`: 每页大小（1-50）
    ///
    /// # Errors
    /// 网络请求失败、多次重试超时或解析失败时返回错误描述字符串。
    pub async fn query_articles(
        &self,
        query: &str,
        scope: i32,
        begin_date: Option<&str>,
        end_date: Option<&str>,
        page: u32,
        limit: u32,
    ) -> Result<(u64, Vec<CleanedArticle>), String> {
        let current_page = page.saturating_sub(1);
        let search_date_type = if begin_date.is_some() || end_date.is_some() {
            Some("custom")
        } else {
            None
        };

        let request_body = VsbSearchRequest {
            alias_name: "article_data",
            key_word: query,
            search_key_word: true,
            search_aggregation: false,
            search_type: "text",
            search_scope: scope,
            search_operator: 1,
            order_type: "date",
            allsite: true,
            owner: GLOBAL_OWNERS,
            column_id: GLOBAL_COLUMN_IDS,
            auditing: &["1"],
            search_date_type,
            begin_date,
            end_date,
            page: VsbPageParam {
                current: current_page,
                size: limit,
            },
        };

        let mut last_err = String::new();

        for attempt in 0..=MAX_RETRIES {
            if attempt > 0 {
                tokio::time::sleep(RETRY_BASE_DELAY * (1 << (attempt - 1))).await;
            }

            let response_result = self
                .client
                .post(VSB_SEARCH_ENDPOINT)
                .header("Authorization", "preview")
                .header("owner", "1864855249")
                .header("allsite", "1")
                .header("Content-Type", "application/json;charset=UTF-8")
                .json(&request_body)
                .send()
                .await;

            let response = match response_result {
                Ok(resp) => resp,
                Err(err) => {
                    last_err = format!("网络请求失败: {err}");
                    continue;
                }
            };

            let status = response.status();
            if status.is_server_error() || status.as_u16() == 408 || status.as_u16() == 429 {
                last_err = format!("上游服务器返回临时错误码 ({status})");
                continue;
            }

            if !status.is_success() {
                return Err(format!("上游接口响应异常 ({status})"));
            }

            let vsb_resp: VsbSearchResponse = match response.json().await {
                Ok(data) => data,
                Err(err) => return Err(format!("解析上游响应 JSON 失败: {err}")),
            };

            if vsb_resp.code != "0000" {
                let msg = vsb_resp.message.unwrap_or_else(|| "未知错误".to_string());
                return Err(format!("上游业务处理失败: {msg}"));
            }

            let page_data =
                vsb_resp
                    .data
                    .and_then(|d| d.page)
                    .unwrap_or(super::models::VsbPageResult {
                        current: current_page,
                        size: limit,
                        total: 0,
                        records: None,
                    });

            let total = page_data.total;
            let raw_records = page_data.records.unwrap_or_default();

            let mut articles = Vec::with_capacity(raw_records.len());
            for record in raw_records {
                if let Some(cleaned) = clean_record(&record) {
                    articles.push(cleaned);
                }
            }

            return Ok((total, articles));
        }

        Err(if last_err.is_empty() {
            "搜索请求超时重试超限".to_string()
        } else {
            last_err
        })
    }
}

/// 清洗单条文章记录。
fn clean_record(record: &VsbArticleRecord) -> Option<CleanedArticle> {
    let raw_title = record.title.as_deref()?;
    let title = strip_html_tags(raw_title);
    if title.is_empty() {
        return None;
    }

    let raw_url = record.url.as_deref()?;
    if raw_url.trim().is_empty() {
        return None;
    }

    let owner_str = match &record.owner {
        Some(serde_json::Value::String(s)) => s.clone(),
        Some(serde_json::Value::Number(n)) => n.to_string(),
        _ => "1864855249".to_string(),
    };

    let url = resolve_article_url(&owner_str, raw_url);

    let department = get_source_by_owner(&owner_str)
        .map_or_else(|| "上海大学".to_string(), |s| s.name.to_string());

    let column = record
        .column_name
        .as_deref()
        .map(strip_html_tags)
        .unwrap_or_default();

    let date = record
        .create_date
        .as_deref()
        .map(|d| {
            if d.len() >= 10 {
                d[0..10].to_string()
            } else {
                d.to_string()
            }
        })
        .unwrap_or_default();

    let raw_snippet = record
        .intro
        .as_deref()
        .or(record.summary.as_deref())
        .or(record.content.as_deref())
        .unwrap_or("");

    let snippet_text = strip_html_tags(raw_snippet);
    let snippet = truncate_chars(&snippet_text, SNIPPET_MAX_CHARS);

    Some(CleanedArticle {
        title,
        url,
        department,
        column,
        date,
        snippet,
    })
}

/// 剥离 HTML 标签并转换常见字符实体。
fn strip_html_tags(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut inside_tag = false;

    for ch in input.chars() {
        match ch {
            '<' => inside_tag = true,
            '>' => inside_tag = false,
            _ if !inside_tag => result.push(ch),
            _ => {}
        }
    }

    // 简单替换常见 HTML 实体
    let unescaped = result
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&nbsp;", " ")
        .replace("&#39;", "'");

    // 压缩多余连续空格和换行
    unescaped.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// 按 Unicode 字符数量截断字符串。
fn truncate_chars(s: &str, max_chars: usize) -> String {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() <= max_chars {
        s.to_string()
    } else {
        let truncated: String = chars[..max_chars].iter().collect();
        format!("{truncated}...")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_html_tags() {
        let input = "关于<span style=\"color:red\">竞赛</span> &amp; 通知";
        assert_eq!(strip_html_tags(input), "关于竞赛 & 通知");
    }

    #[test]
    fn test_truncate_chars() {
        let input = "一二三四五六七八九十";
        assert_eq!(truncate_chars(input, 5), "一二三四五...");
        assert_eq!(truncate_chars(input, 15), "一二三四五六七八九十");
    }
}
