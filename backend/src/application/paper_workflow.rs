use std::sync::Arc;

use tokio::sync::RwLock;
use uuid::Uuid;

use crate::domain::paper::{Paper, PaperStatus, PaperSummary};
use crate::domain::repositories::SharedPaperRepository;
use crate::domain::research::SharedResearchSource;
use crate::error::{AppError, AppResult};
use crate::llm::{Interpreter, LlmClient};
use crate::models::api::ProgressInfo;
use crate::pdf::ExtractResult;

/// 论文学习工作流应用服务。
///
/// 目前先作为 routes 与领域/基础设施之间的组合边界，后续可把上传、解读、
/// Feynman Loop、概念实验室等用例逐步迁入这里。
#[derive(Clone)]
pub struct PaperWorkflow {
    pub(super) papers: SharedPaperRepository,
    pub(super) llm: LlmClient,
    pub(super) interpreter: Interpreter,
    pub(super) research: SharedResearchSource,
    pub(super) progress: Arc<RwLock<std::collections::HashMap<Uuid, ProgressInfo>>>,
}

impl PaperWorkflow {
    pub fn new(
        papers: SharedPaperRepository,
        llm: LlmClient,
        interpreter: Interpreter,
        research: SharedResearchSource,
        progress: Arc<RwLock<std::collections::HashMap<Uuid, ProgressInfo>>>,
    ) -> Self {
        Self {
            papers,
            llm,
            interpreter,
            research,
            progress,
        }
    }

    pub fn llm_is_configured(&self) -> bool {
        self.llm.is_configured()
    }

    pub async fn recover_interrupted_work(&self) -> AppResult<()> {
        let affected = self
            .papers
            .mark_interrupted_processing_as_failed()
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        if affected > 0 {
            tracing::warn!(
                affected,
                "检测到服务重启前遗留的 processing 论文，已标记为 failed"
            );
        }

        Ok(())
    }

    pub async fn register_extracted_paper(
        &self,
        filename: String,
        extracted: ExtractResult,
    ) -> AppResult<PaperSummary> {
        let now = chrono::Utc::now().to_rfc3339();
        let paper = Paper {
            id: Uuid::new_v4(),
            filename,
            title: extracted.title,
            authors: extracted.authors,
            char_count: extracted.full_text.chars().count(),
            full_text: extracted.full_text,
            status: PaperStatus::Uploaded,
            created_at: now,
            completed_at: None,
        };

        self.papers
            .insert_paper(&paper)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        self.update_progress(
            paper.id,
            ProgressInfo::new(
                "uploaded",
                "文本已提取",
                &format!("已提取 {} 字符，准备开始 AI 解读", paper.char_count),
                10,
            ),
        )
        .await;

        if self.llm.is_configured() {
            self.spawn_interpretation(paper.clone());
        } else {
            tracing::warn!(
                "LLM 未配置，跳过解读。论文已保存，可在配置 OPENAI_API_KEY 后手动触发。"
            );
            self.update_progress(
                paper.id,
                ProgressInfo::new(
                    "failed",
                    "LLM 未配置",
                    "未配置 OPENAI_API_KEY，请在 .env 中设置后重新上传。",
                    0,
                ),
            )
            .await;
        }

        Ok(paper.into())
    }

    pub async fn list_papers(&self) -> AppResult<Vec<PaperSummary>> {
        self.papers
            .list_papers()
            .await
            .map_err(|e| AppError::Database(e.to_string()))
    }

    pub async fn get_paper_detail(
        &self,
        id: Uuid,
    ) -> AppResult<(Paper, Option<crate::domain::interpretation::Interpretation>)> {
        let paper = self
            .papers
            .get_paper(id)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?
            .ok_or_else(|| AppError::NotFound(format!("论文 {id} 不存在")))?;

        let interpretation = if matches!(paper.status, PaperStatus::Completed) {
            self.papers
                .get_interpretation(id)
                .await
                .map_err(|e| AppError::Database(e.to_string()))?
        } else {
            None
        };

        Ok((paper, interpretation))
    }

    pub async fn update_progress(&self, paper_id: Uuid, info: ProgressInfo) {
        let mut map = self.progress.write().await;
        map.insert(paper_id, info);
    }

