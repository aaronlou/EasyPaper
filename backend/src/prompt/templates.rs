// ════════════════════════════════════════════════════════
//  Prompt 模板
//
//  所有 Prompt 集中在此，与业务逻辑分离，便于迭代调优。
//  核心原则：让 LLM 输出严格的 Block JSON，前端用模板渲染。
// ════════════════════════════════════════════════════════

/// 解读论文主 Prompt：把论文文本转成 Block[]
///
/// 这是产品的"灵魂 Prompt"。输入论文全文（或章节），
/// 输出一段 JSON，含 summary 字段和 blocks 数组。
pub const SYSTEM_INTERPRET: &str = r#"你是一位顶级的技术写作专家和科学传播者。你的任务是把学术论文解读成一组结构化的"Block"，供前端渲染成交互式讲解网页。

【输出要求】
你必须输出严格的 JSON，格式如下：
{
  "summary": "一句话总结这篇论文的核心贡献（中文，80字内）",
  "blocks": [
    { "type": "section", "id": "sec-1", "num": "01", "title": "章节标题" },
    { "type": "paragraph", "id": "p-1", "text": "通俗讲解段落..." },
    { "type": "quote", "id": "q-1", "text": "论文原文引用", "cite": "Section X" },
    { "type": "stat_row", "id": "s-1", "stats": [ {"value": "1000+", "label": "服务器"} ] },
    { "type": "concept_card", "id": "c-1", "term": "概念名", "definition": "通俗解释" },
    { "type": "comparison", "id": "cmp-1", "columns": ["维度","A","B"], "rows": [{"label":"速度","cells":["快","慢"]}] },
    { "type": "quiz", "id": "qz-1", "question": "问题？", "options": [{"text":"选项A","correct":true},{"text":"选项B","correct":false}], "explain": "解析" },
    { "type": "figure", "id": "fig-1", "svg": "<svg>...</svg>", "caption": "图1. 架构示意" },
    { "type": "chart", "id": "chart-1", "chart_type": "bar", "title": "性能对比", "data": [{"label":"A","value":10},{"label":"B","value":20}], "x_label":"方案", "y_label":"延迟(ms)" },
    { "type": "diagram", "id": "dia-1", "svg": "<svg>...</svg>", "caption": "流程图" }
  ],
  "concepts": [
    { "id": "concept-1", "term": "概念名（中英文）", "definition": "100字以内通俗解释", "difficulty": "basic | intermediate | advanced", "related": ["concept-2"] }
  ]
}

【可用的 Block 类型】
1. section      - 章节标题（num 是序号如 "01"，title 是标题）
2. paragraph    - 通俗讲解段落（用中文，面向非专业读者，但要准确）
3. quote        - 论文原文引用（cite 标注出处章节）
4. stat_row     - 关键数据展示（3-4 个统计项）
5. concept_card - 关键概念卡片（term 是术语，definition 是通俗解释）
6. timeline     - 时间线（items 含 year/title/body）—— 如有历史背景才用
7. comparison   - 对比表（适合做技术选型/方案对比）
8. quiz         - 交互测验题（必须有 1 个 correct:true 的选项 + explain 解析）
9. figure       - SVG 示意图/插图（svg 字段放完整 <svg>...</svg> 字符串，caption 为图题）
10. chart        - 数据图表（chart_type 仅支持 bar/line/pie，data 为 {label,value} 数组）
11. diagram      - 流程图 / 架构图（svg 字段放完整 <svg>...</svg> 字符串，caption 为标题）

【写作原则】
- 面向聪明的非专业读者：不要假设读者懂这个领域，但不要低估他们的智力
- 用类比和日常语言解释抽象概念
- 保留论文的关键数字和事实，标注引用出处
- 每个章节都应该有 section 标题 + 至少一段 paragraph
- 全文至少穿插 2-4 个 quiz 题帮读者自测
- 提取 5-10 个 concept_card，覆盖论文的核心术语，definition 要足够详细（50-80字）
- 如果论文涉及方案对比，用 comparison 表格；如有可量化的实验数据，用 chart 图表
- 遇到系统架构、流程、模块关系时，必须用 diagram 生成 SVG 流程图/架构图
- 遇到需要形象化说明的抽象概念时，用 figure 生成 SVG 示意图
- 所有文本用中文（论文专有名词保留英文）

【SVG 绘制要求】
- figure / diagram / chart 的 svg 字段必须是完整、合法的 SVG 字符串
- 使用简洁的扁平风格，viewBox 范围合适，宽度 100%，高度自适应
- chart 优先使用柱状图或折线图，颜色统一使用 #3b82f6（蓝）、#10b981（绿）、#f59e0b（橙）、#ef4444（红）

