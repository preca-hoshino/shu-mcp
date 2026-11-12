//! `search_articles` 工具的所有文本常量定义。
//!
//! 将字符串字面量集中管理，使 `tool.rs` 中的业务逻辑与文本内容解耦。
//! 本模块严格遵循无 Emoji 规范，采用中括号标记。

// ── 参数校验与默认值 ─────────────────────────────────────────────────────────

/// `limit` 参数的最小值。
pub const LIMIT_MIN: u8 = 1;

/// `limit` 参数的最大值。
pub const LIMIT_MAX: u8 = 50;

/// `limit` 参数的默认值。
pub const LIMIT_DEFAULT: u8 = 15;

/// `page` 参数的默认值。
pub const PAGE_DEFAULT: u32 = 1;

// ── 错误信息模板（无 Emoji） ─────────────────────────────────────────────────

/// `query` 参数为空的错误提示。
pub const ERR_QUERY_EMPTY: &str = "[错误] `query` 参数不能为空。";

/// `limit` 参数超出合法范围的错误提示。
pub const ERR_LIMIT_OUT_OF_RANGE: &str = "[错误] `limit` 参数必须在 1-50 之间。";

/// 搜索无结果的提示（无 Emoji）。
pub const WARN_NO_RESULTS: &str = "[提示] 未找到符合条件的文章。\n建议：\n1. 简化或减少搜索关键词；\n2. 将 scope 设为 \"all\" 扩大至正文检索；\n3. 检查或放宽日期过滤区间。";

/// 表格头。
pub const TABLE_HEADER: &str =
    "| 标题 | 部门与栏目 | 发布日期 | 内容摘要 |\n|------|------------|----------|----------|";
