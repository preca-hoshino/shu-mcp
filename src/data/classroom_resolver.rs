//! 上海大学教学楼白名单、重命名与别名解析器。

/// 教学楼静态元数据定义。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BuildingDef {
    /// 教学楼 ID。
    pub id: i32,
    /// 官方后台原始名称。
    pub raw_name: &'static str,
    /// 归一化规范显示名称。
    pub name: &'static str,
    /// 校区名称（宝山、延长、嘉定）。
    pub campus: &'static str,
    /// 常见别名库（全小写，用于模糊匹配）。
    pub aliases: &'static [&'static str],
}

/// 全校 22 栋有效可查询教学楼定义（已剔除 884 计算机大楼）。
pub const SELECTABLE_BUILDINGS: [BuildingDef; 22] = [
    // 宝山校区 - 主楼与阶梯楼对
    BuildingDef {
        id: 303,
        raw_name: "A楼",
        name: "A楼",
        campus: "宝山",
        aliases: &["a", "a楼", "a座"],
    },
    BuildingDef {
        id: 325,
        raw_name: "AJ楼",
        name: "AJ楼",
        campus: "宝山",
        aliases: &["aj", "aj楼", "aj座"],
    },
    BuildingDef {
        id: 122,
        raw_name: "B楼",
        name: "B楼",
        campus: "宝山",
        aliases: &["b", "b楼", "b座"],
    },
    BuildingDef {
        id: 328,
        raw_name: "BJ楼",
        name: "BJ楼",
        campus: "宝山",
        aliases: &["bj", "bj楼", "bj座"],
    },
    BuildingDef {
        id: 528,
        raw_name: "C楼",
        name: "C楼",
        campus: "宝山",
        aliases: &["c", "c楼", "c座"],
    },
    BuildingDef {
        id: 262,
        raw_name: "CJ楼",
        name: "CJ楼",
        campus: "宝山",
        aliases: &["cj", "cj楼", "cj座"],
    },
    BuildingDef {
        id: 164,
        raw_name: "D楼",
        name: "D楼",
        campus: "宝山",
        aliases: &["d", "d楼", "d座"],
    },
    BuildingDef {
        id: 208,
        raw_name: "DJ楼",
        name: "DJ楼",
        campus: "宝山",
        aliases: &["dj", "dj楼", "dj座"],
    },
    BuildingDef {
        id: 930,
        raw_name: "GA楼",
        name: "GA楼",
        campus: "宝山",
        aliases: &["ga", "ga楼", "ga座"],
    },
    BuildingDef {
        id: 222,
        raw_name: "GJ楼",
        name: "GJ楼",
        campus: "宝山",
        aliases: &["gj", "gj楼", "gj座"],
    },
    BuildingDef {
        id: 220,
        raw_name: "EJ楼",
        name: "EJ楼",
        campus: "宝山",
        aliases: &["ej", "ej楼", "ej座"],
    },
    BuildingDef {
        id: 221,
        raw_name: "FJ楼",
        name: "FJ楼",
        campus: "宝山",
        aliases: &["fj", "fj楼", "fj座"],
    },
    BuildingDef {
        id: 700,
        raw_name: "J楼",
        name: "J楼",
        campus: "宝山",
        aliases: &["j", "j楼", "j座"],
    },
    // 宝山校区 - 东区楼宇（归并入宝山，重命名）
    BuildingDef {
        id: 476,
        raw_name: "商管大楼",
        name: "经管楼",
        campus: "宝山",
        aliases: &["经管", "经管楼", "经管大楼", "商管", "商管楼", "商管大楼", "东区经管"],
    },
    BuildingDef {
        id: 478,
        raw_name: "信息大楼",
        name: "计算机/通信楼",
        campus: "宝山",
        aliases: &[
            "计算机",
            "计算机楼",
            "通信",
            "通信楼",
            "计通",
            "计通楼",
            "计算机/通信楼",
            "计算机通信楼",
            "信息大楼",
            "信息楼",
            "东区信息",
            "东区计算机",
        ],
    },
    BuildingDef {
        id: 477,
        raw_name: "材料大楼",
        name: "材料楼",
        campus: "宝山",
        aliases: &["材料", "材料楼", "材料大楼", "东区材料"],
    },
    // 延长校区
    BuildingDef {
        id: 399,
        raw_name: "行健楼",
        name: "行健楼",
        campus: "延长",
        aliases: &["行健", "行健楼", "延长行健"],
    },
    BuildingDef {
        id: 422,
        raw_name: "第四教学楼",
        name: "第四教学楼",
        campus: "延长",
        aliases: &["四教", "第四教学楼", "第四教", "四号楼", "延长四教"],
    },
    BuildingDef {
        id: 449,
        raw_name: "仪表楼",
        name: "仪表楼",
        campus: "延长",
        aliases: &["仪表", "仪表楼", "延长仪表"],
    },
    // 嘉定校区
    BuildingDef {
        id: 607,
        raw_name: "第一教学楼",
        name: "第一教学楼",
        campus: "嘉定",
        aliases: &["一教", "第一教学楼", "第一教", "一号楼", "嘉定一教"],
    },
    BuildingDef {
        id: 575,
        raw_name: "第二教学楼",
        name: "第二教学楼",
        campus: "嘉定",
        aliases: &["二教", "第二教学楼", "第二教", "二号楼", "嘉定二教"],
    },
    BuildingDef {
        id: 759,
        raw_name: "文达楼",
        name: "文达楼",
        campus: "嘉定",
        aliases: &["文达", "文达楼", "嘉定文达"],
    },
];

