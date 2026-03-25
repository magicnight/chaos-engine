# Crucix Engine + NewsPredict — 完整设计规格书

> **日期**: 2026-03-23
> **状态**: 已确认架构方向，规格审查通过，待实施
> **范围**: 两个独立产品的完整技术设计
> **版本**: v1.1 (修复规格审查发现的 21 项问题)

---

## 一、产品定义

### 1.1 Crucix Engine（Rust 情报引擎）

**定位**: 本地化 OSINT 情报采集、分析与服务引擎。单二进制分发，零云依赖。

**三种运行模式**:

| 模式 | 命令 | 绑定地址 | 场景 |
|------|------|---------|------|
| CLI 单次扫描 | `crucix sweep` | N/A | 命令行快速获取情报 |
| 本地 Web 服务 | `crucix serve` | `127.0.0.1:3117` | 本地仪表盘 + API (仅本机访问) |
| 对外 API 服务 | `crucix serve --public --api-key <key>` | `0.0.0.0:3117` | 配合 Caddy 反代，为 NewsPredict 提供数据 |

> **安全约束**: 本地模式**必须**绑定 `127.0.0.1`，防止局域网未授权访问。仅 `--public` 模式允许绑定 `0.0.0.0`，且强制要求 `--api-key` 参数。

**核心设计原则**:
- 零云依赖：支持完全本地运行，含 Ollama LLM
- 单二进制分发：Web 仪表盘 HTML 通过 `include_str!` 嵌入
- 渐进式增强：零 API 密钥即可运行 18+ 免费源
- 故障隔离：`join_all`（非 `try_join_all`），单源失败不影响整体
- 极简依赖：不引入 HTML 解析库、不引入 LLM SDK、不引入 dotenv crate

### 1.2 NewsPredict（预测市场 PWA）

**定位**: 基于 OSINT 情报的预测市场社交平台。消费 Crucix API 数据，提供用户参与式情报消费体验。

**核心功能**:
- 新闻情报的卡片式移动端展示
- 二元预测市场（YES/NO 合约）
- 用户投资组合与排行榜
- 社交功能（关注、趋势、推荐）
- Web3 渐进集成（积分 → BSC 代币 → 多链）

---

## 二、系统总体架构

```
                          ┌─────────────────────────────────┐
                          │         用户 (Users)             │
                          ├────────┬────────┬───────────────┤
                          │ 本地CLI │ 本地Web │ 手机PWA/桌面   │
                          └───┬────┴───┬────┴───────┬───────┘
                              │        │            │
                 ┌────────────┴────────┘            │
                 ▼                                  ▼
┌────────────────────────────────┐  ┌──────────────────────────────────┐
│     CRUCIX ENGINE (Rust)       │  │    NEWSPREDICT (Next.js 16)      │
│     单二进制 crucix             │  │    newspredict.app               │
│                                │  │                                  │
│  情报采集层:                    │  │  前端层 (App Router):            │
│  • 33 文件 / 34 逻辑源         │  │  • PWA 移动端适配                │
│  • tokio::join_all 并行        │  │  • 暗色卡片式 UI (shadcn/ui)     │
│  • 30s 单源超时                │  │  • 预测市场交互                  │
│                                │  │  • Web3 钱包连接 (Reown)         │
│  分析引擎:                      │  │                                  │
│  • Delta 变化检测              │  │  后端层 (API Routes):            │
│  • 跨源关联 (6 规则)           │  │  • NextAuth (邮箱+OAuth+钱包)    │
│  • LLM 合成 (8+ provider)     │  │  • 预测市场引擎 (LMSR)           │
│  • 市场种子生成                │  │  • 组合管理                      │
│                                │  │  • 排行榜/社交                   │
│  存储:                          │  │  • Web3 合约交互 (wagmi/viem)    │
│  • SQLite (bundled)            │  │                                  │
│  • runs/crucix.db              │  │  数据层:                         │
│                                │  │  • Neon PostgreSQL               │
│  输出:                          │  │  • Upstash Redis                 │
│  • REST API /api/v1/ (axum)◄───┼──│  • Crucix API 客户端             │
│  • SSE 实时推送                │  │                                  │
│  • CLI 文本/JSON               │  │  部署: Vercel                    │
│                                │  │                                  │
│  部署: Docker / Binary+Caddy   │  │                                  │
└────────────────────────────────┘  └──────────────────────────────────┘
```

### 2.1 仓库结构

两个独立仓库：

```
crucix/                          # Rust 情报引擎
newspredict/                     # Next.js 预测市场 PWA
```

### 2.2 通信契约

**API 版本化**: 所有端点使用 `/api/v1/` 前缀。主版本号变更表示不兼容变更，次版本号内仅允许追加字段（向后兼容）。

**认证**: `--public` 模式通过 `X-Crucix-Key` header 认证。

**缓存策略**: NewsPredict 缓存 Crucix 数据到 Redis，避免每次用户请求穿透：

| 端点 | 缓存 TTL | 失效策略 |
|------|---------|---------|
| `/api/v1/data` | 5 min | SSE `update` 事件触发刷新 |
| `/api/v1/events` | 5 min | SSE 驱动 |
| `/api/v1/market-seeds` | 30 min | 每次 sweep 后刷新 |
| `/api/v1/trends` | 15 min | 按 sweep 周期 |
| `/api/v1/correlations` | 5 min | SSE 驱动 |

**故障处理**: Crucix 不可达时，NewsPredict 使用 stale-while-revalidate 策略返回最近缓存数据，前端显示"情报数据暂时不可用"提示。SSE 断线使用指数退避重连（1s → 2s → 4s → ... → 60s 上限）。