    pub async fn get_progress(&self, paper_id: Uuid) -> Option<ProgressInfo> {
        let map = self.progress.read().await;
        if let Some(info) = map.get(&paper_id).cloned() {
            return Some(info);
        }
        drop(map);

        let paper = match self.papers.get_paper(paper_id).await {
            Ok(Some(paper)) => paper,
            _ => return None,
        };

        Some(match paper.status {
            PaperStatus::Uploaded => ProgressInfo::new(
                "uploaded",
                "等待解读",
                "论文已保存，正在等待 AI 解读任务启动。",
                10,
            ),
            PaperStatus::Processing => ProgressInfo::new(
                "interpreting",
                "解读恢复中",
                "服务没有找到该任务的实时进度。如果此状态持续不变，请重新上传或打开已完成的历史解读。",
                35,
            ),
            PaperStatus::Completed => ProgressInfo::new(
                "completed",
                "解读完成",
                "解读结果已保存，可以打开交互式讲解页面。",
                100,
            ),
            PaperStatus::Failed => ProgressInfo::new(
                "failed",
                "解读中断",
                "这次解读任务没有完成，可能是服务重启、LLM 调用超时或返回格式异常。请重新上传，或打开历史已完成版本。",
                0,
            ),
        })
    }

    fn spawn_interpretation(&self, paper: Paper) {
        let store = self.papers.clone();
        let interpreter = self.interpreter.clone();
        let progress = self.progress.clone();
        let paper_id = paper.id;
        let title = paper.title.clone();
        let text = paper.full_text.clone();

        let update = move |info: ProgressInfo| {
            let progress = progress.clone();
            async move {
                let mut map = progress.write().await;
                map.insert(paper_id, info);
            }
        };

        tokio::spawn(async move {
            if let Err(e) = store.update_status(paper_id, PaperStatus::Processing).await {
                tracing::error!(paper_id = %paper_id, "更新状态失败: {e}");
                return;
            }

            update(ProgressInfo::new(
                "interpreting",
                "开始解读",
                "正在构建 Prompt 并发送给 AI...",
                20,
            ))
            .await;

            tracing::info!(paper_id = %paper_id, "开始 LLM 解读");

            update(ProgressInfo::new(
                "interpreting",
                "AI 分析中",
                "AI 正在阅读论文，提取核心贡献与关键概念...",
                35,
            ))
            .await;

            match interpreter.interpret(paper_id, &title, &text).await {
                Ok(interp) => {
                    update(ProgressInfo::new(
                        "parsing",
                        "解析结果",
                        "AI 返回完成，正在解析结构化讲解内容...",
                        80,
                    ))
                    .await;

                    if let Err(e) = store.save_interpretation(&interp).await {
                        tracing::error!(paper_id = %paper_id, "保存解读失败: {e}");
                        let _ = store.update_status(paper_id, PaperStatus::Failed).await;
                        update(ProgressInfo::new(
                            "failed",
                            "保存失败",
                            &format!("保存解读结果失败: {e}"),
                            0,
                        ))
                        .await;
                        return;
                    }

                    update(ProgressInfo::new(
                        "saving",
                        "保存完成",
                        "正在写入数据库并准备展示页面...",
                        95,
                    ))
                    .await;

                    if let Err(e) = store.update_status(paper_id, PaperStatus::Completed).await {
                        tracing::error!(paper_id = %paper_id, "更新状态失败: {e}");
                    }

                    update(ProgressInfo::new(
                        "completed",
                        "解读完成",
                        "全部完成，正在跳转到交互式讲解页面...",
                        100,
                    ))
                    .await;

                    tracing::info!(paper_id = %paper_id, "解读完成 ✓");
                }
                Err(e) => {
                    tracing::error!(paper_id = %paper_id, "解读失败: {e}");
                    let _ = store.update_status(paper_id, PaperStatus::Failed).await;
                    update(ProgressInfo::new(
                        "failed",
                        "解读失败",
                        &format!("AI 解读失败: {e}"),
                        0,
                    ))
                    .await;
                }
            }
        });
    }
}
