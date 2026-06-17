// ── API 封装（沿用 PhotoCurate-Rust 的 async function 导出 + store 调用模式）

import type {
  ConceptExpansion,
  HealthResponse,
  PaperDetail,
  PaperSummary,
  ProgressInfo,
  UploadResponse,
} from "@/types";

const BASE = "/api";

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
  const form = new FormData();
  form.append("file", file);
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
  return request(`/papers/${id}/retry`, {
    method: "POST",
  });
}

export async function expandConcept(
  paperId: string,
  conceptId: string,
): Promise<ConceptExpansion> {
  return request(`/papers/${paperId}/concepts/${conceptId}/expand`, {
    method: "POST",
  });
}