**共享类型**: 维护 `crucix-types/` 目录存放 JSON Schema 定义，双方验证 API 契约一致性。

### 2.3 数据迁移策略（Node.js → Rust）

Rust 重写是**全新版本**，不直接兼容 Node.js v2.0.0 的数据格式：

- Node.js 版使用 JSON 文件存储 (`runs/latest.json`, `runs/memory/`)
- Rust 版使用 SQLite (`runs/crucix.db`)

提供可选迁移工具：
```bash
crucix migrate --from-json ./runs/   # 读取 JSON 文件导入 SQLite
```

如果不迁移，Rust 版从零开始积累数据，不影响功能。

---

## 三、Crucix Engine 详细设计

### 3.1 技术栈

| 组件 | 技术 | 说明 |
|------|------|------|
| 语言 | Rust (edition 2021) | 零 GC、真并发、单二进制 |
| 异步运行时 | tokio (multi-thread) | 并行数据采集 |
| HTTP 客户端 | reqwest (rustls) | 连接池、指数退避 |
| Web 框架 | axum | REST + SSE |
| 序列化 | serde + serde_json | JSON 核心 |
| 数据库 | rusqlite (bundled) | SQLite 编译进二进制 |
| CLI | clap (derive) | 结构化命令行 |
| 错误处理 | thiserror + anyhow | 类型安全错误链 |
| 时间 | chrono | UTC 时间戳 |
| LLM | reqwest 原生 HTTP | 零 SDK 依赖 |
| 限流 | tower-governor | 公共 API 速率限制 |

### 3.2 数据源清单：33 文件 / 34 逻辑源

`sanctions.rs` 内聚了 OFAC SDN 和 OpenSanctions 两个逻辑源（共享制裁领域，互为 failback），因此 33 个源文件覆盖 34 个逻辑数据源。

#### 数据源映射表（Node.js → Rust）

| Node.js 文件 | Rust 文件 | 说明 |
|-------------|----------|------|
| gdelt.mjs | gdelt.rs | 直接映射 |
| acled.mjs | acled.rs | 直接映射 |
| who.mjs | who.rs | 直接映射 |
| ofac.mjs + opensanctions.mjs | sanctions.rs | 合并为一个文件，两个逻辑源 |
| opensky.mjs | opensky.rs | 直接映射 |
| firms.mjs | firms.rs | 直接映射 |
| telegram.mjs | telegram.rs | 直接映射 (HTML 爬取) |
| safecast.mjs | safecast.rs | 直接映射 |
| reliefweb.mjs | reliefweb.rs | 直接映射 |
| ships.mjs | ships.rs | 直接映射 |
| adsb.mjs | adsb.rs | 直接映射 |
| fred.mjs | fred.rs | 直接映射 |
| eia.mjs | eia.rs | 直接映射 |
| treasury.mjs | treasury.rs | 直接映射 |
| bls.mjs | bls.rs | 直接映射 |
| gscpi.mjs | gscpi.rs | 直接映射 |
| usaspending.mjs | usaspending.rs | 直接映射 |
| comtrade.mjs | comtrade.rs | 直接映射 |
| noaa.mjs | noaa.rs | 直接映射 |
| epa.mjs | epa.rs | 直接映射 |
| reddit.mjs | reddit.rs | 直接映射 |
| bluesky.mjs | bluesky.rs | 直接映射 |
| patents.mjs | patents.rs | 直接映射 |
| space.mjs | celestrak.rs + swpc.rs | 拆分为卫星追踪 + 太空天气 |
| kiwisdr.mjs | kiwisdr.rs | 直接映射 |
| yfinance.mjs | yfinance.rs | 直接映射 |
| cisa-kev.mjs | cisa_kev.rs | 直接映射 |
| cloudflare-radar.mjs | cloudflare_radar.rs | 直接映射 |
| *(新增)* | usgs.rs | USGS 地震 (GeoJSON) |
| *(新增)* | gdacs.rs | 全球灾害预警 (RSS) |
| *(新增)* | cve.rs | NVD/CVE 漏洞情报 |
| *(新增)* | isc.rs | ISC/SANS 网络威胁 |

### 3.3 项目结构

```
crucix/
├── Cargo.toml
├── Dockerfile
├── docker-compose.yml
├── .env.example
├── Caddyfile.example
├── crucix-types/                    # 共享 JSON Schema (API 契约)
│   ├── sweep-data.schema.json
│   ├── market-seed.schema.json
│   └── event.schema.json
│
├── src/
│   ├── main.rs                      # CLI 入口 (clap derive)
│   ├── config.rs                    # .env 加载 (自实现)
│   ├── error.rs                     # thiserror 统一错误
│   ├── http.rs                      # reqwest 客户端 + 指数退避
│   ├── briefing.rs                  # 编排器
│   ├── store.rs                     # SQLite 持久化
│   ├── correlation.rs               # 跨源关联引擎
│   ├── notify.rs                    # 告警推送
│   ├── watchlist.rs                 # 用户自定义关注
│   ├── report.rs                    # Markdown/CSV 导出
│   ├── auth.rs                      # API Key 验证 + 速率限制
│   ├── migrate.rs                   # JSON → SQLite 迁移工具
│   │
│   ├── llm/
│   │   ├── mod.rs                   # LlmProvider trait + 工厂
│   │   ├── openai_compat.rs         # OpenAI 兼容协议 (覆盖 10+ 提供商)
│   │   ├── anthropic.rs             # Anthropic Claude (原生 API)
│   │   ├── gemini.rs                # Google Gemini (原生 API)
│   │   ├── ollama.rs                # 本地 Ollama
│   │   └── market_seeds.rs          # LLM 生成预测市场种子
│   │
│   ├── sources/                     # 33 文件, 34 逻辑源
│   │   ├── mod.rs                   # IntelSource trait + build_sources() + SourceGroup
│   │   ├── gdelt.rs ... yfinance.rs # (见 3.2 映射表)
│   │
│   ├── delta/
│   │   ├── mod.rs                   # Delta 变化检测引擎
│   │   └── memory.rs               # 热/冷存储 (SQLite)
│   │
│   └── dashboard/
│       └── mod.rs                   # Axum Web + SSE + include_str! HTML
│
├── static/
│   └── dashboard.html               # 自包含 HUD 仪表板
│
└── runs/                            # 运行时数据 (.gitignore)
    ├── crucix.db
    └── reports/
```

