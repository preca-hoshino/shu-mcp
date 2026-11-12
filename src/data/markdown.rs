//! HTML 转换为 Markdown 工具模块。
//!
//! 将 VSB 文章正文树递归转换为标准清晰的 Markdown 语法，
//! 支持标题、段落、列表、代码、图片、超链接与表格，去除冗余样式标签并标准化排版。

use scraper::ElementRef;
use std::fmt::Write;

/// 将给定的 HTML 元素节点转换为 Markdown 字符串。
#[must_use]
pub fn element_to_markdown(root: &ElementRef<'_>, base_url: &str) -> String {
    let mut out = String::new();
    process_children(root, &mut out, base_url, 0);
    clean_markdown_output(&out)
}

/// 递归处理子节点。
fn process_children(element: &ElementRef<'_>, out: &mut String, base_url: &str, depth: usize) {
    for child in element.children() {
        if let Some(text_node) = child.value().as_text() {
            let text = clean_inline_text(text_node);
            if !text.is_empty() {
                out.push_str(&text);
            }
        } else if let Some(child_el) = ElementRef::wrap(child) {
            process_element(&child_el, out, base_url, depth);
        }
    }
}

/// 处理单个 DOM 元素。
#[allow(clippy::too_many_lines)]
fn process_element(element: &ElementRef<'_>, out: &mut String, base_url: &str, depth: usize) {
    let tag = element.value().name();

    // 忽略 script 和 style 标签
    if tag == "script" || tag == "style" {
        return;
    }

    match tag {
        "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
            let level = match tag {
                "h1" => "# ",
                "h2" => "## ",
                "h3" => "### ",
                "h4" => "#### ",
                "h5" => "##### ",
                _ => "###### ",
            };
            ensure_double_newline(out);
            out.push_str(level);
            let mut inner = String::new();
            process_children(element, &mut inner, base_url, depth + 1);
            out.push_str(inner.trim());
            out.push_str("\n\n");
        }
        "p" => {
            let mut inner = String::new();
            process_children(element, &mut inner, base_url, depth + 1);
            let trimmed = inner.trim();
            if !trimmed.is_empty() {
                ensure_double_newline(out);
                out.push_str(trimmed);
                out.push_str("\n\n");
            }
        }
        "div" => {
            // 如果内部含有块级元素，继续透传，否则视作段落
            if has_block_children(element) {
                process_children(element, out, base_url, depth);
            } else {
                let mut inner = String::new();
                process_children(element, &mut inner, base_url, depth + 1);
                let trimmed = inner.trim();
                if !trimmed.is_empty() {
                    ensure_double_newline(out);
                    out.push_str(trimmed);
                    out.push_str("\n\n");
                }
            }
        }
        "br" => {
            out.push('\n');
        }
        "hr" => {
            ensure_double_newline(out);
            out.push_str("---\n\n");
        }
        "blockquote" => {
            ensure_double_newline(out);
            let mut inner = String::new();
            process_children(element, &mut inner, base_url, depth + 1);
            for line in inner.lines() {
                let trimmed = line.trim();
                if !trimmed.is_empty() {
                    let _ = writeln!(out, "> {trimmed}");
                }
            }
            out.push('\n');
        }
        "ul" => {
            ensure_double_newline(out);
            process_list(element, out, base_url, false, depth);
            out.push('\n');
        }
        "ol" => {
            ensure_double_newline(out);
            process_list(element, out, base_url, true, depth);
            out.push('\n');
        }
        "li" => {
            let mut inner = String::new();
            process_children(element, &mut inner, base_url, depth + 1);
            let trimmed = inner.trim();
            if !trimmed.is_empty() {
                let _ = writeln!(out, "- {trimmed}");
            }
        }
        "table" => {
            // 判断是否是单单元格布局外层表（Word 导出模版常见），如果是，则穿透
            if is_single_cell_layout_table(element) {
                process_children(element, out, base_url, depth);
            } else {
                ensure_double_newline(out);
                render_table_to_markdown(element, out, base_url);
                out.push_str("\n\n");
            }
        }
        "strong" | "b" => {
            let mut inner = String::new();
            process_children(element, &mut inner, base_url, depth + 1);
            let trimmed = inner.trim();
            if !trimmed.is_empty() {
                out.push_str("**");
                out.push_str(trimmed);
                out.push_str("**");
            }
        }
        "em" | "i" => {
            let mut inner = String::new();
            process_children(element, &mut inner, base_url, depth + 1);
            let trimmed = inner.trim();
            if !trimmed.is_empty() {
                out.push('*');
                out.push_str(trimmed);
                out.push('*');
            }
        }
        "code" => {
            let mut inner = String::new();
            process_children(element, &mut inner, base_url, depth + 1);
            let trimmed = inner.trim();
            if !trimmed.is_empty() {
                out.push('`');
                out.push_str(trimmed);
                out.push('`');
            }
        }
        "a" => {
            let href = element.value().attr("href").unwrap_or("").trim();
            let mut text = String::new();
            process_children(element, &mut text, base_url, depth + 1);
            let clean_label = text.trim();

            if href.is_empty() || href.starts_with("javascript:") {
                out.push_str(clean_label);
            } else {
                let resolved = resolve_url(base_url, href);
                let label = if clean_label.is_empty() {
                    &resolved
                } else {
                    clean_label
                };
                let _ = write!(out, "[{label}]({resolved})");
            }
        }
        "img" => {
            let src = element
                .value()
                .attr("orisrc")
                .or_else(|| element.value().attr("data-src"))
                .or_else(|| element.value().attr("src"))
                .or_else(|| element.value().attr("vurl"))
                .unwrap_or("")
                .trim();

            if !src.is_empty() {
                let resolved = resolve_url(base_url, src);
                let alt = element
                    .value()
                    .attr("alt")
                    .or_else(|| element.value().attr("title"))
                    .unwrap_or("图片")
                    .trim();
                let clean_alt = if alt.is_empty() { "图片" } else { alt };
                let _ = write!(out, "![{clean_alt}]({resolved})");
            }
        }
        _ => {
            // span, font, sub, sup 等行内或其它无语义标签透传子节点
            process_children(element, out, base_url, depth);
        }
    }
}

