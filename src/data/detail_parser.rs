//! 文章详情与附件解析器。
//!
//! 负责从上海大学官方 VSB 及相关站点的文章 HTML 页面中提取：
//! - 标题
//! - 发布时间
//! - 作者 / 部门
//! - Markdown 格式正文（将 HTML 转换）
//! - 附件列表（名称、文件类型及绝对下载链接）

use super::markdown::{element_to_markdown, resolve_url};
use scraper::{Html, Selector};
use std::sync::LazyLock;

/// 文章附件元数据。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArticleAttachment {
    /// 附件名称/说明标签。
    pub label: String,
    /// 附件完整下载 URL。
    pub url: String,
}

/// 解析提取后的文章详情。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArticleDetail {
    /// 文章标题。
    pub title: String,
    /// 文章页面绝对 URL。
    pub url: String,
    /// 发布日期文本（通常为 `YYYY-MM-DD`）。
    pub published_at: String,
    /// 发布人/作者（若有）。
    pub author: String,
    /// 发布部门（若有）。
    pub department: String,
    /// Markdown 格式正文内容。
    pub markdown_content: String,
    /// 附件列表。
    pub attachments: Vec<ArticleAttachment>,
}

/// 常见附件文件扩展名列表。
const ATTACHMENT_EXTENSIONS: &[&str] = &[
    ".pdf", ".docx", ".doc", ".xlsx", ".xls", ".pptx", ".ppt", ".zip", ".rar", ".7z",
];

/// `.nry` 类选择器。
static SEL_NRY: LazyLock<Selector> = LazyLock::new(|| Selector::parse(".nry").unwrap_or_else(|_| unreachable!()));
/// `h1` 选择器。
static SEL_H1: LazyLock<Selector> = LazyLock::new(|| Selector::parse("h1").unwrap_or_else(|_| unreachable!()));
/// `.art-tit h3` 选择器。
static SEL_ART_TIT_H3: LazyLock<Selector> = LazyLock::new(|| Selector::parse(".art-tit h3").unwrap_or_else(|_| unreachable!()));
/// `.Head` 选择器。
static SEL_HEAD: LazyLock<Selector> = LazyLock::new(|| Selector::parse(".Head").unwrap_or_else(|_| unreachable!()));
/// `title` 选择器。
static SEL_TITLE: LazyLock<Selector> = LazyLock::new(|| Selector::parse("title").unwrap_or_else(|_| unreachable!()));

/// `.v_news_content` 选择器。
static SEL_CONTENT_VSB: LazyLock<Selector> = LazyLock::new(|| Selector::parse(".v_news_content").unwrap_or_else(|_| unreachable!()));
/// `#vsb_content` 选择器。
static SEL_CONTENT_VSB_ID: LazyLock<Selector> = LazyLock::new(|| Selector::parse("#vsb_content").unwrap_or_else(|_| unreachable!()));
/// `div[id^="vsb_content_"]` 选择器。
static SEL_CONTENT_VSB_PREFIX: LazyLock<Selector> = LazyLock::new(|| Selector::parse("div[id^=\"vsb_content_\"]").unwrap_or_else(|_| unreachable!()));

/// `.xx` 选择器。
static SEL_XX: LazyLock<Selector> = LazyLock::new(|| Selector::parse(".xx").unwrap_or_else(|_| unreachable!()));
/// `.art-tit p` 选择器。
static SEL_ART_TIT_P: LazyLock<Selector> = LazyLock::new(|| Selector::parse(".art-tit p").unwrap_or_else(|_| unreachable!()));
/// `span` 选择器。
static SEL_SPAN: LazyLock<Selector> = LazyLock::new(|| Selector::parse("span").unwrap_or_else(|_| unreachable!()));
/// `a[href]` 选择器。
static SEL_ANCHOR: LazyLock<Selector> = LazyLock::new(|| Selector::parse("a[href]").unwrap_or_else(|_| unreachable!()));
/// `script` 选择器。
static SEL_SCRIPT: LazyLock<Selector> = LazyLock::new(|| Selector::parse("script").unwrap_or_else(|_| unreachable!()));

