//! `get_classroom_schedule` 字符串常量（严格零 Emoji）。

/// 楼宇参数为空错误。
pub const ERR_BUILDING_EMPTY: &str = "[错误] 教学楼参数 building 不能为空，请提供教学楼名称（如 \"A楼\"、\"A楼, AJ楼\"、\"行健楼\"）。";

/// 未识别出任何已知教学楼错误。
pub const ERR_NO_VALID_BUILDINGS: &str = "[错误] 未识别到任何有效的教学楼名称。全校支持查询的教学楼包括：\n\
- 宝山校区：A楼/AJ楼, B楼/BJ楼, C楼/CJ楼, D楼/DJ楼, GA楼/GJ楼, EJ楼/FJ楼, J楼, 经管楼, 计算机/通信楼, 材料楼\n\
- 延长校区：行健楼, 第四教学楼, 仪表楼\n\
- 嘉定校区：第一教学楼, 第二教学楼, 文达楼";

/// 参数缺失提示：未传入 room 且未传入 query。
pub const WARN_ROOM_OR_QUERY_REQUIRED: &str = "[提示] 请至少提供 room（教室门牌号，如 \"A101\"）以查看教室全天日程，或提供 query（关键词，如课程名、教师名）以搜索上课地点。\n\
若需查询当前时段或全天有哪些教室空闲，请使用 query_empty_classrooms 工具。";
