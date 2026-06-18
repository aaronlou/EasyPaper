# 📄 EasyPaper

> 上传一篇学术 PDF，AI 解读生成富交互式讲解网页，并提取关键概念供深度探索。

## 架构

```
用户上传 PDF
     ↓
Rust 后端 (axum)           React 前端 (Vite)
  ├─ 提取文本 (pdf-extract)    ├─ UploadView（拖拽上传）
  ├─ LLM 解读 (OpenAI 兼容)    ├─ ProcessingView（进度）
  ├─ 持久化 (SQLite)           └─ ReaderView（交互讲解）
  └─ 静态托管 (dist)               ├─ 章节 / 段落 / 引用
           ↓                       ├─ 概念卡片 / 时间线 / 对比表
      Block JSON ◄───────────────  ├─ 交互测验
       (前后端共享类型契约)          └─ LLM 自定义片段
```

## 快速开始

### Docker 一站式部署（推荐生产）

项目提供单镜像部署：镜像内包含 React 静态资源和 Rust 后端，运行时只需要挂载 `/data` 持久化 SQLite 与上传目录。

#### 服务器直接拉取 GitHub 镜像

```bash
mkdir -p easypaper && cd easypaper
curl -fsSL https://raw.githubusercontent.com/aaronlou/EasyPaper/main/compose.yaml -o compose.yaml
curl -fsSL https://raw.githubusercontent.com/aaronlou/EasyPaper/main/.env.docker.example -o .env

# 编辑 .env，填入 OPENAI_API_KEY
docker compose pull
docker compose up -d
```

访问：

- Web 产品：http://服务器IP:8787
- 健康检查：http://服务器IP:8787/api/health

如果 GHCR 镜像尚未设为 public，需要先登录：

```bash
echo YOUR_GITHUB_TOKEN | docker login ghcr.io -u YOUR_GITHUB_USERNAME --password-stdin
```

#### 本地构建镜像

```bash
docker build -t easypaper:local .
cp .env.docker.example .env
# 编辑 .env，填入 OPENAI_API_KEY
docker compose up -d
```

#### GitHub 自动生成镜像

推送到 `main` 或发布 `v*.*.*` tag 后，GitHub Actions 会构建并推送：

```text
ghcr.io/aaronlou/easypaper:latest
ghcr.io/aaronlou/easypaper:<branch-or-tag>
ghcr.io/aaronlou/easypaper:sha-<commit>
```

首次发布后，请在 GitHub 仓库的 Packages 页面确认镜像可见性；如果服务器不想配置 `docker login`，把 package visibility 设为 public。

### 前置要求
- Rust ≥ 1.88（推荐 1.95+）
- Node.js ≥ 18（推荐 22+）

### 1. 安装依赖

```bash
# 前端
npm install

# 后端（首次编译较慢）
cd backend && cargo build
```

### 2. 配置 LLM

```bash
cp .env.example .env
# 编辑 .env，填入你的 OPENAI_API_KEY
# 也支持 DeepSeek 等 OpenAI 兼容 API
```

可选缓存策略：

- `EASYPAPER_CONCEPT_PREWARM_LIMIT=3`：论文解读完成后，后台预热前 N 个概念深潜结果；设为 `0` 可关闭。
- `EASYPAPER_CONCEPT_CACHE_TTL_DAYS=7`：概念深潜缓存保留天数；过期缓存会在服务启动时清理。

### 3. 启动开发

```bash
# 一键启动前后端
bash scripts/dev.sh

# 或者分别启动：
# 终端1：cargo run -p easypaper-backend
# 终端2：npm run dev
```

- 后端：http://localhost:8787
- 前端开发服务器：http://localhost:5173
- API 健康检查：http://localhost:8787/api/health

### 4. 构建生产版本

```bash
npm run build          # 前端 → dist/
# 然后把 dist/ 丢给 axum 的静态文件托管
```

Docker 镜像会自动执行这一步，并把 `dist/` 复制到运行镜像内的 `/app/dist`。

## 项目结构

```
EasyPaper/
├── Cargo.toml              # Rust workspace 根
├── package.json            # 前端 (React 19 + Vite 6 + Tailwind)
├── vite.config.ts          # /api → 127.0.0.1:8787 代理
├── .env.example            # 配置模板
├── Dockerfile              # 前端 + 后端多阶段构建
├── compose.yaml            # 服务器部署模板
├── .github/workflows/
│   └── docker-image.yml    # GHCR 镜像发布
│
├── backend/                # Rust 后端 crate
│   └── src/
│       ├── main.rs         # 二进制入口
│       ├── lib.rs          # library crate 模块出口
│       ├── app.rs          # composition root，装配依赖
│       ├── domain/         # 领域模型与仓储/研究端口
│       ├── application/    # 上传、解读、概念深潜等用例
│       ├── infrastructure/ # Web 检索等基础设施适配器
│       ├── interfaces/     # HTTP router + handlers
│       ├── config.rs       # 环境变量配置
│       ├── error.rs        # 错误处理
│       ├── pdf/            # PDF 文本提取
│       ├── llm/            # LLM 客户端 + 解读编排
│       ├── prompt/         # Prompt 模板
│       ├── models/         # 数据模型 + Block 协议
│       └── store/          # SQLite 持久化
│
├── src/                    # 前端源码
│   ├── App.tsx             # 根组件 + 视图切换
│   ├── views/              # UploadView / ProcessingView / ReaderView
│   ├── components/reader/  # Block 组件库（10 种）
│   ├── renderer/           # blockRenderer 渲染引擎
│   ├── stores/             # Zustand 状态管理
│   ├── lib/                # API 封装 / cn 工具
│   └── types/              # 共享类型（Block 协议）
│
└── scripts/
    └── dev.sh              # 一键开发启动
```

## Block 协议（前后端共享）

后端 LLM 不直接产出 HTML，而是产出一组结构化 **Block JSON**。前端 `blockRenderer` 把每种 Block 类型映射到对应 React 组件：

| Block 类型 | 渲染效果 | 状态 |
|---|---|---|
| `section` | 章节标题 | ✅ 已实现 |
| `paragraph` | 通俗讲解 | ✅ 已实现 |
| `quote` | 引用块 | ✅ 已实现 |
| `stat_row` | 数据卡片 | ✅ 已实现 |
| `concept_card` | 概念卡片（可展开） | ✅ 已实现 |
| `timeline` | 时间线 | ✅ 已实现 |
| `comparison` | 对比表 | ✅ 已实现 |
| `quiz` | 交互测验 | ✅ 已实现 |
| `code_fragment` | 代码块 | ✅ 已实现 |
| `custom_html` | LLM 自定义片段 | ✅ 已实现 |

## 技术栈

| 层 | 技术 |
|---|---|
| 后端 | Rust + axum 0.8 + tokio + reqwest + sqlx (SQLite) |
| 前端 | React 19 + TypeScript + Vite 6 + Tailwind 3 + Zustand |
| LLM | OpenAI 兼容 API（支持 DeepSeek 等） |
| PDF | pdf-extract 0.7 |

## 路线图

- [x] **M1** — 核心闭环（上传 → LLM 解读 → 交互展示）
- [ ] **M2** — 概念探索（知识图谱、参考关联、联网搜索增强）
- [ ] **M3** — 打磨（SSE 流式进度、历史列表、i18n）
- [ ] **M4** — 增强（RAG 问答、用户系统、多人协作）