/// 解析给定的文章 HTML 页面并返回文章详情。
#[must_use]
pub fn parse_article_detail(html: &str, url: &str) -> ArticleDetail {
    let document = Html::parse_document(html);

    // 1. 提取标题
    let title = extract_title(&document);

    // 2. 提取元数据（时间、作者、部门）
    let (published_at, author, department) = extract_metadata(&document);

    // 3. 定位正文根节点并转为 Markdown
    let (markdown_content, content_root) = extract_content(&document, url);

    // 4. 提取附件列表（正文内外所有下载链接 + script 中嵌入的 PDF）
    let attachments = extract_attachments(&document, content_root.as_ref(), url);

    ArticleDetail {
        title,
        url: url.to_string(),
        published_at,
        author,
        department,
        markdown_content,
        attachments,
    }
}

/// 提取文章标题。
fn extract_title(document: &Html) -> String {
    // 优先：.nry h1
    if let Some(nry) = document.select(&SEL_NRY).next()
        && let Some(h1) = nry.select(&SEL_H1).next()
    {
        let t = clean_text(&h1.text().collect::<String>());
        if !t.is_empty() {
            return t;
        }
    }

    // 其次：.art-tit h3（计算机学院等模版）
    if let Some(h3) = document.select(&SEL_ART_TIT_H3).next() {
        let t = clean_text(&h3.text().collect::<String>());
        if !t.is_empty() {
            return t;
        }
    }

    // 其次：.Head
    if let Some(head) = document.select(&SEL_HEAD).next() {
        let t = clean_text(&head.text().collect::<String>());
        if !t.is_empty() {
            return t;
        }
    }

    // 回退：h1
    if let Some(h1) = document.select(&SEL_H1).next() {
        let t = clean_text(&h1.text().collect::<String>());
        if !t.is_empty() {
            return t;
        }
    }

    // 最后回退：<title>
    if let Some(title_el) = document.select(&SEL_TITLE).next() {
        clean_text(&title_el.text().collect::<String>())
    } else {
        String::new()
    }
}

/// 提取文章元数据（发布时间、作者、发布部门）。
fn extract_metadata(document: &Html) -> (String, String, String) {
    let mut date_text = String::new();
    let mut author = String::new();
    let mut department = String::new();

    // 尝试从 .xx 或 .art-tit p 中解析 span
    let meta_containers: Vec<_> = document
        .select(&SEL_XX)
        .chain(document.select(&SEL_ART_TIT_P))
        .collect();

    for container in meta_containers {
        for span in container.select(&SEL_SPAN) {
            let text = clean_text(&span.text().collect::<String>());
            parse_meta_field(&text, &mut date_text, &mut author, &mut department);
        }
    }

    // 如果还没有日期，全文档扫描包含时间的 span
    if date_text.is_empty() {
        for span in document.select(&SEL_SPAN) {
            let text = clean_text(&span.text().collect::<String>());
            if text.starts_with("发布时间：") || text.starts_with("发布时间:") || text.starts_with("时间：") || text.starts_with("时间:") {
                parse_meta_field(&text, &mut date_text, &mut author, &mut department);
                if !date_text.is_empty() {
                    break;
                }
            }
        }
    }

    // 尝试提取常见的作者标签（如 id 含 lblUser 或 lblAuthor）
    if author.is_empty()
        && let Ok(author_sel) = Selector::parse("span[id*=\"lblUser\"], span[id*=\"lblAuthor\"], span[id*=\"articleAuthor\"], a[id*=\"hypFirst\"]")
        && let Some(author_el) = document.select(&author_sel).next()
    {
        let a = clean_text(&author_el.text().collect::<String>());
        if !a.is_empty() {
            author = a;
        }
    }

    // 尝试从表格行中扫描作者（部分院系如 ITS、档案馆、国际部常见模版）
    if author.is_empty()
        && let Ok(row_sel) = Selector::parse("tr[valign=\"middle\"][height=\"20\"] font[face=\"\"] > span")
        && let Some(font_span) = document.select(&row_sel).next()
    {
        let a = clean_text(&font_span.text().collect::<String>());
        if !a.is_empty() && !a.contains("浏览次数") {
            author = a;
        }
    }

    (date_text, author, department)
}

