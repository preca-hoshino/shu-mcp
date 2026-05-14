# SHU MCP Server

上海大学 MCP (Model Context Protocol) 服务 —— 为 AI 助手提供上海大学各院系/部门新闻资讯的实时查询能力。

## 数据源

本服务运行时从 [shu-mcp-data](https://github.com/arts-amadeus/shu-mcp-data) 仓库动态读取由定时爬虫产出的结构化 JSON 数据。

## 快速开始

```bash
# 构建
cargo build --release

# 运行
cargo run
```