### 3.4 数据源故障容错（Source Groups）

```rust
pub struct SourceGroup {
    pub category: &'static str,
    pub primary: Box<dyn IntelSource>,
    pub fallbacks: Vec<Box<dyn IntelSource>>,
}

// 故障容错组:
// 辐射监测:  Safecast (primary) → EPA RadNet (fallback)
// 航班追踪:  OpenSky (primary) → ADS-B Exchange (fallback, 付费)
// 制裁数据:  OFAC (primary) → OpenSanctions (fallback), 均在 sanctions.rs 内
// 网安威胁:  CVE/NVD (primary) → CISA-KEV (supplementary) → ISC/SANS (supplementary)
// 气象数据:  NOAA (primary) → SWPC (supplementary, 太空天气为独立维度)
```

### 3.5 API 端点 (axum, 版本化)

```
# 本地 + 对外共用 (本地模式绑定 127.0.0.1, 无需认证)
GET  /                         → 本地仪表盘 HTML
GET  /api/v1/data              → 最新扫描数据
GET  /api/v1/health            → 健康检查 (结构化响应, 见 §九)
GET  /api/v1/trends            → 趋势数据
GET  /api/v1/analysis          → LLM 分析结果
GET  /api/v1/sources           → 源列表、状态、可靠性
GET  /api/v1/sse               → SSE 实时推送

# 为 NewsPredict 扩展 (仅 --public 模式, 需 API Key, 有速率限制)
GET  /api/v1/events            → 结构化事件流
GET  /api/v1/correlations      → 跨源关联信号
POST /api/v1/query             → 历史查询 (请求体限制 64KB)
GET  /api/v1/market-seeds      → LLM 生成的市场种子
POST /api/v1/resolve-check     → 条件检查 (LLM 调用预算: 10次/分钟)
```

**公共 API 速率限制** (tower-governor):

| 端点类型 | 限制 | 说明 |
|---------|------|------|
| GET 端点 | 60 req/min per key | 标准读取 |
| POST /query | 20 req/min per key | 查询类 |
| POST /resolve-check | 10 req/min per key | 涉及 LLM 调用, 更严格 |
| SSE 连接 | 5 并发 per key | 长连接资源限制 |

可选 IP 白名单: `--allow-ip 10.0.0.0/8,192.168.1.0/24`

### 3.6 CLI 命令体系

```bash
crucix status                    # 系统状态
crucix sweep [--lang zh|en] [--json] [--no-llm]
crucix source <name>             # 测试单个源
crucix test-llm                  # 测试 LLM 连接
crucix serve [--port 3117] [--public] [--api-key <key>] [--allow-ip <cidr>]
crucix watch [--interval 15]     # 持续监控
crucix history [--limit N] [--show <id>]
crucix trends                    # ASCII 趋势图
crucix report                    # Markdown 报告
crucix export [--format csv|json] [--limit N]
crucix market-seeds [--count 10] # 生成预测市场种子
crucix migrate --from-json <dir> # 从 Node.js JSON 迁移到 SQLite
```

### 3.7 LLM Provider 体系

```rust
#[async_trait]
pub trait LlmProvider: Send + Sync {
    fn name(&self) -> &str;
    fn is_configured(&self) -> bool;
    async fn complete(&self, system: &str, user: &str, opts: &LlmOptions) -> Result<LlmResponse>;
}
```

**提供商矩阵**:

| 实现文件 | Provider | 支持的模型/服务 | 说明 |
|---------|----------|---------------|------|
| `openai_compat.rs` | OpenAI 兼容 | OpenAI, DeepSeek, Moonshot, GLM-4, Qwen, **OpenRouter**, **Mistral**, **MiniMax** | 任何 OpenAI 兼容端点均可通过 `BASE_URL` 切换 |
| `anthropic.rs` | Anthropic | Claude 系列 | 原生 Messages API |
| `gemini.rs` | Gemini | Gemini 系列 | 原生 generateContent API |
| `ollama.rs` | Ollama | 任何本地模型 | 本地 OpenAI 兼容端点 |

> **迁移说明**: Node.js 版的 `codex.mjs`, `minimax.mjs`, `mistral.mjs`, `openrouter.mjs` 在 Rust 版中统一通过 `openai_compat.rs` 覆盖——这些服务均提供 OpenAI 兼容端点，只需设置不同的 `BASE_URL` 和 `API_KEY`。零功能损失。

### 3.8 市场种子生成 (market_seeds.rs)

