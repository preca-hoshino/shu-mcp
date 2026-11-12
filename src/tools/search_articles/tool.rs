//! `search_articles` 工具定义与执行逻辑。

use super::date_parser::parse_date_range;
use super::strings::{
    ERR_LIMIT_OUT_OF_RANGE, ERR_QUERY_EMPTY, LIMIT_DEFAULT, LIMIT_MAX, LIMIT_MIN, PAGE_DEFAULT,
    TABLE_HEADER, WARN_NO_RESULTS,
};
use crate::data::client::VsbClient;
use rust_mcp_sdk::macros::{JsonSchema, mcp_tool};
use rust_mcp_sdk::schema::{CallToolResult, TextContent, schema_utils::CallToolError};
use std::fmt::Write;

/// 搜索上海大学全校各部门/院系新闻公告。
///
/// 直连上海大学官方 VSB 网站群搜索系统，支持多关键词（空格分隔，AND 逻辑）、
/// 检索范围选择、单参数灵活日期区间过滤与分页。
#[mcp_tool(
    name = "search_articles",
    description = "<when_to_use>
- 查询上海大学各学院、部门的通知公告、新闻资讯、学术讲座与公示
- 按关键词查找近期或指定时间范围内的校园资讯
</when_to_use>

<when_not_to_use>
- 询问与上海大学无关的新闻资讯
- 需要获取文章完整正文全文（本工具提供标题、超链接、元数据及内容摘要供导航）
</when_not_to_use>

<parameters>
- query: 搜索关键词（必填），多个关键词用空格分隔，所有关键词须同时命中
- scope: 检索范围（可选，默认为 \"title\"）：
  • \"title\"：仅匹配标题（最精准，推荐）
  • \"body\"：仅匹配正文
  • \"all\"：标题与正文同时匹配
- date_range: 日期或日期范围过滤（可选，格式如下）：
  • 单日：YYYY-MM-DD（如 \"2026-10-25\"）
  • 闭区间：YYYY-MM-DD..YYYY-MM-DD（如 \"2026-01-01..2026-06-30\"，兼容 \"~\" 或 \"/\" 分隔）
  • 起始日至今：YYYY-MM-DD..（如 \"2026-09-01..\"）
  • 截至某日：..YYYY-MM-DD（如 \"..2026-10-31\"）
- limit: 每页返回条数（可选，1-50，默认 15）
- page: 分页页码（可选，从 1 开始，默认 1）
</parameters>

<output_format>
返回紧凑的 Markdown 表格，包含：标题（含超链接）、部门与栏目、发布日期、内容摘要。
</output_format>

<important>
- 数据直连上海大学官方 VSB 网站群搜索服务，实时全网检索
- 检索范围默认自动覆盖全校所有院系与部门
- 默认检索范围为「标题匹配」，精确度最高且响应最精简
</important>",
    read_only_hint = true,
    destructive_hint = false,
    idempotent_hint = true,
    open_world_hint = true
)]
#[derive(Debug, ::serde::Deserialize, ::serde::Serialize, JsonSchema)]
pub struct SearchArticlesTool {
    /// 搜索关键词（必填），多个关键词用空格分隔。
    query: String,

    /// 检索范围（可选，\"title\"、\"body\" 或 \"all\"，默认 \"title\"）。
    scope: Option<String>,

    /// 日期或日期区间（可选，如 \"2026-10-25\" 或 \"2026-01-01..2026-06-30\"）。
    date_range: Option<String>,

    /// 每页返回条数（可选，1-50，默认 15）。
    limit: Option<u8>,

    /// 分页页码（可选，从 1 开始，默认 1）。
    page: Option<u32>,
}

impl SearchArticlesTool {
    /// 执行搜索工具。
    ///
    /// # Errors
    /// 参数校验失败时返回带说明的 `CallToolResult::with_error`。
    /// 系统级错误返回 `CallToolError`。
    pub async fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        // ── 1. 参数校验与清洗 ──

        let trimmed_query = self.query.trim();
        if trimmed_query.is_empty() {
            return Ok(CallToolResult::with_error(CallToolError::from_message(
                ERR_QUERY_EMPTY.to_string(),
            )));
        }

        let limit = self.limit.unwrap_or(LIMIT_DEFAULT);
        if !(LIMIT_MIN..=LIMIT_MAX).contains(&limit) {
            return Ok(CallToolResult::with_error(CallToolError::from_message(
                ERR_LIMIT_OUT_OF_RANGE.to_string(),
            )));
        }

        let page = self.page.unwrap_or(PAGE_DEFAULT).max(1);

        // scope 映射与容错
        let scope_code = match self.scope.as_deref().map(str::to_lowercase).as_deref() {
            Some("body" | "content" | "text" | "正文") => 2,
            Some("all" | "both" | "全部" | "全文") => 3,
            _ => 1, // 默认为仅标题
        };

        // date_range 解析
        let parsed_dates = self.date_range.as_deref().and_then(parse_date_range);
        let begin_date = parsed_dates.as_ref().and_then(|d| d.begin_date.as_deref());
        let end_date = parsed_dates.as_ref().and_then(|d| d.end_date.as_deref());

        // ── 2. 调用 VSB 客户端执行实时搜索 ──

        let client = VsbClient::new();
        let (total, articles) = match client
            .query_articles(
                trimmed_query,
                scope_code,
                begin_date,
                end_date,
                page,
                u32::from(limit),
            )
            .await
        {
            Ok(res) => res,
            Err(err) => {
                return Ok(CallToolResult::with_error(CallToolError::from_message(
                    format!("[错误] 请求上海大学官方搜索接口失败: {err}"),
                )));
            }
        };

        // ── 3. 格式化输出 ──

        if articles.is_empty() {
            return Ok(CallToolResult::text_content(vec![TextContent::from(
                WARN_NO_RESULTS,
            )]));
        }

        let mut output = format!(
            "检索到 {total} 条结果，当前展示第 {page} 页（共 {} 条）：\n\n{}\n",
            articles.len(),
            TABLE_HEADER
        );

        for a in &articles {
            let dept_col = if a.column.is_empty() {
                a.department.clone()
            } else {
                format!("{} - {}", a.department, a.column)
            };

            let _ = writeln!(
                output,
                "| [{}]({}) | {} | {} | {} |",
                escape_md(&a.title),
                a.url,
                escape_md(&dept_col),
                a.date,
                escape_md(&a.snippet),
            );
        }

        // 如果还有下一页，提供紧凑翻页引导
        if u64::from(page) * u64::from(limit) < total {
            let next_page = page + 1;
            let _ = writeln!(
                output,
                "\n> [提示] 若需查看更多，可指定 page: {next_page} 继续获取下一页。"
            );
        }

        Ok(CallToolResult::text_content(vec![TextContent::from(
            output,
        )]))
    }
}

/// 转义 Markdown 表格中的特殊字符（管道符与换行）。
fn escape_md(s: &str) -> String {
    s.replace('|', "\\|").replace('\n', " ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_md() {
        assert_eq!(escape_md("hello|world"), "hello\\|world");
        assert_eq!(escape_md("hello\nworld"), "hello world");
    }
}
