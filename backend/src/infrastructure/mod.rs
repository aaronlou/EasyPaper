// 基础设施层。
//
// 当前适配器仍保留在旧路径：
// - `crate::llm`：OpenAI-compatible LLM client 和解释器适配
// - `crate::pdf`：PDF 文本提取适配
// - `crate::store`：SQLite 仓储实现
// - `crate::prompt`：LLM prompt 模板
//
// 后续新增基础设施能力时优先放入本模块下，再逐步迁移旧路径。
pub mod search;