```rust
pub struct MarketSeed {
    pub id: String,                    // SHA256(question + resolution_criteria + end_time)
    pub question: String,
    pub category: MarketCategory,
    pub options: Vec<String>,          // ["YES", "NO"]
    pub resolution_criteria: String,
    pub resolution_source: String,     // e.g. "yfinance:BTC-USD"
    pub suggested_end_time: DateTime<Utc>,
    pub confidence: f32,
    pub related_sources: Vec<String>,
    pub context: String,
}

pub enum MarketCategory {
    Markets, Politics, Tech, Conflict, Climate, Health, Cyber, Space,
}
```

`id` 由 `SHA256(question + resolution_criteria + suggested_end_time.to_rfc3339())` 生成，保证幂等性——相同问题不会重复创建市场。

---

## 四、NewsPredict 详细设计

### 4.1 技术栈

| 组件 | 技术 | 说明 |
|------|------|------|
| 框架 | Next.js 16 (App Router) | SSR/SSG + API Routes |
| UI | shadcn/ui + Tailwind CSS | 暗色主题卡片式设计 |
| ORM | Drizzle ORM | 类型安全, SQL-first |
| 状态 | Zustand + SWR | 客户端状态 + 数据获取 |
| 认证 | NextAuth v5 | 邮箱 + OAuth + 钱包 (Reown/SIWE) |
| 数据库 | Neon PostgreSQL | Serverless, 可迁移自建 PG |
| 缓存 | Upstash Redis | 会话、Crucix 数据缓存、排行榜 |
| Web3 | wagmi v2 + viem | BSC 链交互 |
| 钱包 | Reown (WalletConnect) | 多链钱包连接 |
| PWA | next-pwa | Service Worker、离线缓存 |
| 错误监控 | Sentry | 前端异常 + API 错误追踪 |
| 部署 | Vercel | 自动部署 |

### 4.2 项目结构

```
newspredict/
├── package.json
├── next.config.ts
├── tailwind.config.ts
├── .env.example
│
├── src/
│   ├── app/
│   │   ├── layout.tsx                 # 根布局 (暗色主题, Providers)
│   │   ├── page.tsx                   # 首页 (For You feed)
│   │   ├── (auth)/
│   │   │   ├── sign-in/page.tsx
│   │   │   └── sign-up/page.tsx
│   │   ├── explore/page.tsx           # 探索/搜索
│   │   ├── markets/
│   │   │   ├── page.tsx              # 市场列表
│   │   │   └── [id]/page.tsx         # 市场详情 + 下注
│   │   ├── portfolio/page.tsx         # 投资组合
│   │   ├── activity/page.tsx          # 动态/通知
│   │   ├── profile/
│   │   │   ├── page.tsx              # 个人主页
│   │   │   └── [userId]/page.tsx     # 他人主页
│   │   ├── leaderboard/page.tsx       # 排行榜
│   │   ├── create/page.tsx            # 创建市场 (UGC)
│   │   │
│   │   └── api/
│   │       ├── auth/[...nextauth]/route.ts
│   │       ├── markets/route.ts
│   │       ├── trades/route.ts        # 交易执行 (含并发控制)
│   │       ├── portfolio/route.ts
│   │       ├── leaderboard/route.ts
│   │       ├── crucix/route.ts        # Crucix 数据代理+缓存
│   │       ├── market-seeds/route.ts
│   │       ├── resolve/route.ts       # 市场解析
│   │       └── web3/route.ts          # 链上交互代理
│   │
│   ├── components/
│   │   ├── ui/                        # shadcn/ui 组件
│   │   ├── layout/
│   │   │   ├── bottom-nav.tsx         # 底部导航 5 tab
│   │   │   ├── top-bar.tsx
│   │   │   └── category-pills.tsx
│   │   ├── cards/
│   │   │   ├── hero-card.tsx          # ~160px 焦点卡
│   │   │   ├── news-prediction-card.tsx # ~100px
│   │   │   ├── video-card.tsx         # ~90px
│   │   │   ├── quick-poll-card.tsx    # ~70px
│   │   │   ├── market-mini-card.tsx   # 110px 趋势迷你卡
│   │   │   ├── resolved-card.tsx
│   │   │   └── breaking-banner.tsx    # ~50px
│   │   ├── market/
│   │   │   ├── order-panel.tsx
│   │   │   ├── price-chart.tsx
│   │   │   ├── donut-gauge.tsx
│   │   │   └── market-stats.tsx
│   │   ├── portfolio/
│   │   │   ├── summary-bar.tsx
│   │   │   └── position-list.tsx
│   │   └── web3/
│   │       ├── connect-button.tsx
│   │       └── chain-badge.tsx
│   │
│   ├── lib/
│   │   ├── crucix-client.ts           # Crucix API 客户端 + Redis 缓存
│   │   ├── market-engine.ts           # LMSR 定价 (含数值稳定性)
│   │   ├── trade-executor.ts          # 交易执行 (含并发控制)
│   │   ├── auth.ts                    # NextAuth 配置
│   │   ├── db.ts                      # Drizzle ORM + Neon
│   │   ├── redis.ts                   # Upstash Redis
│   │   └── web3/
│   │       ├── config.ts              # wagmi + Reown
│   │       ├── contracts.ts           # ABI + 地址
│   │       └── hooks.ts
│   │
│   └── types/
│       ├── market.ts
│       ├── crucix.ts
│       └── user.ts
│
├── drizzle/
│   ├── schema.ts                      # Drizzle schema 定义
│   └── migrations/                    # 自动生成的迁移
│
├── contracts/                         # Solidity (Phase 6)
│   ├── CrucixToken.sol
│   └── PredictionMarket.sol
│
└── public/
    ├── manifest.json                  # PWA manifest
    └── icons/
```

