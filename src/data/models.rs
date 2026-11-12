//! VSB 官方搜索 API 的请求与响应数据模型。

use serde::{Deserialize, Serialize};

/// 发送给 VSB 搜索接口的请求体。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VsbSearchRequest<'a> {
    /// 索引别名，固定为 `"article_data"`。
    pub alias_name: &'static str,
    /// 搜索关键词。
    pub key_word: &'a str,
    /// 是否开启关键词高亮/分词，固定为 `true`。
    pub search_key_word: bool,
    /// 是否聚合统计，固定为 `false`。
    pub search_aggregation: bool,
    /// 搜索类型，固定为 `"text"`。
    pub search_type: &'static str,
    /// 搜索范围：1=仅标题，2=仅正文，3=标题与正文。
    pub search_scope: i32,
    /// 逻辑关系，1=AND。
    pub search_operator: i32,
    /// 排序方式，固定为 `"date"`。
    pub order_type: &'static str,
    /// 全站检索，固定为 `true`。
    pub allsite: bool,
    /// 站点 Owner ID 列表（逗号分隔）。
    pub owner: &'a str,
    /// 栏目 ID 列表（逗号分隔）。
    pub column_id: &'a str,
    /// 审核状态，固定为 `["1"]`。
    pub auditing: &'static [&'static str],
    /// 日期检索类型，使用自定义日期时为 `"custom"`。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_date_type: Option<&'static str>,
    /// 起始时间（`YYYY-MM-DD HH:mm:ss`）。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub begin_date: Option<&'a str>,
    /// 截止时间（`YYYY-MM-DD HH:mm:ss`）。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_date: Option<&'a str>,
    /// 分页参数。
    pub page: VsbPageParam,
}

/// 分页查询参数。
#[derive(Debug, Clone, Copy, Serialize)]
pub struct VsbPageParam {
    /// 当前页码（从 0 开始）。
    pub current: u32,
    /// 每页大小。
    pub size: u32,
}

/// VSB 接口顶层响应包。
#[derive(Debug, Clone, Deserialize)]
pub struct VsbSearchResponse {
    /// 状态码，成功为 `"0000"`。
    pub code: String,
    /// 提示信息。
    pub message: Option<String>,
    /// 数据主体。
    pub data: Option<VsbData>,
}

/// 响应数据主体。
#[derive(Debug, Clone, Deserialize)]
pub struct VsbData {
    /// 分页数据。
    pub page: Option<VsbPageResult>,
}

/// 分页结果。
#[derive(Debug, Clone, Deserialize)]
pub struct VsbPageResult {
    /// 当前页码。
    pub current: u32,
    /// 每页大小。
    pub size: u32,
    /// 总记录数。
    pub total: u64,
    /// 文章记录列表。
    pub records: Option<Vec<VsbArticleRecord>>,
}

/// 单条原始文章记录。
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VsbArticleRecord {
    /// 索引名称，如 `"article_data"`。
    pub index: Option<String>,
    /// 文章 ID。
    pub id: Option<serde_json::Value>,
    /// 站点 Owner ID。
    pub owner: Option<serde_json::Value>,
    /// 栏目 ID。
    pub column: Option<serde_json::Value>,
    /// 栏目名称。
    pub column_name: Option<String>,
    /// 文章标题（可能含 HTML 高亮标签）。
    pub title: Option<String>,
    /// 文章原始 URL（可能为相对路径）。
    pub url: Option<String>,
    /// 创建/发布时间（`YYYY-MM-DD HH:mm:ss`）。
    pub create_date: Option<String>,
    /// 简介摘要。
    pub intro: Option<String>,
    /// 摘要备用字段。
    pub summary: Option<String>,
    /// 正文备用字段。
    pub content: Option<String>,
}

/// 清洗和规范化后的文章数据模型，供 MCP 工具输出。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CleanedArticle {
    /// 纯文本标题。
    pub title: String,
    /// 绝对可访问 URL。
    pub url: String,
    /// 所属部门名称。
    pub department: String,
    /// 所属栏目名称。
    pub column: String,
    /// 发布日期（`YYYY-MM-DD`）。
    pub date: String,
    /// 内容摘要片段（已清洗 HTML 并截断）。
    pub snippet: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_response() {
        let json = r#"{
            "code": "0000",
            "message": "success",
            "data": {
                "page": {
                    "current": 0,
                    "size": 50,
                    "total": 1,
                    "records": [
                        {
                            "index": "article_data",
                            "id": 1001,
                            "owner": 1864855249,
                            "column": 1051,
                            "columnName": "通知公告",
                            "title": "关于测试的<span style=\"color:red\">通知</span>",
                            "url": "/info/1051/1001.htm",
                            "createDate": "2026-10-25 15:30:00",
                            "intro": "正文摘要测试 &amp; 内容"
                        }
                    ]
                }
            }
        }"#;

        let resp: Result<VsbSearchResponse, _> = serde_json::from_str(json);
        assert!(resp.is_ok());
        if let Ok(res) = resp {
            assert_eq!(res.code, "0000");
            let data = res.data.unwrap_or(VsbData { page: None });
            let page = data.page.unwrap_or(VsbPageResult {
                current: 0,
                size: 0,
                total: 0,
                records: None,
            });
            assert_eq!(page.total, 1);
            let records = page.records.unwrap_or_default();
            assert_eq!(records.len(), 1);
            assert_eq!(records[0].column_name.as_deref(), Some("通知公告"));
        }
    }
}
