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

| 工具名称 | 功能描述 | 参数说明 |
|---|---|---|
| `search_articles` | 实时搜索上海大学全校各院系/部门的通知公告与新闻资讯 | • `query` (string, 必填)：搜索关键词，支持多词空格分隔<br>• `scope` (string, 可选)：范围 (`"title"`/`"body"`/`"all"`，默认 `"title"`)<br>• `date_range` (string, 可选)：单日 `YYYY-MM-DD` 或区间 `..`、`~`<br>• `limit` (integer, 可选)：返回数量 (1-50，默认 15)<br>• `page` (integer, 可选)：页码 (从 1 开始) |


