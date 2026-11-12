//! `get_article_detail` 工具的所有文本常量定义。
//!
//! 集中管理错误提示、文本标签与 Markdown 输出模版，严格遵循无 Emoji 规范。

/// URL 参数为空的错误提示。
pub const ERR_URL_EMPTY: &str = "[错误] `url` 参数不能为空。";

/// URL 非法或非上海大学域名的错误提示。
pub const ERR_URL_INVALID_DOMAIN: &str =
    "[错误] `url` 参数必须为上海大学官方域名（*.shu.edu.cn）下的有效链接。";

/// 页面无正文内容的提示。
pub const WARN_NO_CONTENT: &str = "[提示] 未能从该页面中解析出正文内容，请检查链接是否为有效的新闻公���正文页。";
