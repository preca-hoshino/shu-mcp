//! 上海大学各部门/院系通知公告数据源注册表。
//!
//! 包含全校 71 个支持 VSB 网站群集约化搜索的站点元数据，
//! 并提供预计算的全校全局 Owner ID 与 Column ID 常量。

use std::collections::HashMap;
use std::sync::LazyLock;

/// 单个部门/院系数据源元数据。
#[derive(Debug, Clone)]
pub struct AnnouncementSource {
    /// 部门唯一标识。
    pub id: &'static str,
    /// 部门名称。
    pub name: &'static str,
    /// VSB 站点 Owner ID。
    pub owner: &'static str,
    /// 对应的栏目 ID 列表。
    pub column_ids: &'static [&'static str],
    /// 公告列表页基准 URL，用于将相对路径补全为绝对路径。
    pub list_url: &'static str,
}

/// 全校所有支持搜索的数据源列表（共 71 个）。
pub static SOURCES: &[AnnouncementSource] = &[
    AnnouncementSource {
        id: "shu",
        name: "上海大学官网",
        owner: "1864855249",
        column_ids: &["1051"],
        list_url: "https://www.shu.edu.cn/tzgg.htm",
    },
    AnnouncementSource {
        id: "bksy",
        name: "本科生院",
        owner: "1662210750",
        column_ids: &["1015"],
        list_url: "https://bksy.shu.edu.cn/index/tzgg.htm",
    },
    AnnouncementSource {
        id: "bkzsw",
        name: "本科生招生网",
        owner: "1364902694",
        column_ids: &[
            "1798", "1810", "7743", "7563", "7373", "7161", "2175", "1812", "1826", "7753", "7573",
            "7383", "7171", "2168", "1839", "7763", "7583", "7393", "7201", "2176", "1846", "7773",
            "7593", "7403", "7211", "2169", "1878", "7603", "7413", "7262", "2171", "7783", "1880",
            "1928", "7823", "7643", "7453", "7251", "2174", "1893", "7793", "7613", "7423", "7272",
            "2172", "1906", "7623", "7433", "7241", "2177", "7803", "1912", "1915", "7813", "7633",
            "7443", "7151", "2173",
        ],
        list_url: "https://bkzsw.shu.edu.cn/zxgg.htm",
    },
    AnnouncementSource {
        id: "yjszs",
        name: "研究生招生网",
        owner: "1462468299",
        column_ids: &["1003", "1004"],
        list_url: "https://yjszs.shu.edu.cn/cd/tzgg.htm",
    },
    AnnouncementSource {
        id: "yingxin",
        name: "迎新网",
        owner: "1565422366",
        column_ids: &["1008", "1007"],
        list_url: "https://yingxin.shu.edu.cn/tzgg.htm",
    },
    AnnouncementSource {
        id: "xgb",
        name: "本科生工作处",
        owner: "1495423387",
        column_ids: &["1003"],
        list_url: "https://xgb.shu.edu.cn/xgdt/tzgg.htm",
    },
    AnnouncementSource {
        id: "dwygb",
        name: "研究生工作处",
        owner: "1597130582",
        column_ids: &["1050"],
        list_url: "https://dwygb.shu.edu.cn/xwzx/tzgg.htm",
    },
    AnnouncementSource {
        id: "gs",
        name: "研究生院",
        owner: "1880701254",
        column_ids: &["1029", "1027", "1030", "1031"],
        list_url: "https://gs.shu.edu.cn/xwlb/py.htm",
    },
    AnnouncementSource {
        id: "cwc",
        name: "财务处",
        owner: "1393148177",
        column_ids: &["1015", "1041"],
        list_url: "https://cwc.shu.edu.cn/index/tzgg.htm",
    },
    AnnouncementSource {
        id: "lib",
        name: "图书馆",
        owner: "1705785581",
        column_ids: &["1022", "1023"],
        list_url: "https://lib.shu.edu.cn/sho/ggxx.htm",
    },
    AnnouncementSource {
        id: "alumni",
        name: "对外联络处",
        owner: "1551701726",
        column_ids: &["1021"],
        list_url: "https://alumni.shu.edu.cn/xwggzx/tzgg.htm",
    },
    AnnouncementSource {
        id: "shuedf",
        name: "上海大学教育发展基金会",
        owner: "1812157458",
        column_ids: &["1005"],
        list_url: "https://shuedf.shu.edu.cn/sylm/tzgg.htm",
    },
    AnnouncementSource {
        id: "kyglb",
        name: "科研管理部",
        owner: "1425312184",
        column_ids: &["1011"],
        list_url: "https://kyglb.shu.edu.cn/dhl/tzgg.htm",
    },
    AnnouncementSource {
        id: "baoweichu",
        name: "保卫处",
        owner: "1623616573",
        column_ids: &["1061"],
        list_url: "https://baoweichu.shu.edu.cn/ggl.htm",
    },
    AnnouncementSource {
        id: "its",
        name: "信息化工作办公室",
        owner: "1663284523",
        column_ids: &["1095"],
        list_url: "https://newits.shu.edu.cn/index/tzgg.htm",
    },
    AnnouncementSource {
        id: "youth",
        name: "上海大学共青团委员会",
        owner: "1659861027",
        column_ids: &["1010"],
        list_url: "https://youth.shu.edu.cn/syxzlly/tzgg.htm",
    },
    AnnouncementSource {
        id: "global",
        name: "国际部",
        owner: "1479802255",
        column_ids: &["1003"],
        list_url: "https://global.shu.edu.cn/cd/tzgg.htm",
    },
    AnnouncementSource {
        id: "hbb",
        name: "后勤保障部",
        owner: "1675169118",
        column_ids: &["1005"],
        list_url: "https://hbb.shu.edu.cn/tzgg.htm",
    },
    AnnouncementSource {
        id: "xyy",
        name: "校医院",
        owner: "1678802850",
        column_ids: &["1090"],
        list_url: "https://xyy.shu.edu.cn/sylm/ggl.htm",
    },
    AnnouncementSource {
        id: "archive",
        name: "档案馆",
        owner: "1663770242",
        column_ids: &["1013"],
        list_url: "https://dangan.shu.edu.cn/gqgk/xxfb/fwgg.htm",
    },
    AnnouncementSource {
        id: "sbc",
        name: "实验室与设备管理处",
        owner: "1530700580",
        column_ids: &["1004"],
        list_url: "https://sbc.shu.edu.cn/tg/tzgg.htm",
    },
    AnnouncementSource {
        id: "hr",
        name: "人事处",
        owner: "1598626614",
        column_ids: &["1029"],
        list_url: "https://hr.shu.edu.cn/xwgg1/xwgg.htm",
    },
    AnnouncementSource {
        id: "cie",
        name: "国际教育学院",
        owner: "1504253137",
        column_ids: &["1014"],
        list_url: "https://cie.shu.edu.cn/tg/xytz.htm",
    },
    AnnouncementSource {
        id: "cla",
        name: "文学院",
        owner: "1500565633",
        column_ids: &["1017", "1015", "1019", "1020"],
        list_url: "https://cla.shu.edu.cn/tg/tzgg.htm",
    },
    AnnouncementSource {
        id: "ece",
        name: "环境与化学工程学院",
        owner: "1880699954",
        column_ids: &["1303", "1304", "1147", "1148"],
        list_url: "https://ece.shu.edu.cn/sylm/bksjx.htm",
    },
    AnnouncementSource {
        id: "mat",
        name: "材料科学与工程学院",
        owner: "1449189309",
        column_ids: &["1018", "1019", "1013"],
        list_url: "https://mat.shu.edu.cn/sycdlm/tzgg/bks.htm",
    },
    AnnouncementSource {
        id: "mba",
        name: "MBA教育管理中心",
        owner: "1797966698",
        column_ids: &["1024"],
        list_url: "https://mba.shu.edu.cn/cslm/tzgg.htm",
    },
    AnnouncementSource {
        id: "mgi",
        name: "材料基因组工程研究院",
        owner: "1650566287",
        column_ids: &["1062"],
        list_url: "https://mgi.shu.edu.cn/index/lmzl/tzgg.htm",
    },
    AnnouncementSource {
        id: "suitm",
        name: "转化医学研究院",
        owner: "1771177353",
        column_ids: &["1082", "1085", "1086"],
        list_url: "https://suitm.shu.edu.cn/syzl/xwdt.htm",
    },
    AnnouncementSource {
        id: "adri",
        name: "亚洲人口研究中心",
        owner: "1691313259",
        column_ids: &["1057"],
        list_url: "https://adrichn.shu.edu.cn/sylmnr/ggl.htm",
    },
    AnnouncementSource {
        id: "cast",
        name: "先进凝固技术中心",
        owner: "1676522650",
        column_ids: &["1042", "1043", "1044"],
        list_url: "https://cast.shu.edu.cn/sylm/xwkd.htm",
    },
    AnnouncementSource {
        id: "icn",
        name: "碳中和创新研究中心",
        owner: "2052219376",
        column_ids: &["1531", "1541"],
        list_url: "https://icn.shu.edu.cn/index/kydt.htm",
    },
    AnnouncementSource {
        id: "igs",
        name: "全球问题研究院",
        owner: "1471238366",
        column_ids: &["1011", "1012"],
        list_url: "https://igs.shu.edu.cn/cd/tzgg.htm",
    },
    AnnouncementSource {
        id: "ihe",
        name: "高等教育研究所",
        owner: "2063254418",
        column_ids: &["1021"],
        list_url: "https://ihe.shu.edu.cn/sybk/gjdt.htm",
    },
    AnnouncementSource {
        id: "jgzx",
        name: "上海经济管理中心",
        owner: "1649809844",
        column_ids: &["1017", "1019"],
        list_url: "https://jgzx.shu.edu.cn/synr/ywsd.htm",
    },
    AnnouncementSource {
        id: "jhai",
        name: "无人艇工程研究院",
        owner: "1730694740",
        column_ids: &["1042"],
        list_url: "https://jhai.shu.edu.cn/sylm/xwdt.htm",
    },
    AnnouncementSource {
        id: "museum",
        name: "博物馆",
        owner: "1880701068",
        column_ids: &["1030"],
        list_url: "https://museum.shu.edu.cn/bggk/xwkd.htm",
    },
    AnnouncementSource {
        id: "suns",
        name: "数据科学与都市研究中心",
        owner: "1678865198",
        column_ids: &["1020", "1019"],
        list_url: "https://suns.shu.edu.cn/sy/gg.htm",
    },
    AnnouncementSource {
        id: "skl4nm",
        name: "核电关键材料全国重点实验室",
        owner: "2132341510",
        column_ids: &["1031", "1071", "1051", "1061"],
        list_url: "https://skl4nm.shu.edu.cn/sybk/tzgg.htm",
    },
    AnnouncementSource {
        id: "sklar",
        name: "先进耐火材料全国实验室",
        owner: "2132008928",
        column_ids: &["1502"],
        list_url: "https://sklar.shu.edu.cn/sybk/tzgg.htm",
    },
    AnnouncementSource {
        id: "sklass",
        name: "先进特殊钢全国重点实验室",
        owner: "1339981612",
        column_ids: &["1044"],
        list_url: "https://sklass.shu.edu.cn/index/gsgg.htm",
    },
    AnnouncementSource {
        id: "fpd",
        name: "新型显示技术及应用集成教育部重点实验室",
        owner: "1383592681",
        column_ids: &["1111", "1113", "1112"],
        list_url: "https://fpd.shu.edu.cn/kfyjl/tzgg.htm",
    },
    AnnouncementSource {
        id: "ms",
        name: "管理学院",
        owner: "1657519516",
        column_ids: &["1245"],
        list_url: "https://ms.shu.edu.cn/syzl/zytz.htm",
    },
    AnnouncementSource {
        id: "qwc",
        name: "钱伟长学院",
        owner: "1600705936",
        column_ids: &["1086"],
        list_url: "https://qwc.shu.edu.cn/tzgg.htm",
    },
    AnnouncementSource {
        id: "scicol",
        name: "理学院",
        owner: "1381981180",
        column_ids: &["1006", "1009", "1010"],
        list_url: "https://scicol.shu.edu.cn/sy/tzgg.htm",
    },
    AnnouncementSource {
        id: "scie",
        name: "通信与信息工程学院",
        owner: "1508948420",
        column_ids: &["1065", "1070"],
        list_url: "https://scie.shu.edu.cn/sywzzl/tzgg.htm",
    },
    AnnouncementSource {
        id: "xiangying",
        name: "翔英学院",
        owner: "1600705763",
        column_ids: &["1031"],
        list_url: "https://xiangying.shu.edu.cn/syxgzlpzd/tzgg.htm",
    },
    AnnouncementSource {
        id: "sfs",
        name: "外国语学院",
        owner: "1507601601",
        column_ids: &["1141"],
        list_url: "https://sfs.shu.edu.cn/index/tzgg.htm",
    },
    AnnouncementSource {
        id: "sjc",
        name: "新闻传播学院",
        owner: "1496137373",
        column_ids: &["1003"],
        list_url: "https://sjc.shu.edu.cn/tg/xygg.htm",
    },
    AnnouncementSource {
        id: "sme",
        name: "微电子学院",
        owner: "1880700804",
        column_ids: &["1030"],
        list_url: "https://sme.shu.edu.cn/index/tzgg.htm",
    },
    AnnouncementSource {
        id: "smes",
        name: "力学与工程科学学院",
        owner: "1570605493",
        column_ids: &["1016", "1015"],
        list_url: "https://smes.shu.edu.cn/index/xsxx.htm",
    },
    AnnouncementSource {
        id: "sociology",
        name: "社会学院",
        owner: "1658331948",
        column_ids: &["1103"],
        list_url: "https://sociology.shu.edu.cn/synr/tzgg.htm",
    },
    AnnouncementSource {
        id: "tiyu",
        name: "体育学院",
        owner: "1604366189",
        column_ids: &["1073"],
        list_url: "https://tiyu.shu.edu.cn/syxwzl/tzgg.htm",
    },
    AnnouncementSource {
        id: "cs",
        name: "计算机工程与科学学院",
        owner: "2105799499",
        column_ids: &["1081", "1061"],
        list_url: "https://cs.shu.edu.cn/index/zytz.htm",
    },
    AnnouncementSource {
        id: "ai",
        name: "未来技术学院",
        owner: "1682478267",
        column_ids: &["1150"],
        list_url: "https://ai.shu.edu.cn/xwtz/tzgg.htm",
    },
    AnnouncementSource {
        id: "medicine",
        name: "医学院",
        owner: "1556105810",
        column_ids: &["1054"],
        list_url: "https://medicine.shu.edu.cn/index/tzgg.htm",
    },
    AnnouncementSource {
        id: "music",
        name: "音乐学院",
        owner: "1652596067",
        column_ids: &["1024", "1175"],
        list_url: "https://music.shu.edu.cn/xwzx/tzgg.htm",
    },
    AnnouncementSource {
        id: "modart",
        name: "巴黎国际时装艺术学院",
        owner: "1580473210",
        column_ids: &["1087", "1009", "1010"],
        list_url: "https://modart.shu.edu.cn/tzgg.htm",
    },
    AnnouncementSource {
        id: "safa",
        name: "上海美术学院",
        owner: "1676726341",
        column_ids: &["1122"],
        list_url: "https://safa.shu.edu.cn/synr/tzgg.htm",
    },
    AnnouncementSource {
        id: "cce",
        name: "继续教育学院",
        owner: "1429464619",
        column_ids: &["1014", "1542", "1464"],
        list_url: "https://cce.shu.edu.cn/symk/tzgg.htm",
    },
    AnnouncementSource {
        id: "zhgy",
        name: "卓越工程师学院",
        owner: "1997493522",
        column_ids: &["1021"],
        list_url: "https://zhgy.shu.edu.cn/tzgg.htm",
    },
    AnnouncementSource {
        id: "sfa",
        name: "上海电影学院",
        owner: "1600705553",
        column_ids: &["1212"],
        list_url: "https://sfa.shu.edu.cn/index/tzgg.htm",
    },
    AnnouncementSource {
        id: "mkszyxy",
        name: "马克思主义学院",
        owner: "1653118915",
        column_ids: &["1129"],
        list_url: "https://mkszyxy.shu.edu.cn/sylm/tzgg.htm",
    },
    AnnouncementSource {
        id: "law",
        name: "法学院",
        owner: "1498770058",
        column_ids: &["1012"],
        list_url: "https://law.shu.edu.cn/zxzx/tzgg.htm",
    },
    AnnouncementSource {
        id: "schim",
        name: "文化遗产与信息管理学院",
        owner: "1861247740",
        column_ids: &["1375"],
        list_url: "https://schim.shu.edu.cn/xwdt/tzgg.htm",
    },
    AnnouncementSource {
        id: "soe",
        name: "经济学院",
        owner: "1657520282",
        column_ids: &["1192"],
        list_url: "https://soe.shu.edu.cn/index/sytzgg.htm",
    },
    AnnouncementSource {
        id: "silc",
        name: "悉尼工商学院",
        owner: "1542326829",
        column_ids: &["1035"],
        list_url: "https://silc.shu.edu.cn/tzgg.htm",
    },
    AnnouncementSource {
        id: "auto",
        name: "机电工程与自动化学院",
        owner: "1498552290",
        column_ids: &["1123"],
        list_url: "https://auto.shu.edu.cn/synr/tzgg.htm",
    },
    AnnouncementSource {
        id: "bio",
        name: "生命科学学院",
        owner: "1656223304",
        column_ids: &["1145"],
        list_url: "https://bio.shu.edu.cn/sylm/tzgg.htm",
    },
    AnnouncementSource {
        id: "ulisboas",
        name: "里斯本学院",
        owner: "1852683123",
        column_ids: &["1049"],
        list_url: "https://ulisboas.shu.edu.cn/index/tzgg.htm",
    },
    AnnouncementSource {
        id: "utseus",
        name: "中欧工程技术学院",
        owner: "1430791623",
        column_ids: &["1010"],
        list_url: "https://utseus.shu.edu.cn/symkzj/gg.htm",
    },
];