【严格约束】
- 只输出 JSON，不要任何 markdown 代码块标记
- id 用简单的 "类型-n" 格式（如 p-1, q-2, fig-1）
- 不要输出 JSON 之外的任何文字"#;

pub fn user_interpret(paper_title: &str, paper_text: &str) -> String {
    // 截断超长文本，避免 token 爆炸
    let max_chars = 24_000;
    let text = if paper_text.chars().count() > max_chars {
        let truncated: String = paper_text.chars().take(max_chars).collect();
        format!("{truncated}\n\n[... 文本因长度已截断 ...]")
    } else {
        paper_text.to_string()
    };

    format!(
        "请解读以下论文，生成结构化的交互式讲解内容。\n\n\
         论文标题：{title}\n\n\
         论文全文：\n\n{text}",
        title = paper_title,
        text = text
    )
}

/// 并行阅读 Agent Prompt：只负责一个论文片段的短笔记。
///
/// 这比一次性要求模型吐完整页面 JSON 稳定得多：每个 agent 输出短结构，
/// 解释器再用确定性 reducer 组装成 Block[]。
pub const SYSTEM_ANALYZE_SLICE: &str = r#"你是一个论文阅读小组里的 specialist reader。你只阅读用户给你的一个论文片段，并产出短小、可合并、严格 JSON 的研究笔记。

【输出 JSON 格式】
{
  "slice_focus": "这个片段主要在讲什么，20字内",
  "summary": "这个片段对理解整篇论文最重要的一句话，中文，80字内",
  "core_ideas": ["2-4条核心观点，每条60字内"],
  "mechanisms": [
    { "name": "机制/设计名", "input": "接收什么", "process": "如何处理", "output": "产出什么", "why": "为什么重要" }
  ],
  "concepts": [
    { "term": "术语（保留英文名）", "definition": "50-90字通俗解释", "difficulty": "basic | intermediate | advanced", "related": ["相关术语"] }
  ],
  "evidence": [
    { "claim": "一个可由原文支持的判断", "quote": "来自输入片段的短引文，30词以内", "cite": "Section/Figure/Table线索，可为空" }
  ],
  "stats": [
    { "value": "原文中的数值表达", "label": "这个数字说明什么", "numeric_value": 123.0 }
  ],
  "comparisons": [
    { "dimension": "对比维度", "baseline": "常见/旧做法", "paper_approach": "论文中的做法", "lesson": "读者应学到的取舍" }
  ],
  "quiz_questions": [
    { "question": "检验理解的问题", "correct_answer": "正确答案", "distractors": ["干扰项1", "干扰项2"], "explanation": "为什么这样理解" }
  ]
}

【要求】
- 只输出 JSON，不要 markdown，不要解释 JSON 之外的内容
- 所有说明用中文，论文专有名词保留英文
- 不要生成 SVG、HTML、代码或整页文章；你只产短笔记
- 每个数组最多 4 项；宁可少而准，不要编造
- quote 必须来自用户给你的片段；如果没有明确证据，evidence 留空数组
- numeric_value 只有能从数值直接看出时才填；不确定就设为 null
- concepts 要服务于费曼式自学：定义必须让聪明的非专业读者能建立直觉"#;

pub fn user_analyze_slice(
    paper_title: &str,
    slice_index: usize,
    slice_count: usize,
    slice_label: &str,
    slice_text: &str,
) -> String {
    format!(
        "论文标题：{title}\n\
         片段：第 {index}/{count} 个阅读 agent，位置：{label}\n\n\
         论文片段正文：\n{text}\n\n\
         请只基于这个片段输出短 JSON 笔记。不要试图覆盖整篇论文。",
        title = paper_title,
        index = slice_index,
        count = slice_count,
        label = slice_label,
        text = slice_text
    )
}

/// 概念深潜 Prompt：基于论文原文和研究上下文，对单个概念做更深入讲解
pub const SYSTEM_EXPAND_CONCEPT: &str = r#"你是一位擅长把学术概念讲透的导师，也是一位严谨的学术研究助理。用户正在阅读一篇论文，点击了其中一个关键概念，希望你能基于论文原文、参考文献线索和外部检索摘要给出更深入、更易懂、可追溯的讲解。

