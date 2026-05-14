//! `search_articles` 工具定义与执行逻辑。

use super::search::search_articles;
use super::strings::{
    ERR_INVALID_DATE_FROM, ERR_INVALID_DATE_TO, ERR_LIMIT_OUT_OF_RANGE, LIMIT_DEFAULT, LIMIT_MAX,
    LIMIT_MIN, TABLE_HEADER, WARN_NO_RESULTS,
};
use rust_mcp_sdk::macros::{JsonSchema, mcp_tool};
use rust_mcp_sdk::schema::{CallToolResult, TextContent, schema_utils::CallToolError};
use std::fmt::Write;

/// 搜索上海大学各部门新闻文章。
///
/// 支持多关键词（空格分隔，AND 逻辑）、日期范围过滤，返回 Markdown 表格。
#[mcp_tool(
    name = "search_articles",
    description = "<when_to_use>
- 用户需要搜索上海大学各部门/院系的新闻文章
- 用户需要按关键词查找特定主题的校园新闻
- 用户需要了解上海大学近期发生的事情
- 用户需要查找特定日期范围内的校园新闻
</when_to_use>

<when_not_to_use>
- 用户询问非上海大学相关的新闻
- 用户需要文章正文内容（本工具仅返回标题和链接）
</when_not_to_use>

<parameters>
- query: 搜索关键词（必填），多个关键词用空格分隔，所有关键词须同时出现在标题中
- date_from: 起始日期（可选，格式 YYYY-MM-DD），不填则不限
- date_to: 结束日期（可选，格式 YYYY-MM-DD），不填则不限
- limit: 最大返回条数（可选，1-50，默认 20）
</parameters>

<output_format>
返回 Markdown 表格，每行包含：标题（含链接）、部门+栏目、发布时间。
按发布时间倒序排列（最新在前）。
</output_format>

<important>
- 关键词匹配为大小写不敏感
- 多关键词之间为 AND 关系，须全部出现在标题中
- 数据来源为定时更新的爬虫，可能存在延迟
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

    /// 起始日期（可选，格式 `YYYY-MM-DD`）。
    date_from: Option<String>,

    /// 结束日期（可选，格式 `YYYY-MM-DD`）。
    date_to: Option<String>,

    /// 最大返回条数（可选，1-50，默认 20）。
    limit: Option<u8>,
}

impl SearchArticlesTool {
    /// 执行搜索工具。
    ///
    /// # Errors
    /// 参数校验失败时返回带中文说明的 `CallToolResult::with_error`。
    /// 系统级错误返回 `CallToolError`。
    #[allow(clippy::unused_async)]
    pub async fn call_tool(&self) -> Result<CallToolResult, CallToolError> {
        // ── 1. 参数校验 ──

        // limit 校验
        let limit = self.limit.unwrap_or(LIMIT_DEFAULT);
        if !(LIMIT_MIN..=LIMIT_MAX).contains(&limit) {
            return Ok(CallToolResult::with_error(CallToolError::from_message(
                ERR_LIMIT_OUT_OF_RANGE.to_string(),
            )));
        }

        // date_from 格式校验
        if let Some(ref from) = self.date_from
            && !is_valid_date(from)
        {
            return Ok(CallToolResult::with_error(CallToolError::from_message(
                ERR_INVALID_DATE_FROM.to_string(),
            )));
        }

        // date_to 格式校验
        if let Some(ref to) = self.date_to
            && !is_valid_date(to)
        {
            return Ok(CallToolResult::with_error(CallToolError::from_message(
                ERR_INVALID_DATE_TO.to_string(),
            )));
        }

        // ── 2. 执行搜索 ──

        let results = search_articles(
            &self.query,
            self.date_from.as_deref(),
            self.date_to.as_deref(),
            limit.into(),
        );

        // ── 3. 格式化输出 ──

        if results.is_empty() {
            return Ok(CallToolResult::text_content(vec![TextContent::from(
                WARN_NO_RESULTS,
            )]));
        }

        let mut output = String::from(TABLE_HEADER);
        output.push('\n');

        for r in &results {
            let a = r.article_ref.article;
            let dept = r.article_ref.department;
            let dept_col = format!("{dept} - {}", a.column);
            let _ = writeln!(
                output,
                "| [{}]({}) | {} | {} |",
                escape_md(&a.title),
                a.url,
                dept_col,
                a.date,
            );
        }

        Ok(CallToolResult::text_content(vec![TextContent::from(
            output,
        )]))
    }
}

/// 校验 `YYYY-MM-DD` 日期格式。
fn is_valid_date(s: &str) -> bool {
    if s.len() != 10 {
        return false;
    }
    let bytes = s.as_bytes();
    bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes[0..4].iter().all(u8::is_ascii_digit)
        && bytes[5..7].iter().all(u8::is_ascii_digit)
        && bytes[8..10].iter().all(u8::is_ascii_digit)
}

/// 转义 Markdown 表格中的特殊字符（管道符）。
fn escape_md(s: &str) -> String {
    s.replace('|', "\\|")
}
