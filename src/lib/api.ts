// ── API 封装（沿用 PhotoCurate-Rust 的 async function 导出 + store 调用模式）

import type {
  HealthResponse,
  PaperDetail,
  PaperSummary,
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