/// 检查元素是否含有常见的块级子元素。
fn has_block_children(element: &ElementRef<'_>) -> bool {
    const BLOCK_TAGS: &[&str] = &[
        "div", "p", "table", "ul", "ol", "li", "h1", "h2", "h3", "h4", "h5", "h6", "hr",
        "blockquote",
    ];
    for child in element.children() {
        if let Some(el) = ElementRef::wrap(child) {
            let name = el.value().name();
            if BLOCK_TAGS.contains(&name) {
                return true;
            }
        }
    }
    false
}

/// 处理列表容器（ul/ol）。
fn process_list(
    list_el: &ElementRef<'_>,
    out: &mut String,
    base_url: &str,
    ordered: bool,
    depth: usize,
) {
    let mut index = 1;
    for child in list_el.children() {
        if let Some(child_el) = ElementRef::wrap(child)
            && child_el.value().name() == "li"
        {
            let mut inner = String::new();
            process_children(&child_el, &mut inner, base_url, depth + 1);
            let trimmed = inner.trim();
            if !trimmed.is_empty() {
                let indent = "  ".repeat(depth.min(4));
                if ordered {
                    let _ = writeln!(out, "{indent}{index}. {trimmed}");
                    index += 1;
                } else {
                    let _ = writeln!(out, "{indent}- {trimmed}");
                }
            }
        }
    }
}

/// 检查表格是否只是外层单单元格布局表。
fn is_single_cell_layout_table(table: &ElementRef<'_>) -> bool {
    let mut tr_count = 0;
    let mut found_tr = None;

    for child in table.children() {
        if let Some(el) = ElementRef::wrap(child) {
            match el.value().name() {
                "tr" => {
                    tr_count += 1;
                    found_tr = Some(el);
                }
                "tbody" => {
                    for tbody_child in el.children() {
                        if let Some(tbody_el) = ElementRef::wrap(tbody_child)
                            && tbody_el.value().name() == "tr"
                        {
                            tr_count += 1;
                            found_tr = Some(tbody_el);
                        }
                    }
                }
                _ => {}
            }
        }
    }

    if tr_count != 1 {
        return false;
    }

    if let Some(tr) = found_tr {
        let mut cell_count = 0;
        let mut single_cell = None;
        for child in tr.children() {
            if let Some(el) = ElementRef::wrap(child) {
                let name = el.value().name();
                if name == "td" || name == "th" {
                    cell_count += 1;
                    single_cell = Some(el);
                }
            }
        }

        if cell_count == 1
            && let Some(cell) = single_cell
        {
            return has_block_children(&cell);
        }
    }

    false
}

