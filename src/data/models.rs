//! 数据模型定义。
//!
//! 对应 `shu-mcp-data/output/*.json` 的 JSON 结构，用于反序列化爬虫产出。

use serde::Deserialize;
use std::collections::HashMap;

/// 单个部门的爬取数据文件结构。
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct DepartmentData {
    /// 数据来源域名。
    pub domain: String,
    /// 部门/院系名称。
    pub department: String,
    /// 站点类型编号。
    #[serde(rename = "type")]
    pub site_type: String,
    /// 爬取时间戳（格式 `YYYYMMDD_HHMMSS`）。
    pub crawl_time: String,
    /// 该部门文章总数。
    pub total_articles: u64,
    /// 栏目分布统计（栏目名 → 文章数）。
    pub columns: HashMap<String, u64>,
    /// 文章列表。
    pub articles: Vec<Article>,
}

/// 单篇文章的元数据。
#[derive(Debug, Clone, Deserialize)]
pub struct Article {
    /// 文章标题。
    pub title: String,
    /// 文章原始链接。
    pub url: String,
    /// 发布日期（`YYYY-MM-DD` 格式，支持字典序比较）。
    pub date: String,
    /// 所属栏目名称。
    pub column: String,
}

/// 带部门归属的文章引用，用于搜索结果输出。
#[derive(Debug, Clone)]
pub struct ArticleRef<'a> {
    /// 文章数据。
    pub article: &'a Article,
    /// 所属部门名称。
    pub department: &'a str,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_department_data() {
        let json = r#"{
            "domain": "www.shu.edu.cn",
            "department": "上海大学",
            "type": "16",
            "crawl_time": "20260514_004804",
            "total_articles": 2,
            "columns": {"综合新闻": 1, "通知公告": 1},
            "articles": [
                {"title": "测试文章", "url": "https://example.com", "date": "2026-05-13", "column": "综合新闻"},
                {"title": "通知", "url": "https://example.com/2", "date": "2026-05-12", "column": "通知公告"}
            ]
        }"#;

        let result: Result<DepartmentData, _> = serde_json::from_str(json);
        assert!(result.is_ok());
        if let Ok(data) = result {
            assert_eq!(data.department, "上海大学");
            assert_eq!(data.site_type, "16");
            assert_eq!(data.total_articles, 2);
            assert_eq!(data.columns.len(), 2);
            assert_eq!(data.articles.len(), 2);
            assert_eq!(data.articles[0].title, "测试文章");
            assert_eq!(data.articles[0].date, "2026-05-13");
        }
    }
}
