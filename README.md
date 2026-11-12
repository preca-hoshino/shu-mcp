# SHU MCP Server

上海大学 MCP (Model Context Protocol) 服务 —— 为 AI 助手提供上海大学全校各院系/部门通知公告与新闻资讯的实时检索能力。

## 接入方式

基于标准输入输出 (`stdio`) 通道通信（JSON-RPC 2.0）。可直接从 GitHub Releases 下载对应平台的单文件可执行二进制（或使用 `cargo run` 源码运行）。

### 客户端配置示例

以 Claude Desktop / Cursor / VS Code 的 MCP 配置为例：

```json
{
  "mcpServers": {
    "shu-mcp": {
      "command": "C:\\path\\to\\shu-mcp-windows-x64.exe"
    }
  }
}
```

## 提供的工具

| 工具名称 | 功能描述 | 参数列表 |
|---|---|---|
| `search_articles` | 实时搜索上海大学全校各院系/部门的通知公告与新闻资讯 | `query`, `scope`, `date_range`, `limit`, `page` |
| `get_article_detail` | 获取上海大学官方通知公告/新闻的具体正文内容、发布元数据与附件列表 | `url` |
| `query_empty_classrooms` | 查询上海大学指定教学楼在指定日期的空闲自习教室（支持多楼并排、指定节次或全天） | `building`, `date`, `sections`, `floor` |
| `get_classroom_schedule` | 查询指定教室全天排课日历及空闲时段，或按关键词反查课程地点（支持多楼并排） | `building`, `room`, `query`, `date` |

## 许可证

[AGPL 3.0](./LICENSE)