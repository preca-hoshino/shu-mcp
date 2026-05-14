//! `search_articles` 工具的所有文本常量定义。
//!
//! 将字符串字面量集中管理，使 `tool.rs` 中的业务逻辑与文本内容解耦。

// ── 参数校验 ──────────────────────────────────────────────────────────────────

/// `limit` 参数的最小值。
pub const LIMIT_MIN: u8 = 1;

/// `limit` 参数的最大值。
pub const LIMIT_MAX: u8 = 50;

/// `limit` 参数的默认值。
pub const LIMIT_DEFAULT: u8 = 20;

// ── 错误信息模板 ──────────────────────────────────────────────────────────────

/// `limit` 参数超出合法范围的错误提示。
pub const ERR_LIMIT_OUT_OF_RANGE: &str = "❌ 错误：`limit` 参数必须在 1-50 之间。";

/// `date_from` 格式不合法的错误提示。
pub const ERR_INVALID_DATE_FROM: &str = "❌ 错误：`date_from` 格式不正确，须为 YYYY-MM-DD。";

/// `date_to` 格式不合法的错误提示。
pub const ERR_INVALID_DATE_TO: &str = "❌ 错误：`date_to` 格式不正确，须为 YYYY-MM-DD。";

/// 搜索无结果的提示。
pub const WARN_NO_RESULTS: &str = "🔍 未找到匹配的文章，请尝试调整关键词或日期范围。";

/// 表格头。
pub const TABLE_HEADER: &str = "| 标题 | 部门+栏目 | 时间 |\n|------|-----------|------|";
