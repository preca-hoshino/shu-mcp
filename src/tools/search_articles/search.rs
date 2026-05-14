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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::models::{Article, DepartmentData};
    use std::collections::HashMap;

    /// 构建测试用 `DepartmentData` 列表。
    fn make_test_index() -> Vec<DepartmentData> {
        let mut columns_a = HashMap::new();
        columns_a.insert("综合新闻".to_string(), 3);
        columns_a.insert("通知公告".to_string(), 1);

        let mut columns_b = HashMap::new();
        columns_b.insert("科研动态".to_string(), 2);

        vec![
            DepartmentData {
                domain: "www.shu.edu.cn".to_string(),
                department: "上海大学".to_string(),
                site_type: "16".to_string(),
                crawl_time: "20260514_000000".to_string(),
                total_articles: 4,
                columns: columns_a,
                articles: vec![
                    Article {
                        title: "上海大学召开2026年第二次党群工作例会".to_string(),
                        url: "https://www.shu.edu.cn/info/1056/1001.htm".to_string(),
                        date: "2026-05-13".to_string(),
                        column: "综合新闻".to_string(),
                    },
                    Article {
                        title: "国际心脏研究会主席到访上海大学".to_string(),
                        url: "https://www.shu.edu.cn/info/1056/1002.htm".to_string(),
                        date: "2026-05-12".to_string(),
                        column: "综合新闻".to_string(),
                    },
                    Article {
                        title: "上海大学2026年春季学期期末考试通知".to_string(),
                        url: "https://www.shu.edu.cn/info/1056/1003.htm".to_string(),
                        date: "2026-05-10".to_string(),
                        column: "通知公告".to_string(),
                    },
                    Article {
                        title: "上海大学援疆教师获评优秀援疆干部人才".to_string(),
                        url: "https://www.shu.edu.cn/info/1056/1004.htm".to_string(),
                        date: "2026-05-13".to_string(),
                        column: "综合新闻".to_string(),
                    },
                ],
            },
            DepartmentData {
                domain: "sfs.shu.edu.cn".to_string(),
                department: "外国语学院".to_string(),
                site_type: "1".to_string(),
                crawl_time: "20260514_000000".to_string(),
                total_articles: 2,
                columns: columns_b,
                articles: vec![
                    Article {
                        title: "外国语学院科研项目申报通知".to_string(),
                        url: "https://sfs.shu.edu.cn/1001.htm".to_string(),
                        date: "2026-05-11".to_string(),
                        column: "科研动态".to_string(),
                    },
                    Article {
                        title: "上海大学外国语学院举办学术研讨会".to_string(),
                        url: "https://sfs.shu.edu.cn/1002.htm".to_string(),
                        date: "2026-05-09".to_string(),
                        column: "科研动态".to_string(),
                    },
                ],
            },
        ]
    }

    #[test]
    fn single_keyword_matches_all() {
        let index = make_test_index();
        let results = search_articles("上海大学", None, None, 50, &index);
        assert_eq!(results.len(), 5);
        assert_eq!(results[0].article_ref.article.date, "2026-05-13");
    }

    #[test]
    fn multi_keyword_and_logic() {
        let index = make_test_index();
        let results = search_articles("上海大学 教师", None, None, 50, &index);
        assert_eq!(results.len(), 1);
        assert!(results[0].article_ref.article.title.contains("援疆教师"));
    }

    #[test]
    fn case_insensitive() {
        let index = make_test_index();
        let results = search_articles("外国语学院", None, None, 50, &index);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn date_range_filter() {
        let index = make_test_index();
        let results = search_articles(
            "上海大学",
            Some("2026-05-12"),
            Some("2026-05-13"),
            50,
            &index,
        );
        for r in &results {
            assert!(r.article_ref.article.date.as_str() >= "2026-05-12");
            assert!(r.article_ref.article.date.as_str() <= "2026-05-13");
        }
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn limit_truncation() {
        let index = make_test_index();
        let results = search_articles("上海大学", None, None, 2, &index);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn empty_query_returns_empty() {
        let index = make_test_index();
        let results = search_articles("   ", None, None, 50, &index);
        assert!(results.is_empty());
    }

    #[test]
    fn no_match_returns_empty() {
        let index = make_test_index();
        let results = search_articles("量子计算", None, None, 50, &index);
        assert!(results.is_empty());
    }

    #[test]
    fn sorted_by_date_desc() {
        let index = make_test_index();
        let results = search_articles("上海大学", None, None, 50, &index);
        for i in 1..results.len() {
            assert!(results[i - 1].article_ref.article.date >= results[i].article_ref.article.date);
        }
    }

    #[test]
    fn date_from_only() {
        let index = make_test_index();
        let results = search_articles("上海大学", Some("2026-05-13"), None, 50, &index);
        for r in &results {
            assert!(r.article_ref.article.date.as_str() >= "2026-05-13");
        }
    }

    #[test]
    fn date_to_only() {
        let index = make_test_index();
        let results = search_articles("上海大学", None, Some("2026-05-10"), 50, &index);
        for r in &results {
            assert!(r.article_ref.article.date.as_str() <= "2026-05-10");
        }
    }
}