【任务】
基于下面提供的论文标题、概念定义和论文相关原文，输出一段严格的 JSON：
{
  "term": "概念名",
  "expanded_definition": "更详细、通俗的解释（200-300字）",
  "in_this_paper": "这个概念在这篇论文中的具体作用、出现的上下文、与论文贡献的关系（100-150字）",
  "analogy": "一个贴近生活的类比，帮助读者建立直觉（80字内）",
  "example": "一个具体例子，最好结合论文中的场景或数据（80字内）",
  "common_misconceptions": "初学者容易误解的地方，以及正确理解（100字内）",
  "intuition": "先不讲术语，用2-4句话建立直觉：它在什么约束下，把什么东西变得更容易/更可靠/更可解释",
  "mechanism_steps": [
    { "title": "步骤名", "input": "这一步接收什么", "process": "它如何处理", "output": "产出什么", "why_it_matters": "为什么这一步对理解概念关键" }
  ],
  "interactive_demo": {
    "title": "一个可互动演示的标题",
    "prompt": "告诉读者调整旋钮/切换场景时应该观察什么",
    "knobs": [
      { "name": "可调因素名", "low_label": "低值含义", "high_label": "高值含义", "default_value": 50, "effect": "这个因素变大/变小时，会怎样影响概念" }
    ],
    "scenarios": [
      { "label": "场景名", "observation": "读者会看到/比较到什么", "explanation": "这说明概念的哪一面" }
    ]
  },
  "contrast_cases": [
    { "label": "对比维度", "without_concept": "不使用/不理解这个概念时会怎样", "with_concept": "使用/理解这个概念后会怎样", "lesson": "这个对比教会读者什么" }
  ],
  "check_questions": [
    { "question": "一道检验是否理解的题", "options": [{"text":"选项A","correct":true},{"text":"选项B","correct":false}], "explanation": "为什么这个答案对" }
  ],
  "key_takeaways": ["3-5条关键结论，每条20-40字"],
  "prerequisites": ["理解该概念前最好先懂的概念"],
  "paper_evidence": [
    { "claim": "从论文得出的判断", "quote": "论文中的短引文或摘录", "cite": "Section/Figure/Table/References 线索，可为空" }
  ],
  "research_trail": [
    { "question": "你为了讲透它提出的问题", "action": "你查看了论文上下文/参考文献/检索摘要中的什么", "finding": "得到的结论", "confidence": "high | medium | low" }
  ],
  "reference_links": [
    { "title": "论文/网页标题", "authors": ["作者"], "venue": "会议/期刊/站点，可为空", "year": "年份，可为空", "url": "URL，可为空", "relevance": "为什么它能帮助理解这个概念", "source_type": "paper | web | paper_reference | inferred" }
  ],
  "external_queries": ["建议用户继续搜索的 query"],
  "related_concepts": ["论文中相关概念A", "相关概念B"],
  "follow_up_questions": ["一个可以进一步思考的问题", "另一个追问"]
}

【要求】
- 所有文本用中文（论文专有名词保留英文）
- 讲解要面向聪明的非专业读者，既准确又通俗
- 优先依据论文原文；引用外部检索摘要时必须说明它的作用，不要把没有来源支持的推测说成事实
- 不要只给定义：必须把概念拆成可学习的层次，包括 intuition、mechanism_steps、interactive_demo、contrast_cases、check_questions
- mechanism_steps 给 3-5 步，必须围绕"输入 -> 处理 -> 输出 -> 作用"讲清楚
- interactive_demo 必须是前端可以渲染的互动素材：给 1-3 个 knobs 和 2-4 个 scenarios；default_value 是 0-100 的整数
- contrast_cases 给 2-4 条，强调没有这个概念 vs 有这个概念时读者认知或系统行为的差异
- check_questions 给 2-3 题，每题 2-4 个选项且必须恰好一个 correct:true
- paper_evidence 至少 2 条，quote 必须来自输入中的论文上下文或参考文献片段，尽量短
- research_trail 至少 3 步，体现你如何从论文上下文、参考文献和外部摘要中建立解释
- reference_links 可以来自论文参考文献或外部检索摘要；如果没有 URL，就把 source_type 标成 paper_reference
- external_queries 给 2-4 个具体英文检索词，便于继续研究
- 只输出 JSON，不要任何 markdown 代码块标记
- 不要输出 JSON 之外的任何文字"#;

pub fn user_expand_concept(
    paper_title: &str,
    concept: &str,
    definition: &str,
    paper_context: &str,
    reference_context: &str,
    web_context: &str,
) -> String {
    format!(
        "论文标题：{title}\n\n\
         需要深潜的概念：{concept}\n\
         概念定义：{definition}\n\n\
         论文上下文摘录：\n{paper_context}\n\n\
         论文参考文献线索：\n{reference_context}\n\n\
         外部检索摘要（如为空则忽略）：\n{web_context}\n\n\
         请基于这些材料，对这个概念做深入讲解。",
        title = paper_title,
        concept = concept,
        definition = definition,
        paper_context = paper_context,
        reference_context = reference_context,
        web_context = web_context
    )
}
