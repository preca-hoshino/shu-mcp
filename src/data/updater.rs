//! 后台定时拉取远程仓库最新 JSON 数据。
//!
//! 通过 [`super::loader::refresh_from_remote`] 协调数据获取与索引更新。
//! 启动时立即执行一次拉取，之后每 [`REFRESH_INTERVAL_SECS`] 秒重复执行。
//! 整个过程在后台任务中运行，不阻塞 MCP 服务器启动或请求处理。

/// 拉取间隔（秒）。
const REFRESH_INTERVAL_SECS: u64 = 3600;

/// 启动后台定时拉取任务。
///
/// 首次立即执行一次拉取，之后每 [`REFRESH_INTERVAL_SECS`] 秒重复执行。
/// 拉取失败时保留旧索引并打印错误，不中断任务循环。
pub fn spawn_updater() {
    tokio::spawn(async move {
        // 首次拉取
        if let Err(e) = super::loader::refresh_from_remote().await {
            eprintln!("⚠ 首次远程数据拉取失败: {e}");
        }
        // 定时循环
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(REFRESH_INTERVAL_SECS)).await;
            if let Err(e) = super::loader::refresh_from_remote().await {
                eprintln!("⚠ 远程数据拉取失败: {e}");
            }
        }
    });
}