/// 根据单一字符串名称尝试解析为某一栋已知的教学楼。
#[must_use]
pub fn resolve_single_building(query: &str) -> Option<&'static BuildingDef> {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return None;
    }
    let lower = trimmed.to_lowercase();

    // 1. 尝试按标准规范名或后台原始名精确匹配
    for b in &SELECTABLE_BUILDINGS {
        if b.name.eq_ignore_ascii_case(trimmed) || b.raw_name.eq_ignore_ascii_case(trimmed) {
            return Some(b);
        }
    }

    // 2. 尝试按别名列表精确匹配
    for b in &SELECTABLE_BUILDINGS {
        if b.aliases.iter().any(|alias| *alias == lower) {
            return Some(b);
        }
    }

    // 3. 尝试去除前后“上海大学”、“宝山”、“延长”、“嘉定”、“校区”等前缀后匹配
    let clean = lower
        .replace("上海大学", "")
        .replace("宝山校区", "")
        .replace("延长校区", "")
        .replace("嘉定校区", "")
        .replace("宝山", "")
        .replace("延长", "")
        .replace("嘉定", "")
        .replace("校区", "")
        .trim()
        .to_string();

    if !clean.is_empty() {
        for b in &SELECTABLE_BUILDINGS {
            if b.name.eq_ignore_ascii_case(&clean)
                || b.raw_name.eq_ignore_ascii_case(&clean)
                || b.aliases.iter().any(|alias| *alias == clean)
            {
                return Some(b);
            }
        }
    }

    None
}

/// 解析用户或模型输入的教学楼参数，支持并排多楼查询（以逗号、顿号、分号或空格分隔）。
///
/// 保持去重且保序输出。
#[must_use]
pub fn resolve_multiple_buildings(input: &str) -> Vec<&'static BuildingDef> {
    let mut results = Vec::new();

    // 按常见分隔符切分（逗号、顿号、分号、换行、空格）
    let parts = input.split([',', '，', '、', ';', '；', '\n', '\t', ' ']);

    for part in parts {
        let trimmed = part.trim();
        if trimmed.is_empty() {
            continue;
        }

        if let Some(building) = resolve_single_building(trimmed)
            && !results.iter().any(|b: &&'static BuildingDef| b.id == building.id)
        {
            results.push(building);
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_standard_names() {
        assert_eq!(resolve_single_building("A楼").map(|b| b.id), Some(303));
        assert_eq!(resolve_single_building("AJ楼").map(|b| b.id), Some(325));
        assert_eq!(resolve_single_building("行健楼").map(|b| b.id), Some(399));
        assert_eq!(resolve_single_building("第一教学楼").map(|b| b.id), Some(607));
    }

    #[test]
    fn test_resolve_renames_and_aliases() {
        // 商管大楼 -> 经管楼
        assert_eq!(resolve_single_building("经管楼").map(|b| b.id), Some(476));
        assert_eq!(resolve_single_building("商管大楼").map(|b| b.id), Some(476));
        assert_eq!(resolve_single_building("经管").map(|b| b.id), Some(476));

        // 信息大楼 -> 计算机/通信楼
        assert_eq!(resolve_single_building("信息大楼").map(|b| b.id), Some(478));
        assert_eq!(resolve_single_building("计算机楼").map(|b| b.id), Some(478));
        assert_eq!(resolve_single_building("计算机/通信楼").map(|b| b.id), Some(478));

        // 简写与别名
        assert_eq!(resolve_single_building("a").map(|b| b.id), Some(303));
        assert_eq!(resolve_single_building("aj").map(|b| b.id), Some(325));
        assert_eq!(resolve_single_building("一教").map(|b| b.id), Some(607));
        assert_eq!(resolve_single_building("四教").map(|b| b.id), Some(422));
    }

    #[test]
    fn test_resolve_multiple_buildings_split() {
        let res = resolve_multiple_buildings("A楼, AJ楼、经管楼; 行健楼 第一教学楼");
        let ids: Vec<i32> = res.iter().map(|b| b.id).collect();
        assert_eq!(ids, vec![303, 325, 476, 399, 607]);
    }

    #[test]
    fn test_deduplication() {
        let res = resolve_multiple_buildings("A楼, a, A座, AJ");
        let ids: Vec<i32> = res.iter().map(|b| b.id).collect();
        assert_eq!(ids, vec![303, 325]);
    }
}