/// 全校所有支持搜索部门的 Owner ID 逗号连接字符串。
pub const GLOBAL_OWNERS: &str = "1864855249,1662210750,1364902694,1462468299,1565422366,1495423387,1597130582,1880701254,1393148177,1705785581,1551701726,1812157458,1425312184,1623616573,1663284523,1659861027,1479802255,1675169118,1678802850,1663770242,1530700580,1598626614,1504253137,1500565633,1880699954,1449189309,1797966698,1650566287,1771177353,1691313259,1676522650,2052219376,1471238366,2063254418,1649809844,1730694740,1880701068,1678865198,2132341510,2132008928,1339981612,1383592681,1657519516,1600705936,1381981180,1508948420,1600705763,1507601601,1496137373,1880700804,1570605493,1658331948,1604366189,2105799499,1682478267,1556105810,1652596067,1580473210,1676726341,1429464619,1997493522,1600705553,1653118915,1498770058,1861247740,1657520282,1542326829,1498552290,1656223304,1852683123,1430791623";

/// 全校所有支持搜索栏目的 Column ID 去重逗号连接字符串。
pub const GLOBAL_COLUMN_IDS: &str = "1051,1015,1798,1810,7743,7563,7373,7161,2175,1812,1826,7753,7573,7383,7171,2168,1839,7763,7583,7393,7201,2176,1846,7773,7593,7403,7211,2169,1878,7603,7413,7262,2171,7783,1880,1928,7823,7643,7453,7251,2174,1893,7793,7613,7423,7272,2172,1906,7623,7433,7241,2177,7803,1912,1915,7813,7633,7443,7151,2173,1003,1004,1008,1007,1050,1029,1027,1030,1031,1041,1022,1023,1021,1005,1011,1061,1095,1010,1090,1013,1014,1017,1019,1020,1303,1304,1147,1148,1018,1024,1062,1082,1085,1086,1057,1042,1043,1044,1531,1541,1012,1071,1502,1111,1113,1112,1245,1006,1009,1065,1070,1141,1016,1103,1073,1081,1150,1054,1175,1087,1122,1542,1464,1212,1129,1375,1192,1035,1123,1145,1049";