### 4.3 数据库 Schema (PostgreSQL + Drizzle ORM)

```sql
-- 用户
CREATE TABLE users (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email           TEXT UNIQUE,
    name            TEXT,
    avatar_url      TEXT,
    wallet_address  TEXT UNIQUE,
    balance         DECIMAL(18,2) DEFAULT 1000.00 CHECK (balance >= 0),
    total_trades    INTEGER DEFAULT 0,
    wins            INTEGER DEFAULT 0,
    created_at      TIMESTAMPTZ DEFAULT NOW(),
    updated_at      TIMESTAMPTZ DEFAULT NOW()
);

-- 预测市场
CREATE TABLE markets (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    question        TEXT NOT NULL,
    description     TEXT,
    category        TEXT NOT NULL CHECK (category IN (
        'markets','politics','tech','conflict','climate','health','cyber','space'
    )),
    image_url       TEXT,
    status          TEXT DEFAULT 'open' CHECK (status IN (
        'open','closed','resolved','cancelled'
    )),
    creator_id      UUID REFERENCES users(id),
    creator_type    TEXT DEFAULT 'system' CHECK (creator_type IN ('system','user')),
    -- LMSR 状态
    yes_shares      DECIMAL(18,4) DEFAULT 0,      -- 累计 YES 份额
    no_shares       DECIMAL(18,4) DEFAULT 0,       -- 累计 NO 份额
    liquidity_param DECIMAL(8,2) DEFAULT 100.00,   -- LMSR b 参数
    volume          DECIMAL(18,2) DEFAULT 0,
    trader_count    INTEGER DEFAULT 0,
    version         INTEGER DEFAULT 0,             -- 乐观并发控制版本号
    -- 解析
    resolution_criteria TEXT NOT NULL,
    resolution_source   TEXT,
    resolution_result   TEXT CHECK (resolution_result IN ('YES','NO','CANCELLED')),
    resolved_at     TIMESTAMPTZ,
    close_at        TIMESTAMPTZ NOT NULL,
    created_at      TIMESTAMPTZ DEFAULT NOW(),
    -- Crucix 关联
    crucix_seed_id  TEXT UNIQUE,                   -- 对应 MarketSeed.id, UNIQUE 防重复
    related_sources TEXT[],
    tags            TEXT[]
);

-- 交易记录
CREATE TABLE trades (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         UUID NOT NULL REFERENCES users(id),
    market_id       UUID NOT NULL REFERENCES markets(id),
    side            TEXT NOT NULL CHECK (side IN ('YES','NO')),
    shares          DECIMAL(18,4) NOT NULL CHECK (shares > 0),
    price           DECIMAL(8,4) NOT NULL CHECK (price > 0 AND price < 1),
    cost            DECIMAL(18,2) NOT NULL,
    tx_hash         TEXT,                          -- Web3 链上哈希 (Phase 6)
    created_at      TIMESTAMPTZ DEFAULT NOW()
);

-- 用户持仓
CREATE TABLE positions (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         UUID NOT NULL REFERENCES users(id),
    market_id       UUID NOT NULL REFERENCES markets(id),
    side            TEXT NOT NULL CHECK (side IN ('YES','NO')),
    shares          DECIMAL(18,4) NOT NULL CHECK (shares >= 0),
    avg_price       DECIMAL(8,4) NOT NULL,
    realized_pnl    DECIMAL(18,2) DEFAULT 0,
    UNIQUE(user_id, market_id, side)
);

-- 排行榜快照
CREATE TABLE leaderboard_snapshots (
    id              SERIAL PRIMARY KEY,
    period          TEXT NOT NULL CHECK (period IN ('daily','weekly','alltime')),
    rankings        JSONB NOT NULL,
    created_at      TIMESTAMPTZ DEFAULT NOW()
);

-- 市场评论
CREATE TABLE comments (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    market_id       UUID NOT NULL REFERENCES markets(id),
    user_id         UUID NOT NULL REFERENCES users(id),
    content         TEXT NOT NULL CHECK (length(content) BETWEEN 1 AND 2000),
    created_at      TIMESTAMPTZ DEFAULT NOW()
);

-- 用户关注
CREATE TABLE follows (
    follower_id     UUID NOT NULL REFERENCES users(id),
    following_id    UUID NOT NULL REFERENCES users(id),
    created_at      TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (follower_id, following_id),
    CHECK (follower_id != following_id)
);
```

> **注意**: `yes_price` / `no_price` 不存储在数据库中——通过 LMSR 公式从 `yes_shares` + `no_shares` + `liquidity_param` 实时计算，避免冗余数据不一致。

### 4.4 预测市场定价引擎 (LMSR, 含数值稳定性)