/// 解析单条元数据字段。
fn parse_meta_field(
    text: &str,
    date_text: &mut String,
    author: &mut String,
    department: &mut String,
) {
    if let Some(stripped) = text.strip_prefix("发布时间：").or_else(|| text.strip_prefix("发布时间:")) {
        *date_text = normalize_date_string(stripped.trim());
    } else if let Some(stripped) = text.strip_prefix("时间：").or_else(|| text.strip_prefix("时间:")) {
        *date_text = normalize_date_string(stripped.trim());
    } else if let Some(stripped) = text.strip_prefix("部门：").or_else(|| text.strip_prefix("部门:")) {
        *department = stripped.trim().to_string();
    } else if is_author_label(text) {
        let val = extract_label_val(text);
        if !val.is_empty() && author.is_empty() {
            *author = val;
        }
    }
}

/// 判断是否是作者标签。
fn is_author_label(text: &str) -> bool {
    const PREFIXES: &[&str] = &[
        "投稿：", "投稿:", "作者：", "作者:", "供稿：", "供稿:",
        "发布人：", "发布人:", "发布者：", "发布者:",
    ];
    PREFIXES.iter().any(|&p| text.starts_with(p))
}

/// 提取标签后的冒号内容。
fn extract_label_val(text: &str) -> String {
    if let Some(pos) = text.find(':') {
        clean_text(&text[pos + 1..])
    } else if let Some(pos) = text.find('：') {
        clean_text(&text[pos + '：'.len_utf8()..])
    } else {
        String::new()
    }
}

/// 规范化日期字符串为 `YYYY-MM-DD`。
fn normalize_date_string(input: &str) -> String {
    let trimmed = input.trim();
    let chars: Vec<char> = trimmed.chars().collect();
    if chars.len() >= 10 {
        let s: String = chars.into_iter().collect();
        let parts: Vec<&str> = s.split(['-', '/', '.', '年', '月', '日']).collect();
        if parts.len() >= 3
            && let (Ok(y), Ok(m), Ok(d)) = (
                parts[0].trim().parse::<u32>(),
                parts[1].trim().parse::<u32>(),
                parts[2].trim().parse::<u32>(),
            )
            && (1900..=2100).contains(&y)
            && (1..=12).contains(&m)
            && (1..=31).contains(&d)
        {
            return format!("{y:04}-{m:02}-{d:02}");
        }
    }
    trimmed.to_string()
}

/// 提取正文内容并转换为 Markdown。
fn extract_content<'a>(document: &'a Html, base_url: &str) -> (String, Option<scraper::ElementRef<'a>>) {
    // 候选正文容器节点：.v_news_content, #vsb_content, div[id^="vsb_content_"], .nry
    let root = document
        .select(&SEL_CONTENT_VSB)
        .next()
        .or_else(|| document.select(&SEL_CONTENT_VSB_ID).next())
        .or_else(|| document.select(&SEL_CONTENT_VSB_PREFIX).next())
        .or_else(|| document.select(&SEL_NRY).next());

    if let Some(content_el) = root {
        let md = element_to_markdown(&content_el, base_url);
        (md, Some(content_el))
    } else {
        // 全局回退：如果找不到正文容器，将 body 作为备选
        if let Some(body) = document.select(&Selector::parse("body").unwrap_or_else(|_| unreachable!())).next() {
            (element_to_markdown(&body, base_url), Some(body))
        } else {
            (String::new(), None)
        }
    }
}

