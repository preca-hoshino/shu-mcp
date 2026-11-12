//! `get_article_detail` 工具定义与执行逻辑。

use super::strings::{ERR_URL_EMPTY, ERR_URL_INVALID_DOMAIN, WARN_NO_CONTENT};
use crate::data::client::VsbClient;
use rust_mcp_sdk::macros::{JsonSchema, mcp_tool};
use rust_mcp_sdk::schema::{CallToolResult, TextContent, schema_utils::CallToolError};
use std::fmt::Write;

/// 获取上海大学通知公告/新闻的具体正文内容、发布元数据与附件列表。
///
/// 输入由 `search_articles` 工具搜索得到的官方文章链接（或任意 `*.shu.edu.cn` 官方页面链接），
/// 自动提取文章标题、发布日期、作者/部门、转译为 Markdown 的正文以及附件���载直链。
#[mcp_tool(
    name = "get_article_detail",
    description = "<when_to_use>
- 获取上海大学官方通知、公告、新闻的具体正文内容、发布元数据与附件下载链接
- 在通过 search_articles 检索到目标文章 URL 后，提取文章完整内容进行深度阅读
</when_to_use>

<when_not_to_use>
- 检索与搜索文章列表（请使用 search_articles）
- 请求非上海大学官方域名（非 *.shu.edu.cn）的外部网页
</when_not_to_use>

<parameters>
- url: 文章页面 URL（必填）。必须为 *.shu.edu.cn 域名下的有效页面链接。
</parameters>

<output_format>
返回包含元数据、转译 Markdown 正文及附件下载清单的结构化文本。
</output_format>

<important>
- 仅允许抓取上海大学官方域名（*.shu.edu.cn）的文章页面
- 正文已完整转译为标准 Markdown 语法，保留表格、链接、图片及加粗样式
</important>",
    read_only_hint = true,
    destructive_hint = false,
    idempotent_hint = true,
    open_world_hint = true
)]
#[derive(Debug, ::serde::Deserialize, ::serde::Serialize, JsonSchema)]
pub struct GetArticleDetailTool {
    /// 文章页面绝对 URL（必填，必须属于 *.shu.edu.cn）。
    url: String,
}

impl GetArticleDetailTool {
    /// 执行获取文章详情工具。
    ///
    /// # Errors
    /// URL 参数不合法或非上海大学域名时返回带说明的 `CallToolResult::with_error`。
    /// 网络或系统级错误返回 `CallToolError`。
    pub async fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        let trimmed_url = self.url.trim();
        if trimmed_url.is_empty() {
            return Ok(CallToolResult::with_error(CallToolError::from_message(
                ERR_URL_EMPTY.to_string(),
            )));
        }

        // 校验域名是否属于 *.shu.edu.cn
        if !is_valid_shu_url(trimmed_url) {
            return Ok(CallToolResult::with_error(CallToolError::from_message(
                ERR_URL_INVALID_DOMAIN.to_string(),
            )));
        }

        let client = VsbClient::new();
        let detail = match client.fetch_article_detail(trimmed_url).await {
            Ok(d) => d,
            Err(err) => {
                return Ok(CallToolResult::with_error(CallToolError::from_message(
                    format!("[错误] 获取文章内容失败: {err}"),
                )));
            }
        };

        let mut output = String::new();

        // 标题
        let title = if detail.title.is_empty() {
            "未命名通知公告"
        } else {
            &detail.title
        };
        let _ = writeln!(output, "# {title}\n");

        // 元数据区块
        let mut meta_lines = Vec::new();
        if !detail.department.is_empty() {
            meta_lines.push(format!("- 发布部门：{}", detail.department));
        }
        if !detail.published_at.is_empty() {
            meta_lines.push(format!("- 发布时间：{}", detail.published_at));
        }
        if !detail.author.is_empty() {
            meta_lines.push(format!("- 作者/供稿：{}", detail.author));
        }
        meta_lines.push(format!("- 原文链接：{trimmed_url}"));

        for line in meta_lines {
            let _ = writeln!(output, "{line}");
        }
        output.push_str("\n---\n\n");

        // 正文内容
        if detail.markdown_content.trim().is_empty() {
            let _ = writeln!(output, "{WARN_NO_CONTENT}\n");
        } else {
            let _ = writeln!(output, "{}\n", detail.markdown_content.trim());
        }

        // 附件列表
        if !detail.attachments.is_empty() {
            output.push_str("---\n\n### 附件列表\n\n");
            for (idx, att) in detail.attachments.iter().enumerate() {
                let _ = writeln!(output, "{}. [{}]({})", idx + 1, att.label, att.url);
            }
        }

        Ok(CallToolResult::text_content(vec![TextContent::from(
            output,
        )]))
    }
}

/// 检查给定 URL 是否为合法的上海大学官方链接（*.shu.edu.cn）。
fn is_valid_shu_url(url_str: &str) -> bool {
    if let Ok(parsed) = reqwest::Url::parse(url_str) {
        if parsed.scheme() != "http" && parsed.scheme() != "https" {
            return false;
        }
        if let Some(host) = parsed.host_str() {
            let lower_host = host.to_lowercase();
            return lower_host == "shu.edu.cn" || lower_host.ends_with(".shu.edu.cn");
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_shu_url() {
        assert!(is_valid_shu_url("https://www.shu.edu.cn/info/1051/123.htm"));
        assert!(is_valid_shu_url("http://bksy.shu.edu.cn/tzgg.htm"));
        assert!(is_valid_shu_url("https://cs.shu.edu.cn/info/1081/456.htm"));
        assert!(is_valid_shu_url("https://shu.edu.cn/"));

        assert!(!is_valid_shu_url("https://example.com/info/123.htm"));
        assert!(!is_valid_shu_url("https://not-shu.edu.cn"));
        assert!(!is_valid_shu_url("javascript:void(0)"));
        assert!(!is_valid_shu_url("ftp://ftp.shu.edu.cn"));
    }
}
