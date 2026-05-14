//! 文章搜索核心逻辑。
//!
//! 支持多关键词 AND 搜索（空格分隔）、日期范围过滤、结果数量限制。

use crate::data::models::{ArticleRef, DepartmentData};

/// 搜索结果条目。
#[derive(Debug, Clone)]
pub struct SearchResult<'a> {
    /// 文章引用。
    pub article_ref: ArticleRef<'a>,
}

/// 执行文章搜索。
///
/// - `query`: 搜索关键词，空格分隔为多关键词（AND 逻辑），大小写不敏感。
/// - `date_from`: 起始日期（`YYYY-MM-DD`），`None` 表示不限。
/// - `date_to`: 结束日期（`YYYY-MM-DD`），`None` 表示不限。
/// - `limit`: 最大返回条数。
/// - `index`: 共享索引的只读引用。
///
/// 返回按日期倒序排列的搜索结果。
pub fn search_articles<'a>(
    query: &str,
    date_from: Option<&str>,
    date_to: Option<&str>,
    limit: usize,
    index: &'a [DepartmentData],
) -> Vec<SearchResult<'a>> {
    let keywords: Vec<String> = query.split_whitespace().map(str::to_lowercase).collect();

    if keywords.is_empty() {
        return Vec::new();
    }

    let mut results: Vec<SearchResult<'a>> = Vec::new();

    for dept in index {
        for article in &dept.articles {
            // 多关键词 AND 匹配（全部命中）
            let title_lower = article.title.to_lowercase();
            if !keywords.iter().all(|kw| title_lower.contains(kw.as_str())) {
                continue;
            }

            // 日期范围过滤
            if let Some(from) = date_from
                && article.date.as_str() < from
            {
                continue;
            }
            if let Some(to) = date_to
                && article.date.as_str() > to
            {
                continue;
            }

            results.push(SearchResult {
                article_ref: ArticleRef {
                    article,
                    department: &dept.department,
                },
            });
        }
    }

    // 按日期倒序排列
    results.sort_by(|a, b| b.article_ref.article.date.cmp(&a.article_ref.article.date));

    // 截断到 limit
    results.truncate(limit);
    results
}
