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

- Web 产品：建议通过 Caddy 绑定域名后访问 `https://your-domain.com`
- 容器健康检查：在服务器上执行 `curl http://127.0.0.1:8787/api/health`

如果 GHCR 镜像尚未设为 public，需要先登录：

```bash
echo YOUR_GITHUB_TOKEN | docker login ghcr.io -u YOUR_GITHUB_USERNAME --password-stdin
```

#### 本地构建镜像

```bash
docker build -t easypaper:local .
cp .env.docker.example .env
# 编辑 .env，填入 OPENAI_API_KEY
EASYPAPER_IMAGE=easypaper:local docker compose up -d
```

#### 离线包部署（适合 GitHub/GHCR 网络很慢的服务器）

如果服务器访问 GitHub、GHCR 很慢，可以在本地构建镜像并打成离线包，服务器只负责加载镜像和启动容器。

本地执行：

```bash
bash scripts/package-offline-deploy.sh
```

脚本会生成类似：

```text
release/easypaper-offline-20260101-120000.tar.gz
```

把这个包上传到服务器：

```bash
scp release/easypaper-offline-*.tar.gz root@服务器IP:/opt/
```

服务器执行：

```bash
mkdir -p /opt/easypaper
tar -xzf /opt/easypaper-offline-*.tar.gz -C /opt/easypaper --strip-components=1
cd /opt/easypaper

# 第一次运行会创建 .env 并提示你编辑
bash deploy/offline-up.sh

# 编辑 .env，填入 OPENAI_API_KEY 和生产域名后，再运行一次
nano .env
bash deploy/offline-up.sh
```

后续更新也一样：本地重新生成离线包，上传到服务器，解压覆盖 `/opt/easypaper` 后运行 `bash deploy/offline-up.sh`。Docker volume `easypaper_data` 会保留 SQLite 数据库和上传文件。

#### GitHub 自动生成镜像

推送到 `main` 或发布 `v*.*.*` tag 后，GitHub Actions 会构建并推送：

```text
ghcr.io/aaronlou/easypaper:latest
ghcr.io/aaronlou/easypaper:<branch-or-tag>
ghcr.io/aaronlou/easypaper:sha-<commit>
```

首次发布后，请在 GitHub 仓库的 Packages 页面确认镜像可见性；如果服务器不想配置 `docker login`，把 package visibility 设为 public。

#### 使用 Caddy 绑定域名

生产环境推荐让 EasyPaper 容器只监听服务器本机 `127.0.0.1:8787`，由 Caddy 负责公网 HTTPS、证书续期和反向代理。项目里的 `compose.yaml` 已按这个方式配置。

1. 域名 DNS 添加 A 记录，指向服务器公网 IP：

```text
your-domain.com      A      服务器公网 IP
www.your-domain.com  A      服务器公网 IP
```

2. 服务器防火墙 / 云安全组开放 `80/tcp` 和 `443/tcp`。`8787` 不需要对公网开放。

3. 将 `deploy/Caddyfile.example` 复制到 Caddy 配置目录，替换成你的域名：

```caddyfile
your-domain.com, www.your-domain.com {
    encode zstd gzip

    request_body {
        max_size 60MB
    }

    reverse_proxy 127.0.0.1:8787
}
```

`request_body max_size 60MB` 需要大于 EasyPaper 的 50MB PDF 上传限制，否则大文件会先被 Caddy 拦截。

4. 修改 `.env`，至少配置 LLM Key 和生产域名：

```env
OPENAI_API_KEY=sk-xxxxxxxxxxxxxxxxxxxxxxxx
EASYPAPER_CORS_ORIGINS=https://your-domain.com,https://www.your-domain.com
```

5. 启动应用并重载 Caddy：

```bash
docker compose pull
docker compose up -d
curl http://127.0.0.1:8787/api/health

sudo caddy validate --config /etc/caddy/Caddyfile
sudo systemctl reload caddy
```

Caddy 会自动申请并续期 HTTPS 证书。证书签发前请确认域名已经解析到当前服务器，且 `80` / `443` 端口可从公网访问。

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

