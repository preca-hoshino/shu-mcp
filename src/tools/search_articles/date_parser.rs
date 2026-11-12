//! 日期范围解析器。
//!
//! 提供单参数日期区间的容错解析功能，支持单日、闭区间、单侧区间以及多种分隔符，
//! 具备日期反转自动纠偏和非法输入降级容错机制。

/// 解析后的 VSB 日期过滤参数。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedDateRange {
    /// 搜索起始时间（格式：`YYYY-MM-DD HH:mm:ss`）。
    pub begin_date: Option<String>,
    /// 搜索截止时间（格式：`YYYY-MM-DD HH:mm:ss`）。
    pub end_date: Option<String>,
}

/// 解析用户或模型输入的日期字符串。
///
/// # 支持格式
/// - 单日：`"2026-10-25"` -> `2026-10-25 00:00:00` 至 `2026-10-25 23:59:59`
/// - 闭区间：`"2026-01-01..2026-06-30"`（支持 `..`、`~`、`/`、`,` 分隔）
/// - 单侧区间：`"2026-09-01.."`、`"..2026-10-31"`
///
/// # 容错机制
/// - 若起始日期晚于截止日期，自动对调纠正。
/// - 若输入格式完全无法识别，返回 `None`（外层可退化为全时间检索，不中断交互）。
#[must_use]
pub fn parse_date_range(input: &str) -> Option<ParsedDateRange> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None;
    }

    // 检查常见区间分隔符：优先匹配 ".."，其次是 "~", "/", ","
    if let Some((left, right)) = split_range(trimmed) {
        let left_date = extract_single_date(left);
        let right_date = extract_single_date(right);

        match (left_date, right_date) {
            (Some(mut start), Some(mut end)) => {
                // 若日期颠倒，自动对调
                if start > end {
                    std::mem::swap(&mut start, &mut end);
                }
                Some(ParsedDateRange {
                    begin_date: Some(format!("{start} 00:00:00")),
                    end_date: Some(format!("{end} 23:59:59")),
                })
            }
            (Some(start), None) => Some(ParsedDateRange {
                begin_date: Some(format!("{start} 00:00:00")),
                end_date: None,
            }),
            (None, Some(end)) => Some(ParsedDateRange {
                begin_date: None,
                end_date: Some(format!("{end} 23:59:59")),
            }),
            (None, None) => None,
        }
    } else {
        extract_single_date(trimmed).map(|single| ParsedDateRange {
            begin_date: Some(format!("{single} 00:00:00")),
            end_date: Some(format!("{single} 23:59:59")),
        })
    }
}

/// 尝试将输入拆分为双端区间。
fn split_range(input: &str) -> Option<(&str, &str)> {
    if let Some(idx) = input.find("..") {
        let (left, right_with_dots) = input.split_at(idx);
        return Some((left.trim(), right_with_dots[2..].trim()));
    }
    if let Some(idx) = input.find('~') {
        let (left, right_with_char) = input.split_at(idx);
        return Some((left.trim(), right_with_char[1..].trim()));
    }
    if let Some(idx) = input.find('/') {
        let (left, right_with_char) = input.split_at(idx);
        return Some((left.trim(), right_with_char[1..].trim()));
    }
    if let Some(idx) = input.find(',') {
        let (left, right_with_char) = input.split_at(idx);
        return Some((left.trim(), right_with_char[1..].trim()));
    }
    None
}

/// 尝试从字符串中提取合法的 `YYYY-MM-DD`。
fn extract_single_date(input: &str) -> Option<String> {
    let s = input.trim();
    if s.is_empty() {
        return None;
    }

    // 简单高效提取 YYYY-MM-DD
    if s.len() == 10 {
        let b = s.as_bytes();
        if b[4] == b'-'
            && b[7] == b'-'
            && b[0..4].iter().all(u8::is_ascii_digit)
            && b[5..7].iter().all(u8::is_ascii_digit)
            && b[8..10].iter().all(u8::is_ascii_digit)
        {
            return Some(s.to_string());
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_date() {
        let range = parse_date_range("2026-10-25");
        assert_eq!(
            range,
            Some(ParsedDateRange {
                begin_date: Some("2026-10-25 00:00:00".to_string()),
                end_date: Some("2026-10-25 23:59:59".to_string()),
            })
        );
    }

    #[test]
    fn test_double_ended_range() {
        let range = parse_date_range("2026-01-01..2026-06-30");
        assert_eq!(
            range,
            Some(ParsedDateRange {
                begin_date: Some("2026-01-01 00:00:00".to_string()),
                end_date: Some("2026-06-30 23:59:59".to_string()),
            })
        );
    }

    #[test]
    fn test_tilde_delimiter() {
        let range = parse_date_range("2026-01-01 ~ 2026-06-30");
        assert_eq!(
            range,
            Some(ParsedDateRange {
                begin_date: Some("2026-01-01 00:00:00".to_string()),
                end_date: Some("2026-06-30 23:59:59".to_string()),
            })
        );
    }

    #[test]
    fn test_reversed_dates_auto_corrected() {
        let range = parse_date_range("2026-12-31..2026-01-01");
        assert_eq!(
            range,
            Some(ParsedDateRange {
                begin_date: Some("2026-01-01 00:00:00".to_string()),
                end_date: Some("2026-12-31 23:59:59".to_string()),
            })
        );
    }

    #[test]
    fn test_start_only() {
        let range = parse_date_range("2026-09-01..");
        assert_eq!(
            range,
            Some(ParsedDateRange {
                begin_date: Some("2026-09-01 00:00:00".to_string()),
                end_date: None,
            })
        );
    }

    #[test]
    fn test_end_only() {
        let range = parse_date_range("..2026-10-31");
        assert_eq!(
            range,
            Some(ParsedDateRange {
                begin_date: None,
                end_date: Some("2026-10-31 23:59:59".to_string()),
            })
        );
    }

    #[test]
    fn test_invalid_input() {
        assert_eq!(parse_date_range(""), None);
        assert_eq!(parse_date_range("abc"), None);
        assert_eq!(parse_date_range(".."), None);
    }
}
