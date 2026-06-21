// ── API 封装（沿用 PhotoCurate-Rust 的 async function 导出 + store 调用模式）

import type {
  ClientLlmProfile,
  ConceptExpansion,
  HealthResponse,
  PaperDetail,
  PaperSummary,
  ProgressInfo,
  StudyPack,
  UploadResponse,
} from "@/types";
import { buildRequestLlmProfile, profileCacheKey } from "@/lib/llmProfile";

const BASE = "/api";
const conceptExpansionCache = new Map<string, Promise<ConceptExpansion>>();
const studyPackCache = new Map<string, Promise<StudyPack>>();

function clearConceptExpansionCache(paperId?: string) {
  if (!paperId) {
    conceptExpansionCache.clear();
    studyPackCache.clear();
    return;
  }

  for (const key of conceptExpansionCache.keys()) {
    if (key.startsWith(`${paperId}:`)) {
      conceptExpansionCache.delete(key);
    }
  }
  for (const key of studyPackCache.keys()) {
    if (key.startsWith(`${paperId}:`)) {
      studyPackCache.delete(key);
    }
  }
}

async function request<T>(url: string, opts?: RequestInit): Promise<T> {
  const res = await fetch(BASE + url, {
    headers: { "Content-Type": "application/json" },
    ...opts,
  });
  if (!res.ok) {
    const body = await res.json().catch(() => ({}));
    throw new Error(body?.message ?? `请求失败: ${res.status}`);
  }
  return res.json();
}

// ── Health ──────────────────────

export async function getHealth(): Promise<HealthResponse> {
  return request("/health");
}

// ── Papers ──────────────────────

export async function uploadPaper(file: File): Promise<UploadResponse> {
  if (file.type && file.type !== "application/pdf") {
    throw new Error("请上传 PDF 文件");
  }
  if (!file.name.toLowerCase().endsWith(".pdf")) {
    throw new Error("请上传 .pdf 文件");
  }

  clearConceptExpansionCache();
  const form = new FormData();
  form.append("file", file);
  const llmProfile = buildRequestLlmProfile();
  if (llmProfile) {
    form.append("llm_profile", JSON.stringify(llmProfile));
  }
  const res = await fetch(BASE + "/papers", {
    method: "POST",
    body: form,
  });
  if (!res.ok) {
    const body = await res.json().catch(() => ({}));
    throw new Error(body?.message ?? `上传失败: ${res.status}`);
  }
  return res.json();
}

export async function listPapers(): Promise<PaperSummary[]> {
  return request("/papers");
}

export async function getPaper(id: string): Promise<PaperDetail> {
  return request(`/papers/${id}`);
}

export async function getProgress(id: string): Promise<ProgressInfo> {
  return request(`/papers/${id}/progress`);
}

export async function retryInterpretation(id: string): Promise<UploadResponse> {
  clearConceptExpansionCache(id);
  const llmProfile = buildRequestLlmProfile();
  return request(`/papers/${id}/retry`, {
    method: "POST",
    body: JSON.stringify(llmProfile ? { llm_profile: llmProfile } : {}),
  });
}

export async function expandConcept(
  paperId: string,
  conceptId: string,
  profile: ClientLlmProfile | undefined = buildRequestLlmProfile(),
): Promise<ConceptExpansion> {
  const cacheKey = `${paperId}:${conceptId}:${profileCacheKey(profile)}`;
  const cached = conceptExpansionCache.get(cacheKey);
  if (cached) return cached;

  const pending = request<ConceptExpansion>(
    `/papers/${paperId}/concepts/${conceptId}/expand`,
    {
      method: "POST",
      body: JSON.stringify(profile ? { llm_profile: profile } : {}),
    },
  ).catch((error) => {
    conceptExpansionCache.delete(cacheKey);
    throw error;
  });

  conceptExpansionCache.set(cacheKey, pending);
  return pending;
}

export async function getStudyPack(
  paperId: string,
  profile: ClientLlmProfile | undefined = buildRequestLlmProfile(),
): Promise<StudyPack> {
  const cacheKey = `${paperId}:${profileCacheKey(profile)}`;
  const cached = studyPackCache.get(cacheKey);
  if (cached) return cached;

  const pending = request<StudyPack>(`/papers/${paperId}/study-pack`, {
    method: "POST",
    body: JSON.stringify(profile ? { llm_profile: profile } : {}),
  }).catch((error) => {
    studyPackCache.delete(cacheKey);
    throw error;
  });

  studyPackCache.set(cacheKey, pending);
  return pending;
}