```typescript
// lib/market-engine.ts

const DEFAULT_LIQUIDITY = 100;
const MAX_SHARES_PER_SIDE = 50000; // 防溢出上限

/**
 * 数值稳定的 log-sum-exp 计算
 * log(exp(a) + exp(b)) = max(a,b) + log(1 + exp(-|a-b|))
 * 避免 Math.exp() 溢出为 Infinity
 */
function logSumExp(a: number, b: number): number {
  const max = Math.max(a, b);
  return max + Math.log(1 + Math.exp(-Math.abs(a - b)));
}

/**
 * LMSR 成本函数 (数值稳定版)
 */
export function calculateCost(
  currentYesShares: number,
  currentNoShares: number,
  buyYesShares: number,
  buyNoShares: number,
  b: number = DEFAULT_LIQUIDITY
): number {
  // 验证份额上限
  if (currentYesShares + buyYesShares > MAX_SHARES_PER_SIDE ||
      currentNoShares + buyNoShares > MAX_SHARES_PER_SIDE) {
    throw new Error('Market share limit exceeded');
  }

  const before = b * logSumExp(currentYesShares / b, currentNoShares / b);
  const after = b * logSumExp(
    (currentYesShares + buyYesShares) / b,
    (currentNoShares + buyNoShares) / b
  );
  return after - before;
}

/**
 * 当前价格 (数值稳定版)
 */
export function getPrice(
  yesShares: number,
  noShares: number,
  b: number = DEFAULT_LIQUIDITY
): { yes: number; no: number } {
  const diff = (yesShares - noShares) / b;
  const yesPrice = 1 / (1 + Math.exp(-diff)); // sigmoid, 数值稳定
  return {
    yes: yesPrice,
    no: 1 - yesPrice,
  };
}
```

### 4.5 交易执行流程 (含并发控制)

```typescript
// lib/trade-executor.ts

export async function executeTrade(
  userId: string,
  marketId: string,
  side: 'YES' | 'NO',
  amount: number, // 用户愿意花费的金额
  db: DrizzleClient
): Promise<TradeResult> {
  // 在数据库事务中执行，使用乐观并发控制
  return await db.transaction(async (tx) => {
    // 1. 锁定市场行 (SELECT FOR UPDATE)
    const market = await tx.query.markets.findFirst({
      where: eq(markets.id, marketId),
      for: 'update',
    });

    if (!market || market.status !== 'open') {
      throw new Error('Market not available');
    }

    // 2. 检查用户余额
    const user = await tx.query.users.findFirst({
      where: eq(users.id, userId),
      for: 'update',
    });

    if (!user || user.balance < amount) {
      throw new Error('Insufficient balance');
    }

    // 3. 计算 LMSR 份额
    const buyYes = side === 'YES' ? amount : 0;
    const buyNo = side === 'NO' ? amount : 0;
    const cost = calculateCost(
      market.yesShares, market.noShares,
      buyYes, buyNo, market.liquidityParam
    );
    const shares = amount / cost * amount; // 简化: 份额 ≈ 花费金额

    const price = getPrice(
      market.yesShares + buyYes,
      market.noShares + buyNo,
      market.liquidityParam
    );

    // 4. 原子更新: 扣余额 + 更新市场 + 记录交易 + 更新持仓
    await tx.update(users)
      .set({ balance: sql`balance - ${cost}` })
      .where(and(eq(users.id, userId), gte(users.balance, cost))); // 双重检查

    await tx.update(markets).set({
      yesShares: side === 'YES' ? sql`yes_shares + ${shares}` : market.yesShares,
      noShares: side === 'NO' ? sql`no_shares + ${shares}` : market.noShares,
      volume: sql`volume + ${cost}`,
      traderCount: sql`trader_count + 1`,
      version: sql`version + 1`,
    }).where(eq(markets.id, marketId));

    await tx.insert(trades).values({
      userId, marketId, side, shares, price: price[side.toLowerCase()], cost,
    });

    // 5. 更新或创建持仓
    // ... upsert positions ...

    return { success: true, shares, cost, newPrice: price };
  });
}
```

### 4.6 市场解析 (Resolution) 流程

```
Crucix 情报数据  ──→  解析检查器  ──→  市场结果
                      │
                      ├── 自动解析: 价格类 (BTC > $100K?)
                      │   → POST /api/v1/resolve-check 查询 YFinance 数据
                      │   → 自动设置 resolution_result
                      │
                      ├── 半自动解析: 事件类 (选举结果?)
                      │   → 管理员确认 + 多数据源交叉验证
                      │
                      └── 人工解析: UGC 市场
                          → 创建者提交 + 争议仲裁 (7 天申诉期)
```

### 4.7 SIWE (Sign-In With Ethereum) 安全规范

钱包登录遵循 EIP-4361 标准：

1. **服务端生成 nonce**: 每次登录请求生成唯一 nonce，存储在 Redis（TTL 5 分钟）
2. **域绑定验证**: 签名消息中的 `domain` 必须匹配 `newspredict.app`
3. **nonce 一次性消费**: 签名验证成功后立即从 Redis 删除 nonce，防止重放
4. **签名验证**: 使用 viem `verifyMessage()` 验证签名恢复出的地址匹配
5. **会话绑定**: NextAuth session 绑定 wallet address，不可跨钱包复用

### 4.8 PWA 配置

```json
{
  "name": "NewsPredict",
  "short_name": "NP",
  "start_url": "/",
  "display": "standalone",
  "background_color": "#0b1220",
  "theme_color": "#00d4ff",
  "icons": [
    { "src": "/icons/icon-192.png", "sizes": "192x192", "type": "image/png" },
    { "src": "/icons/icon-512.png", "sizes": "512x512", "type": "image/png" }
  ]
}
```

### 4.9 移动端 UI 页面结构

