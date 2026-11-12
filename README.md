# SHU MCP Server

上海大学 MCP (Model Context Protocol) 服务 —— 为 AI 助手提供上海大学全校各院系/部门通知公告与新闻资讯的实时检索能力。

## 数据源与服务特点

本服务直连上海大学官方集约化 VSB 网站群搜索系统（`/aop_component/webber/search/search/search/queryPage`）：
- **实时响应**：毫秒级全网实时检索，无需等待离线爬虫产出；
- **全校覆盖**：内置全校 71 个院系与部门的站点与栏目映射，全校范围深度检索；
- **智能过滤**：支持标题/正文搜索范围、单参数灵活日期区间过滤及紧凑分页；
- **零额外存储**：纯内存直连无状态服务，启动极速。

## 快速开始

```bash
# 构建发布版本
cargo build --release

# 运行
cargo run
```