默认单 provider 配置仍然兼容：

- `OPENAI_API_KEY`：OpenAI 兼容 API Key。
- `OPENAI_BASE_URL=https://api.deepseek.com/v1`：可切换到 DeepSeek、OpenRouter、硅基流动等 OpenAI 兼容端点。
- `OPENAI_MODEL=deepseek-chat`：当前 provider 使用的模型。

如果你把产品开放给其他用户，页面右上角的 **AI 使用方式** 支持两种模式：

- **使用 EasyPaper AI**：用户不需要配置 API Key，后端使用产品托管 provider。正式上线前应接入登录、订阅校验、额度和用量记录。
- **使用自己的 Provider**：用户在浏览器里填入 DeepSeek、OpenAI、OpenRouter 等 provider 和 API Key。该配置保存在用户自己的浏览器 localStorage 中，并随上传、重试、概念深潜请求临时发送给后端使用；当前版本不会把用户 API Key 写入 SQLite。

没有账号系统时，推荐先开放自带 Provider 模式；托管 EasyPaper AI 模式适合接入订阅服务后提供给普通用户。

如果要同时配置多个 LLM provider，可使用 provider profile：

```bash
EASYPAPER_LLM_PROVIDERS=deepseek,openai

EASYPAPER_LLM_PROVIDER_DEEPSEEK_BASE_URL=https://api.deepseek.com/v1
EASYPAPER_LLM_PROVIDER_DEEPSEEK_MODEL=deepseek-chat
EASYPAPER_LLM_PROVIDER_DEEPSEEK_API_KEY_ENV=DEEPSEEK_API_KEY
DEEPSEEK_API_KEY=sk-...

EASYPAPER_LLM_PROVIDER_OPENAI_BASE_URL=https://api.openai.com/v1
EASYPAPER_LLM_PROVIDER_OPENAI_MODEL=gpt-4o-mini
EASYPAPER_LLM_PROVIDER_OPENAI_API_KEY_ENV=OPENAI_API_KEY
EASYPAPER_LLM_PROVIDER_OPENAI_RESPONSES_API=true

EASYPAPER_LLM_ROUTE_DEFAULT=deepseek,openai
EASYPAPER_LLM_ROUTE_READER=deepseek,openai
EASYPAPER_LLM_ROUTE_SPECIALIST=deepseek,openai
EASYPAPER_LLM_ROUTE_CONCEPT=deepseek,openai
EASYPAPER_LLM_ROUTE_REPAIR=openai,deepseek
EASYPAPER_LLM_ROUTE_STUDY=deepseek,openai
EASYPAPER_LLM_ROUTE_LITERATURE=openai,deepseek
EASYPAPER_LLM_ROUTE_TRANSLATION=deepseek,openai
```

每个 route 都是 fallback 顺序。比如 `EASYPAPER_LLM_ROUTE_READER=deepseek,openai` 表示 reader agent 先用 DeepSeek，失败后自动切 OpenAI；未单独配置的 agent role 会走 `EASYPAPER_LLM_ROUTE_DEFAULT`。

阅读页包含 **研究地图** 模块，会生成论文启发、结构逻辑、前提知识、继续研究方向、文献脉络、前后继研究和中文翻译摘要。该模块由独立的 Study Pack workflow 生成并缓存，不会混入主解读器。

可选缓存策略：

- `EASYPAPER_CONCEPT_PREWARM_LIMIT=3`：论文解读完成后，后台预热前 N 个概念深潜结果；设为 `0` 可关闭。
- `EASYPAPER_CONCEPT_PREWARM_CONCURRENCY=1`：后台概念预热的全局并发数，避免多篇论文同时完成时打满 LLM。
- `EASYPAPER_CONCEPT_CACHE_TTL_DAYS=7`：概念深潜缓存保留天数；过期缓存会在服务启动时清理。
- `EASYPAPER_CORS_ORIGINS=https://your-domain.com`：生产环境建议配置 CORS 白名单；未配置时仅允许 `localhost:5173` / `127.0.0.1:5173` 访问 API，方便本地开发。

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