/// 按 Owner ID 索引的数据源映射表。
static SOURCES_BY_OWNER: LazyLock<HashMap<&'static str, &'static AnnouncementSource>> =
    LazyLock::new(|| {
        let mut map = HashMap::with_capacity(SOURCES.len());
        for source in SOURCES {
            map.insert(source.owner, source);
        }
        map
    });

/// 根据 Owner ID 获取对应的数据源元数据。
#[must_use]
pub fn get_source_by_owner(owner: &str) -> Option<&'static AnnouncementSource> {
    SOURCES_BY_OWNER.get(owner).copied()
}

/// 将相对 URL 基于部门基准地址还原为绝对 URL。
#[must_use]
pub fn resolve_article_url(owner: &str, raw_url: &str) -> String {
    let trimmed = raw_url.trim();
    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        if let Some(pos) = trimmed.find('#') {
            return trimmed[..pos].to_string();
        }
        return trimmed.to_string();
    }

    let base_url = get_source_by_owner(owner).map_or("https://www.shu.edu.cn", |s| s.list_url);

    if let Ok(base) = reqwest::Url::parse(base_url)
        && let Ok(resolved) = base.join(trimmed)
    {
        let s = resolved.as_str();
        if let Some(pos) = s.find('#') {
            return s[..pos].to_string();
        }
        return s.to_string();
    }
    format!("https://www.shu.edu.cn{trimmed}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sources_count() {
        assert_eq!(SOURCES.len(), 71);
    }

    #[test]
    fn test_get_source_by_owner() {
        let shu = get_source_by_owner("1864855249");
        assert!(shu.is_some());
        if let Some(s) = shu {
            assert_eq!(s.id, "shu");
            assert_eq!(s.name, "上海大学官网");
        }
    }

    #[test]
    fn test_resolve_relative_url() {
        let resolved = resolve_article_url("1864855249", "/info/1051/123.htm#top");
        assert_eq!(resolved, "https://www.shu.edu.cn/info/1051/123.htm");

        let absolute = resolve_article_url("1864855249", "https://news.shu.edu.cn/info/1.htm");
        assert_eq!(absolute, "https://news.shu.edu.cn/info/1.htm");
    }
}
