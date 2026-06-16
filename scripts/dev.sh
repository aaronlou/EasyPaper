#!/usr/bin/env bash
# ──────────────────────────────────────────────────────────
# EasyPaper 开发启动脚本
# 同时启动 Rust 后端（8787）和 Vite 前端开发服务器（5173）
# Vite 会自动把 /api 请求代理到后端
# ──────────────────────────────────────────────────────────
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

echo "══════════════════════════════════════"
echo "  📄 EasyPaper 开发模式"
echo "══════════════════════════════════════"
echo ""

# 确保 dist 目录存在（axum 启动时需要）
mkdir -p dist
touch dist/index.html

# ── 启动后端 ────────────────────────────────────────────
RED='\033[0;31m'
GRN='\033[0;32m'
CYN='\033[0;36m'
NC='\033[0m'

echo -e "${GRN}[1] Rust 后端${NC}"
cargo run -p easypaper-backend &
BACKEND_PID=$!
echo -e "  → PID: ${BACKEND_PID}"

# 等后端就绪
echo -n "  → 等待后端启动..."
for i in $(seq 1 30); do
  if curl -s http://127.0.0.1:8787/api/health > /dev/null 2>&1; then
    echo -e " ${GRN}就绪 ✓${NC}"
    break
  fi
  sleep 1
  echo -n "."
done

# ── 启动前端 ────────────────────────────────────────────
echo ""
echo -e "${CYN}[2] Vite 前端${NC}"
npx vite --host &
FRONTEND_PID=$!
echo -e "  → PID: ${FRONTEND_PID}"

echo ""
echo -e "══════════════════════════════════════"
echo -e "  ${GRN}后端${NC}  http://localhost:8787"
echo -e "  ${CYN}前端${NC}  http://localhost:5173"
echo -e "  ${GRN}API${NC}   http://localhost:8787/api/health"
echo -e "══════════════════════════════════════"
echo ""
echo -e "按 Ctrl-C 停止所有服务"

# 监听退出信号
cleanup() {
  echo ""
  echo "正在停止..."
  kill $BACKEND_PID 2>/dev/null || true
  kill $FRONTEND_PID 2>/dev/null || true
  wait
  echo "已停止。"
}
trap cleanup EXIT INT TERM
wait