/// 提取附件下载列表（正文内外 `DownloadAttachUrl` 链接 + 常见文档扩展名链接 + VSB PDF 脚本）。
fn extract_attachments(
    document: &Html,
    content_root: Option<&scraper::ElementRef<'_>>,
    base_url: &str,
) -> Vec<ArticleAttachment> {
    let mut attachments = Vec::new();
    let mut seen_urls = std::collections::HashSet::new();

    // 1. 扫描正文及文档所有超链接
    for anchor in document.select(&SEL_ANCHOR) {
        let href = anchor.value().attr("href").unwrap_or("").trim();
        if is_attachment_href(href) {
            let full_url = resolve_url(base_url, href);
            if !full_url.is_empty() && seen_urls.insert(full_url.clone()) {
                let label = clean_text(&anchor.text().collect::<String>());
                let clean_label = if label.is_empty() {
                    extract_filename_from_url(&full_url)
                } else {
                    label
                };
                attachments.push(ArticleAttachment {
                    label: clean_label,
                    url: full_url,
                });
            }
        }
    }

    // 2. 扫描 JavaScript 中嵌入的 VSB PDF（如 showVsbpdfIframe("/__local/notice.pdf", ...)）
    if let Some(root) = content_root {
        for script in root.select(&SEL_SCRIPT) {
            let code = script.text().collect::<String>();
            if let Some(pdf_path) = parse_vsb_pdf_script(&code) {
                let full_url = resolve_url(base_url, &pdf_path);
                if !full_url.is_empty() && seen_urls.insert(full_url.clone()) {
                    attachments.push(ArticleAttachment {
                        label: extract_filename_from_url(&full_url),
                        url: full_url,
                    });
                }
            }
        }
    }

    attachments
}

/// 判断超链接是否为附件。
fn is_attachment_href(href: &str) -> bool {
    if href.contains("DownloadAttachUrl") {
        return true;
    }
    let lower = href.to_lowercase();
    let path = lower.split(['?', '#']).next().unwrap_or("");
    ATTACHMENT_EXTENSIONS.iter().any(|ext| path.ends_with(ext))
}

/// 从 URL 中提取文件名。
fn extract_filename_from_url(url: &str) -> String {
    let path = url.split(['?', '#']).next().unwrap_or(url);
    if let Some(idx) = path.rfind('/') {
        let filename = &path[idx + 1..];
        if !filename.is_empty() {
            return filename.to_string();
        }
    }
    "附件文档".to_string()
}

/// 从 VSB 脚本中解析 PDF 路径。
fn parse_vsb_pdf_script(code: &str) -> Option<String> {
    // 寻找 showVsbpdfIframe("...", ...)
    if let Some(start) = code.find("showVsbpdfIframe(") {
        let after = &code[start + "showVsbpdfIframe(".len()..];
        let quote = after.chars().find(|&c| c == '"' || c == '\'')?;
        let sub = &after[after.find(quote)? + 1..];
        let end = sub.find(quote)?;
        let path = &sub[..end];
        if std::path::Path::new(path)
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("pdf"))
        {
            return Some(path.to_string());
        }
    }
    None
}

/// 清洗字符串中的冗余空白与不间断空格。
fn clean_text(input: &str) -> String {
    input
        .replace('\u{00a0}', " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_vsb_detail_complete() {
        let html = r"
            <!DOCTYPE html>
            <html>
            <head>
                <title>关于举办程序设计竞赛的通知-计算机工程与科学学院</title>
            </head>
            <body>
                <div class='nry'>
                    <h1>关于举办2026年程序设计竞赛的通知</h1>
                    <div class='xx'>
                        <span>发布时间：2026-10-25</span>
                        <span>部门：计算机学院</span>
                        <span>作者：竞赛工作组</span>
                    </div>
                    <div class='v_news_content'>
                        <p>请各位同学积极报名参加程序设计竞赛。</p>
                        <p>附件列表见下方：</p>
                        <p><a href='/system/_content/download.jsp?urltype=news.DownloadAttachUrl&wbfileid=123'>2026竞赛规程.pdf</a></p>
                    </div>
                </div>
            </body>
            </html>
        ";

        let detail = parse_article_detail(html, "https://cs.shu.edu.cn/info/1081/12345.htm");
        assert_eq!(detail.title, "关于举办2026年程序设计竞赛的通知");
        assert_eq!(detail.published_at, "2026-10-25");
        assert_eq!(detail.department, "计算机学院");
        assert_eq!(detail.author, "竞赛工作组");
        assert!(detail.markdown_content.contains("请各位同学积极报名参加程序设计竞赛。"));
        assert_eq!(detail.attachments.len(), 1);
        assert_eq!(detail.attachments[0].label, "2026竞赛规程.pdf");
        assert_eq!(
            detail.attachments[0].url,
            "https://cs.shu.edu.cn/system/_content/download.jsp?urltype=news.DownloadAttachUrl&wbfileid=123"
        );
    }
}