```
┌─────────────────────────────┐
│ Good morning, Alex    Mar 23│  ← 个人化问候
│ Portfolio +$142 │ 3 Active │ Win 68%  ← 组合摘要条
│ [All] [Markets] [Politics] [Tech] ...   ← 分类 pills
├─────────────────────────────┤
│ ■ Top Stories      Updated 2m ago       ← Section 1
│ ┌─────────────────────────┐
│ │ [大图] Tech Earnings... │  ← Hero Card (160px)
│ │ Beat estimates? 71% YES │
│ └─────────────────────────┘
├─────────────────────────────┤
│ ■ Markets                               ← Section 2
│ [Fed Rate][Oil Price][S&P 500]          ← 水平滚动环形图卡
├─────────────────────────────┤
│ ■ Just Resolved                         ← Section 3
│ ✓ Election Result: +$10                 ← Resolved Card
├─────────────────────────────┤
│ ■ Recommended for You                   ← Section 4
│ [AI Regulation] [Climate...]            ← 个性化推荐
├─────────────────────────────┤
│ 🏠 Home  🔍 Explore  ➕  📈 Activity  👤 Profile │
└─────────────────────────────┘
```

---

## 五、Web3 渐进式路线图

### Phase MVP: 虚拟积分制
- 新用户注册赠送 1,000 积分
- 积分用于预测下注，LMSR 自动做市
- 排行榜基于盈利排名
- 无需钱包、无区块链交互

### Phase 2: BSC 代币集成
- 发行 BEP-20 代币 (CRUX)
- BSC Testnet 先行测试
- 用户可用代币参与预测
- 链下订单撮合 + 链上结算

### Phase 3: 全链上预测市场
- PredictionMarket 智能合约管理市场生命周期
- 链上资金托管 + 自动清算
- Chainlink 预言机辅助解析
- 扩展到 Base / Arbitrum / Polygon

---

## 六、部署架构

### Crucix Engine

**方式 A: Docker Compose**
```yaml
services:
  crucix:
    build: .
    ports:
      - "3117:3117"
    volumes:
      - ./runs:/data/runs
    env_file: .env
    working_dir: /data
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "wget", "-q", "--spider", "http://localhost:3117/api/v1/health"]
      interval: 60s
      timeout: 10s
```

**Dockerfile (多阶段构建)**:
```dockerfile
FROM rust:1.82-alpine AS builder
RUN apk add --no-cache musl-dev openssl-dev pkgconf
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src/ src/
COPY static/ static/
RUN cargo build --release

FROM alpine:3.20
RUN apk add --no-cache ca-certificates
COPY --from=builder /app/target/release/crucix /usr/local/bin/
RUN mkdir -p /data/runs
VOLUME /data/runs
ENV DATABASE_PATH=/data/runs/crucix.db
EXPOSE 3117
ENTRYPOINT ["crucix"]
CMD ["serve", "--public", "--port", "3117"]
```

**方式 B: 裸二进制 + Caddy**
```
# Caddyfile
crucix-api.yourdomain.com {
    reverse_proxy localhost:3117
    encode gzip

    # Caddy 自动管理 TLS (Let's Encrypt)
    # 安全头
    header {
        Strict-Transport-Security "max-age=31536000; includeSubDomains"
        X-Content-Type-Options "nosniff"
        X-Frame-Options "DENY"
        Referrer-Policy "strict-origin-when-cross-origin"
    }

    # POST 请求体限制
    request_body {
        max_size 1MB
    }
}
```

### NewsPredict

```
Vercel (自动部署)
├── Next.js 16 App
├── Neon PostgreSQL (Marketplace)
├── Upstash Redis (Marketplace)
├── Sentry (错误监控)
└── 环境变量:
    CRUCIX_API_URL=https://crucix-api.yourdomain.com
    CRUCIX_API_KEY=YOUR_SECRET_KEY
    NEXTAUTH_SECRET=...
    DATABASE_URL=...     (Neon 自动注入)
    REDIS_URL=...        (Upstash 自动注入)
    SENTRY_DSN=...
```

---

## 七、分阶段实施计划

> **时间估算说明**: 以下为乐观估算，基于单人全职开发。实际进度受 API 对接复杂度、测试覆盖、UI 打磨等因素影响，建议在每个 Phase 结束时评估进度并调整后续计划。总体建议预留 30% buffer。

### Phase 1: Crucix Engine 骨架 (2-3 周)

**目标**: Rust 项目能编译运行，3-5 个源可并行采集

```
P1.1  项目初始化: Cargo.toml, main.rs (clap), config.rs, error.rs
P1.2  HTTP 客户端: http.rs (reqwest + 指数退避 + 连接池)
P1.3  IntelSource trait + sources/mod.rs (build_sources, run_source, SourceGroup)
P1.4  5 个 MVP 源: yfinance.rs, usgs.rs, noaa.rs, gdelt.rs, who.rs
P1.5  briefing.rs 编排器 (并行扫描 + 结果聚合)
P1.6  CLI: crucix sweep + crucix source <name> + crucix status
P1.7  集成测试: 并行扫描、超时、错误隔离
```

### Phase 2: 数据源扩展 (3-4 周)

**目标**: 全部 34 逻辑源实现

```
P2.1  Tier 1 剩余核心源 (9 个): acled, sanctions, opensky, firms,
      telegram, swpc, gdacs, safecast, reliefweb, ships, adsb
P2.2  Tier 2 经济源 (7 个): fred, eia, treasury, bls, gscpi, usaspending, comtrade
P2.3  Tier 3 (9 个): epa, cve, isc, cisa_kev, cloudflare_radar,
      reddit, bluesky, patents, kiwisdr
P2.4  Tier 4: celestrak (从 space.mjs 拆分)
P2.5  Source Group failback 逻辑
P2.6  每个源独立测试脚本
```

### Phase 3: 分析引擎 + LLM (2-3 周)

**目标**: Delta 检测 + LLM 分析 + 关联引擎