/// 渲染 Markdown 表格。
fn render_table_to_markdown(table: &ElementRef<'_>, out: &mut String, base_url: &str) {
    let mut rows: Vec<Vec<String>> = Vec::new();

    for tr in table.select(&scraper::Selector::parse("tr").unwrap_or_else(|_| unreachable!())) {
        let mut row = Vec::new();
        for cell in tr.children() {
            if let Some(cell_el) = ElementRef::wrap(cell) {
                let tag = cell_el.value().name();
                if tag == "td" || tag == "th" {
                    let mut cell_text = String::new();
                    process_children(&cell_el, &mut cell_text, base_url, 0);
                    // 清理换行符，表格内换行用空格或 <br> 替代，避免破坏 Markdown 表格结构
                    let cleaned = cell_text
                        .replace('\n', " ")
                        .replace('|', "\\|")
                        .trim()
                        .to_string();
                    row.push(cleaned);
                }
            }
        }
        if !row.is_empty() {
            rows.push(row);
        }
    }

    if rows.is_empty() {
        return;
    }

    // 计算最大列数
    let max_cols = rows.iter().map(Vec::len).max().unwrap_or(0);
    if max_cols == 0 {
        return;
    }

    // 填充对齐
    for (i, row) in rows.iter_mut().enumerate() {
        while row.len() < max_cols {
            row.push(String::new());
        }

        let line = format!("| {} |", row.join(" | "));
        let _ = writeln!(out, "{line}");

        // 第一行后输出分割线
        if i == 0 {
            let sep = vec!["---"; max_cols];
            let _ = writeln!(out, "| {} |", sep.join(" | "));
        }
    }
}

/// 确保写入前结尾有两个换行。
fn ensure_double_newline(out: &mut String) {
    if out.is_empty() {
        return;
    }
    if !out.ends_with("\n\n") {
        if out.ends_with('\n') {
            out.push('\n');
        } else {
            out.push_str("\n\n");
        }
    }
}

/// 清洗内联文本，去除全角不换行空格（`\u{00a0}`），保留正常字符。
fn clean_inline_text(text: &str) -> String {
    text.replace('\u{00a0}', " ")
}

/// 清洗 Markdown 最终输出，去除多余空白行。
fn clean_markdown_output(input: &str) -> String {
    let mut lines = Vec::new();
    let mut prev_blank = false;

    for line in input.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            if !prev_blank && !lines.is_empty() {
                lines.push("");
                prev_blank = true;
            }
        } else {
            lines.push(line);
            prev_blank = false;
        }
    }

    lines.join("\n").trim().to_string()
}

/// 将相对 URL 补全为绝对 URL。
#[must_use]
pub fn resolve_url(base_url: &str, relative: &str) -> String {
    let trimmed = relative.trim();
    if trimmed.is_empty() || trimmed.starts_with("javascript:") {
        return String::new();
    }

    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        return trimmed.to_string();
    }

    if let Ok(base) = reqwest::Url::parse(base_url)
        && let Ok(resolved) = base.join(trimmed)
    {
        return resolved.to_string();
    }

    trimmed.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use scraper::Html;

    #[test]
    fn test_html_headings_and_paragraphs() {
        let html = r"
            <div>
                <h2>重要通知</h2>
                <p>这是第一段内容，包含<strong>加粗文本</strong>和<a href='/info/1.htm'>链接</a>。</p>
                <p>这是第二段内容。</p>
            </div>
        ";
        let document = Html::parse_fragment(html);
        let root = document.root_element();
        let md = element_to_markdown(&root, "https://www.shu.edu.cn/info/1051/123.htm");
        assert!(md.contains("## 重要通知"));
        assert!(md.contains("这是第一段内容，包含**加粗文本**和[链接](https://www.shu.edu.cn/info/1.htm)。"));
        assert!(md.contains("这是第二段内容。"));
    }

    #[test]
    fn test_table_conversion() {
        let html = r"
            <table>
                <tr><th>姓名</th><th>部门</th></tr>
                <tr><td>张三</td><td>计院</td></tr>
            </table>
        ";
        let document = Html::parse_fragment(html);
        let root = document.root_element();
        let md = element_to_markdown(&root, "https://cs.shu.edu.cn");
        assert!(md.contains("| 姓名 | 部门 |"));
        assert!(md.contains("| --- | --- |"));
        assert!(md.contains("| 张三 | 计院 |"));
    }

    #[test]
    fn test_single_cell_layout_table_unwrapping() {
        let html = r"
            <table>
                <tr>
                    <td>
                        <p>外层被表格包裹的段落内容</p>
                    </td>
                </tr>
            </table>
        ";
        let document = Html::parse_fragment(html);
        let root = document.root_element();
        let md = element_to_markdown(&root, "https://bksy.shu.edu.cn");
        assert!(!md.contains("| --- |"));
        assert_eq!(md, "外层被表格包裹的段落内容");
    }
}
