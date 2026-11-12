//! `query_empty_classrooms` 字符串常量（严格零 Emoji）。

/// 楼宇参数为空错误。
pub const ERR_BUILDING_EMPTY: &str = "[错误] 教学楼参数 building 不能为空，请提供教学楼名称（如 \"A楼\"、\"A楼, AJ楼\"、\"行健楼\"）。";

/// 未识别出任何已知教学楼错误。
pub const ERR_NO_VALID_BUILDINGS: &str = "[错误] 未识别到任何有效的教学楼名称。全校支持查询的教学楼包括：\n\
- 宝山校区：A楼/AJ楼, B楼/BJ楼, C楼/CJ楼, D楼/DJ楼, GA楼/GJ楼, EJ楼/FJ楼, J楼, 经管楼, 计算机/通信楼, 材料楼\n\
- 延长校区：行健楼, 第四教学楼, 仪表楼\n\
- 嘉定校区：第一教学楼, 第二教学楼, 文达楼";
