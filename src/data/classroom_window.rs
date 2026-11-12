//! 上海大学空教室外网服务开放时间守卫。

use chrono::{Datelike, Timelike, Utc};

/// 外网服务开放起始小时（北京时间 07:00）。
const OPEN_HOUR: u32 = 7;

/// 外网服务开放结束小时（北京时间 23:00）。
const CLOSE_HOUR: u32 = 23;

/// 禁用时段提示错误文案（严格无 Emoji）。
pub const NIGHT_WINDOW_ERROR_MESSAGE: &str =
    "[错误] 当前处于上海大学空教室系统外网关闭时段（每日 23:00 至次日 07:00）。\n\
     外网服务暂未开放，系统已自动拦截请求。请在开放时段（07:00 - 23:00）重试。";

/// 获取当前北京时间（东八区 UTC+8）的年、月、日、小时。
#[must_use]
pub fn current_beijing_time() -> (i32, u32, u32, u32) {
    let utc_now = Utc::now();
    // UTC 转换为东八区（+8 小时）
    let beijing_naive = utc_now.naive_utc() + chrono::Duration::hours(8);
    (
        beijing_naive.year(),
        beijing_naive.month(),
        beijing_naive.day(),
        beijing_naive.hour(),
    )
}

/// 检查当前是否处于外网开放时段（每日 07:00 至 23:00）。
///
/// 若处于夜间 23:00 至次日 07:00，返回 `false`。
/// 支持环境变量 `SHU_IGNORE_CLASSROOM_WINDOW=1` 旁路时间守卫用于开发或内部网络测试。
#[must_use]
pub fn is_external_access_open() -> bool {
    if std::env::var("SHU_IGNORE_CLASSROOM_WINDOW").is_ok_and(|v| v == "1") {
        return true;
    }
    let (_, _, _, hour) = current_beijing_time();
    (OPEN_HOUR..CLOSE_HOUR).contains(&hour)
}

/// 获取默认查询日期（YYYY-MM-DD）。
///
/// 若当前处于夜间 21:40（最后一节下课）之后，自动顺延到次日。
#[must_use]
pub fn default_query_date() -> String {
    let utc_now = Utc::now();
    let beijing_naive = utc_now.naive_utc() + chrono::Duration::hours(8);
    let hour = beijing_naive.hour();
    let minute = beijing_naive.minute();

    // 21:40 之后顺延至次日
    let target = if hour > 21 || (hour == 21 && minute >= 40) {
        beijing_naive + chrono::Duration::days(1)
    } else {
        beijing_naive
    };

    format!("{:04}-{:02}-{:02}", target.year(), target.month(), target.day())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_current_beijing_time_format() {
        let (y, m, d, h) = current_beijing_time();
        assert!(y >= 2026);
        assert!((1..=12).contains(&m));
        assert!((1..=31).contains(&d));
        assert!(h < 24);
    }

    #[test]
    fn test_default_query_date_format() {
        let date = default_query_date();
        assert_eq!(date.len(), 10);
        let parts: Vec<&str> = date.split('-').collect();
        assert_eq!(parts.len(), 3);
        assert_eq!(parts[0].len(), 4);
        assert_eq!(parts[1].len(), 2);
        assert_eq!(parts[2].len(), 2);
    }
}