```
P3.1  store.rs: SQLite schema + CRUD
P3.2  delta/: 变化检测引擎
P3.3  delta/memory.rs: 热/冷存储
P3.4  llm/: LlmProvider trait + openai_compat.rs + ollama.rs
P3.5  llm/: anthropic.rs + gemini.rs
P3.6  correlation.rs: 6 条跨源关联规则
P3.7  watchlist.rs + notify.rs + report.rs
P3.8  CLI: history, trends, report, export, test-llm
P3.9  migrate.rs: JSON → SQLite 迁移工具
```

### Phase 4: Crucix Web + 对外 API (2-3 周)

**目标**: 本地仪表盘 + --public API 模式

```
P4.1  dashboard/mod.rs: Axum 路由 + SSE
P4.2  static/dashboard.html: 自包含 HUD
P4.3  auth.rs: API Key + 速率限制 (tower-governor)
P4.4  对外 API 端点: /api/v1/events, correlations, market-seeds, query, resolve-check
P4.5  llm/market_seeds.rs: 市场种子生成
P4.6  crucix-types/: JSON Schema 契约
P4.7  Docker + Caddyfile
P4.8  端到端测试
```

### Phase 5: NewsPredict MVP (6-8 周)

**目标**: 积分制预测市场 PWA 上线

```
P5.1  Next.js 16 初始化 (shadcn/ui, Tailwind, PWA, Sentry)
P5.2  NextAuth 配置 (邮箱 + Google/GitHub OAuth)
P5.3  Drizzle ORM + Neon PostgreSQL schema + 迁移
P5.4  Crucix API 客户端 + Redis 缓存层
P5.5  首页 Feed: 卡片组件体系
P5.6  市场详情页: 价格图表 + 下注面板
P5.7  LMSR 定价引擎 (含 logSumExp 数值稳定)
P5.8  交易执行器 (SELECT FOR UPDATE 并发控制)
P5.9  用户组合页
P5.10 排行榜 (Redis sorted set)
P5.11 市场创建 (UGC) + 审核
P5.12 Crucix 自动市场种子 → 市场创建
P5.13 市场解析流程 (自动 + 半自动 + 人工)
P5.14 PWA 优化: Service Worker, 离线缓存
P5.15 Vercel 部署 + Neon + Upstash 集成
```

### Phase 6: Web3 集成 (4-6 周)

**目标**: BSC 代币 + 链上交互

```
P6.1  Reown (WalletConnect) 钱包连接
P6.2  SIWE 钱包登录 (EIP-4361, 含 nonce/域验证/重放防护)
P6.3  CrucixToken.sol (BEP-20) + BSC Testnet
P6.4  wagmi/viem: 代币余额、转账
P6.5  PredictionMarket.sol 合约
P6.6  链下撮合 + 链上结算
P6.7  BSC Mainnet 部署
P6.8  多链扩展 (合约抽象层)
```

---

## 八、关键约束清单

### Crucix Engine 约束
- 不引入 HTML 解析库 — 自实现 tag stripper
- 不引入 LLM SDK — 纯 reqwest HTTP
- 不引入 dotenv crate — 自实现 .env 加载
- SQLite 必须 bundled — 编译进二进制
- 仪表盘 HTML 必须 include_str! — 嵌入二进制
- 每个数据源一个文件 — 独立可测
- 所有源输出 JSON 必须含 `"source": "SourceName"`
- 本地模式绑定 127.0.0.1 — 安全默认
- 公共 API 必须有速率限制 — tower-governor
- API 使用 /api/v1/ 版本化 — 向后兼容策略

### NewsPredict 约束
- 移动端优先 (Mobile First)
- 暗色主题 — zinc/neutral 色调，强调色 #00d4ff
- PWA 标准 — manifest, Service Worker, 离线兜底
- 积分制先行 — Web3 不阻塞 MVP
- Crucix 数据缓存到 Redis — 不直接暴露 Crucix 给前端用户
- 交易执行必须在数据库事务中 — SELECT FOR UPDATE 防并发
- 用户余额不可为负 — CHECK (balance >= 0) + 双重检查
- 所有用户输入 HTML 转义 — 防 XSS
- 评论长度限制 2000 字符
- SIWE 登录遵循 EIP-4361 — nonce 一次性消费 + 域绑定

---

## 九、可观测性

### Crucix Engine

**结构化日志** (JSON, stdout):
```json
{"ts":"2026-03-23T10:00:00Z","level":"info","module":"sweep","msg":"Sweep complete","sources_ok":30,"sources_err":4,"duration_ms":12340}
```

**健康检查响应** (`GET /api/v1/health`):
```json
{
  "status": "ok",
  "uptime_seconds": 86400,
  "last_sweep": "2026-03-23T10:00:00Z",
  "next_sweep": "2026-03-23T10:15:00Z",
  "sources": { "ok": 30, "error": 3, "timeout": 1 },
  "llm": { "provider": "ollama", "model": "qwen3.5:9b", "configured": true },
  "database": { "path": "runs/crucix.db", "size_mb": 42 },
  "degraded_sources": ["OpenSky", "Reddit"]
}
```

**关键指标**:
- sweep_duration_ms (每次扫描耗时)
- source_success_rate (各源成功率)
- llm_latency_ms (LLM 调用延迟)
- sse_active_connections (SSE 连接数)
- api_requests_per_minute (API 请求频率)

### NewsPredict

- **Sentry**: 前端异常 + API Route 错误 + 性能 traces
- **Vercel Analytics**: Core Web Vitals, 页面性能
- **关键业务指标**: 交易量, 活跃市场数, DAU, 市场解析正确率
- **Redis 监控**: 缓存命中率, 排行榜更新频率
