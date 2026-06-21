// ── Zustand 单 store：管理论文列表 + 当前选中论文 + 解读状态

import { create } from "zustand";
import type { PaperDetail, PaperSummary } from "@/types";
import * as api from "@/lib/api";

let currentPaperRequestId = 0;

interface PaperState {
  // 论文列表
  papers: PaperSummary[];
  loadingPapers: boolean;
  loadPapers: () => Promise<void>;

  // 当前选中论文的详情
  current: PaperDetail | null;
  loadingCurrent: boolean;
  loadPaper: (id: string) => Promise<void>;

  // 上传
  uploading: boolean;
  uploadPaper: (file: File) => Promise<string>; // returns paper id
  retryInterpretation: (id: string) => Promise<string>;

  // 健康检查
  health: { llm_configured: boolean } | null;
  checkHealth: () => Promise<void>;
}

export const usePaperStore = create<PaperState>((set, get) => ({
  papers: [],
  loadingPapers: false,
  loadPapers: async () => {
    set({ loadingPapers: true });
    try {
      const papers = await api.listPapers();
      set({ papers });
    } finally {
      set({ loadingPapers: false });
    }
  },

  current: null,
  loadingCurrent: false,
  loadPaper: async (id) => {
    const requestId = ++currentPaperRequestId;
    set({ loadingCurrent: true, current: null });
    try {
      const detail = await api.getPaper(id);
      if (requestId === currentPaperRequestId) {
        set({ current: detail });
      }
    } finally {
      if (requestId === currentPaperRequestId) {
        set({ loadingCurrent: false });
      }
    }
  },

  uploading: false,
  uploadPaper: async (file) => {
    set({ uploading: true });
    try {
      const res = await api.uploadPaper(file);
      // 顺手刷新列表
      await get().loadPapers();
      return res.paper.id;
    } finally {
      set({ uploading: false });
    }
  },

  retryInterpretation: async (id) => {
    const res = await api.retryInterpretation(id);
    await get().loadPapers();
    return res.paper.id;
  },

  health: null,
  checkHealth: async () => {
    try {
      const h = await api.getHealth();
      set({ health: { llm_configured: h.llm_configured } });
    } catch {
      set({ health: null });
    }
  },
}));
