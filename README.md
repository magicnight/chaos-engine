> 中文 | **[English](README.en.md)**

<div align="center">

# C.H.A.O.S.

**Connected Human-Augmented OSINT Suite**

*混沌中见秩序，冲击前获洞察*

```
   ██████╗ ██╗  ██╗  █████╗   ██████╗  ███████╗
  ██╔════╝ ██║  ██║ ██╔══██╗ ██╔═══██╗ ██╔════╝
  ██║      ███████║ ███████║ ██║   ██║ ███████╗
  ██║      ██╔══██║ ██╔══██║ ██║   ██║ ╚════██║
  ╚██████╗ ██║  ██║ ██║  ██║ ╚██████╔╝ ███████║
   ╚═════╝ ╚═╝  ╚═╝ ╚═╝  ╚═╝  ╚═════╝  ╚══════╝
```

[![Rust](https://img.shields.io/badge/rust-stable-orange)](https://www.rust-lang.org/)
[![License: AGPL v3](https://img.shields.io/badge/license-AGPLv3-blue.svg)](LICENSE)
[![Sources](https://img.shields.io/badge/OSINT%20数据源-46-cyan)](#数据源-46)
[![LLM](https://img.shields.io/badge/LLM-多供应商-green)](#ai-分析)
[![Docker](https://img.shields.io/badge/docker-ready-2496ED)](#部署)

</div>

---

## 什么是 CHAOS？

CHAOS 并行采集 **46 个开源情报数据源** — 卫星火灾检测、航班追踪、核辐射监测、地震数据、经济指标、武装冲突事件、网络安全漏洞、制裁名单、疾病爆发、社交舆情等 — 每 15 分钟自动合成为一幅可操作的全局情报画面。

接入 LLM 后，它成为一个**多语言情报分析师**，生成英文/中文/日文/西班牙文结构化简报，具备跨域关联检测和异常标记。告警推送到 Telegram 和 Discord 机器人，支持三级严重度分类（FLASH / PRIORITY / ROUTINE），机器人还接受命令 — 按需扫描、请求简报、检查状态，全在手机上完成。

一切都渲染在一个自包含的 **Jarvis 风格仪表板**上，包含 3D 地球、可拖拽 GridStack 面板、实时 SSE 更新和完整 API。单一二进制。内嵌 SQLite。零云依赖。零遥测。零订阅。

---

## 快速开始

```bash
git clone https://github.com/magicnight/chaos-engine.git && cd chaos-engine
cargo build --release
cp .env.example .env          # 编辑 API 密钥（可选）
./target/release/chaos serve
```

仪表板地址 `http://localhost:3117`，首次扫描约 30 秒完成。

### 容器部署（推荐）

```bash
cp .env.example .env          # 配置
podman-compose up -d          # 或 docker compose up -d
```

---

## 功能特性

### 情报采集（46 个数据源）

所有数据源通过 `tokio::join_all` 并行运行，按 Tier 差异化超时（T1: 30s, T2: 24s, T3: 15s）。20+ 个数据源**无需任何 API 密钥**即可运行。

| 层级 | 方向 | 数量 | 数据源 |
|------|------|-----:|--------|
| **T1** 核心 OSINT | 冲突、灾害、卫生、交通 | 16 | ACLED, ADS-B, FIRMS, GDACS, GDELT, OpenSky, ProMED, ReliefWeb, Safecast, Sanctions, Ships, SWPC, Telegram, Tsunami, USGS, WHO |
| **T2** 经济金融 | 市场、贸易、财政 | 11 | BLS, CoinGecko, Comtrade, ECB, EIA, ExchangeRates, FRED, GSCPI, Treasury, USAspending, WorldNews |
| **T3** 补充数据 | 网络、环境、社交、科技 | 16 | Bluesky, CISA-KEV, Cloudflare Radar, Copernicus, CVE/NVD, EPA RadNet, EU Sanctions, Google Trends, ISC/SANS, KiwiSDR, NASA NEO, NOAA, NTSB, Patents, Reddit, RIPE Atlas, TechStatus |
| **T4** 太空 | 轨道追踪 | 1 | CelesTrak |
| **T5** 市场 | 实时行情 | 1 | Yahoo Finance |

### AI 分析

- **LLM 降级链**：主供应商 → 备用供应商 → 本地 Ollama（自动故障转移，60s 超时保护）
- **10 个供应商后端**：OpenAI、Anthropic、Gemini、Ollama、DeepSeek、Moonshot、OpenRouter、Mistral、MiniMax、智谱AI
- **4 语言分析**：`--lang en|zh|ja|es` — 完整军事风格简报模板
- **结构化输出**：态势概览、关键发展、风险矩阵、可操作情报、跨域关联
- **预测市场种子**：17 条规则 + LLM 生成 + 7 个模板兜底，保证每次扫描 ≥5 个种子

### CHAOS MONITOR 仪表板

- **22 个可拖拽面板**，分 9 个类别，全部可在设置中开关
- **GridStack.js** 面板系统，支持拖拽、调整大小、布局持久化
- **3D 地球**，实时事件标注（地震、火灾、冲突、天气）
- **Server-Sent Events** 实时数据流
- **公共 API 模式**，支持 API Key 认证和速率限制
- **内嵌二进制** via `include_str!` — 无需外部静态文件

| 类别 | 面板 |
|------|------|
| 态势 | 态势地图、运输与空域 |
| 金融 | 市场数据、风险仪表、能源与宏观、全球经济 |
| 安全 | 冲突事件、OSINT 信息流、制裁监控 |
| 新闻 | 新闻聚合、趋势与创新 |
| 自然 | 地震监控、核辐射监控、气候与环境 |
| 网络 | 网络威胁、网络情报 |
| 太空 | 太空监控、近地天体追踪 |
| 系统 | 数据源健康、变化检测 |
| AI | AI 情报简报、跨源关联信号 |

### 多级告警

| 级别 | 标签 | 触发条件 | 冷却 | 每小时上限 |
|------|------|---------|------|-----------|
| FLASH | 立即行动 | 2+ 关键信号或 5+ 关键变化 | 5 分钟 | 6 |
| PRIORITY | 数小时内处理 | 1 关键或 2+ 高严重度信号 | 30 分钟 | 4 |
| ROUTINE | 信息通报 | 3+ 总变化或 2+ 新信号 | 60 分钟 | 2 |

- **Telegram 机器人**：双向 — 接收命令（`/status`, `/sweep`, `/brief`, `/mute`, `/help`），发送分级告警
- **Discord 机器人**：Webhook 模式或完整 Bot Token 模式，支持富嵌入和颜色分级
- **桌面通知**：Windows、macOS、Linux 原生提示
- **Webhook**：兼容 Slack / Discord / 飞书

### 历史分析

- **Delta 引擎**：25 个追踪指标（14 数值 + 11 计数），严重度评分
- **6 条关联规则**：自然灾害级联、地缘政治风险汇聚、网络威胁汇聚、基础设施压力、市场恐慌、人道主义危机
- **异常检测**：对历史扫描数据做 z-score 分析，标记偏差 > 2σ
- **风险方向**：自动分类（risk-on / risk-off / 混合）

---

## CLI 命令

| 命令 | 说明 |
|------|------|
| `chaos status` | 引擎状态、配置、数据源可用性、LLM 连接 |
| `chaos sweep` | 对所有 46 个数据源执行完整情报扫描 |
| `chaos sweep --json` | JSON 输出，可管道传输到其他工具 |
| `chaos sweep --lang zh` | 中文 LLM 分析 |
| `chaos sweep --no-llm` | 跳过 LLM 分析 |
| `chaos serve` | 启动 Web 仪表板 (localhost:3117) |
| `chaos serve --public --api-key KEY` | 公共 API 模式（绑定 0.0.0.0，限速） |
| `chaos source <name>` | 测试单个数据源 |
| `chaos test-llm` | 测试 LLM 连接 |
| `chaos history` | 查看扫描历史 |
| `chaos trends` | 所有追踪指标的 ASCII 迷你图 |
| `chaos trends --anomalies` | 标记统计异常值 |
| `chaos report` | 生成 Markdown 情报报告 |
| `chaos export --format csv` | 导出趋势数据为 CSV |

---

## 架构

```
                        ┌─────────────────────────────────┐
                        │         CLI (clap v4)           │
                        │  status│sweep│serve│watch│...   │
                        └──────────────┬──────────────────┘
                                       │
              ┌────────────────────────┼────────────────────────┐
              │                        │                        │
     ┌────────▼────────┐    ┌─────────▼─────────┐   ┌─────────▼─────────┐
     │   46 数据源       │    │   LLM 降级链       │   │   仪表板            │
     │  (异步并行)       │    │                    │   │   (Axum + SSE)    │
     │                  │    │  OpenAI 兼容        │   │                   │
     │  T1: 核心 OSINT  │    │  Anthropic         │   │  GridStack 面板    │
     │  T2: 经济金融     │    │  Gemini            │   │  3D 地球           │
     │  T3: 补充数据     │    │  Ollama (本地)      │   │  实时 SSE          │
     └────────┬────────┘    └─────────┬─────────┘   └─────────┬─────────┘
              │                        │                        │
              └────────────────────────┼────────────────────────┘
                                       │
                        ┌──────────────▼──────────────────┐
                        │           核心引擎                │
                        │  简报 ─ Delta ─ 关联 ─ 异常检测   │
                        │  SQLite 存储 ─ 关注列表           │
                        └──────────────┬──────────────────┘
                                       │
              ┌────────────────────────┼────────────────────────┐
              │                        │                        │
     ┌────────▼────────┐    ┌─────────▼─────────┐   ┌─────────▼─────────┐
     │  Telegram 机器人  │    │  Discord 机器人    │   │   通知推送          │
     │  (双向交互)       │    │  (Webhook/Bot)    │   │  桌面/Slack/飞书    │
     └─────────────────┘    └───────────────────┘   └───────────────────┘
```

---

## 部署

### 生产环境（Podman / Docker Compose）

全栈部署：CHAOS 引擎 + NewsPredict 预测市场 + PostgreSQL + Caddy 反向代理。

```bash
git clone https://github.com/magicnight/chaos-engine.git && cd chaos-engine
cp .env.example .env              # 配置（见下方）
podman-compose up -d              # 或 docker compose up -d
```

启动 4 个服务：

```
互联网 → Caddy (:80/:443, 自动 HTTPS)
            ├─ /api/v1/*  → CHAOS 引擎 (:3117)
            └─ /*         → NewsPredict (:3000)
                               └─ PostgreSQL (:5432)
```

**有域名**（自动 HTTPS）：
```bash
# .env 中设置：
DOMAIN=chaos.yourdomain.com
# Caddy 自动申请 Let's Encrypt 证书
```

**无域名**（仅 HTTP）：
```bash
# .env 中 DOMAIN 留空 — 通过 http://服务器IP 访问
```

### 仅部署 CHAOS 引擎（不含前端）

```bash
cargo build --release
./target/release/chaos serve --public --api-key YOUR_SECRET --port 3117
```

### 配置说明

复制 `.env.example` 为 `.env` 并配置。所有变量均为可选（除特别标注外）。

#### 核心配置

| 变量 | 默认值 | 说明 |
|------|--------|------|
| `DOMAIN` | *（空）* | 域名，用于自动 HTTPS |
| `POSTGRES_PASSWORD` | `chaos_secret` | PostgreSQL 密码 — **生产环境必须修改** |
| `REFRESH_INTERVAL_MINUTES` | `15` | OSINT 扫描间隔（分钟） |
| `SOURCE_TIMEOUT_SECS` | `30` | 数据源超时（T1: 100%, T2: 80%, T3: 50%） |

#### LLM 配置（可选 — 启用 AI 分析和更丰富的预测市场种子）

| 变量 | 示例 | 说明 |
|------|------|------|
| `LLM_PROVIDER` | `openai` | 主供应商：openai, anthropic, gemini, ollama, deepseek, zhipuai, openrouter, mistral, minimax |
| `LLM_API_KEY` | | 主供应商 API 密钥 |
| `DEFAULT_MODEL` | `gpt-4o` | 模型名称 |
| `SWEEP_LANG` | `en` | 简报语言：en, zh, ja, es |
| `FALLBACK_PROVIDER` | `gemini` | 备用供应商（自动故障转移） |
| `OLLAMA_URL` | `http://localhost:11434` | 本地 Ollama 地址（零云端降级） |

#### 数据源 API 密钥（可选 — 每多一个密钥，解锁更多数据）

| 变量 | 数据源 | 免费？ |
|------|--------|:------:|
| `FRED_API_KEY` | 美联储经济数据 | 是 |
| `FIRMS_MAP_KEY` | NASA 火灾检测 | 是 |
| `EIA_API_KEY` | 美国能源信息 | 是 |
| `WORLDNEWS_API_KEY` | 全球新闻 API | 是 |
| `ACLED_EMAIL` + `ACLED_PASSWORD` | 武装冲突数据 | 是 |

20+ 个数据源**零 API 密钥**即可运行。

#### NewsPredict 配置

| 变量 | 说明 |
|------|------|
| `NEXTAUTH_SECRET` | **必填** — 会话签名随机密钥 |
| `CRON_SECRET` | 自动种子/解决 API 调用密钥 |
| `NEXT_PUBLIC_CHAOS_URL` | 公共 CHAOS API 地址（客户端 SSE 用） |

#### 机器人（可选）

| 变量 | 说明 |
|------|------|
| `TELEGRAM_BOT_TOKEN` + `TELEGRAM_CHAT_ID` | Telegram 告警 + 命令 |
| `DISCORD_BOT_TOKEN` + `DISCORD_CHANNEL_ID` | Discord Bot 模式 |
| `DISCORD_WEBHOOK_URL` | Discord Webhook 模式（更简单） |
| `WEBHOOK_URL` | 通用 Webhook（Slack/飞书） |

---

## API

仪表板提供 RESTful API + SSE 流式推送。公共模式（`--public --api-key KEY`）下所有扩展端点需要认证。

| 端点 | 方法 | 认证 | 说明 |
|------|------|:----:|------|
| `/api/v1/data` | GET | 否 | 最新扫描数据（所有源、delta、关联、分析） |
| `/api/v1/health` | GET | 否 | 健康检查（运行时间、LLM、数据库、降级源） |
| `/api/v1/trends` | GET | 否 | 历史扫描数据（最近 50 次） |
| `/api/v1/analysis` | GET | 否 | 最新 LLM 情报简报 |
| `/api/v1/sources` | GET | 否 | 数据源列表（层级、描述、可靠性） |
| `/api/v1/sse` | GET | 否 | Server-Sent Events（实时更新） |
| `/api/v1/events` | GET | 公共 | 结构化事件（分类 + 地理标签） |
| `/api/v1/correlations` | GET | 公共 | 跨源关联信号 |
| `/api/v1/market-seeds` | GET | 公共 | 预测市场种子问题 |
| `/api/v1/query` | POST | 公共 | 查询历史数据 |
| `/api/v1/resolve-check` | POST | 公共 | 检查条件是否满足 |

完整规格：[`docs/api.md`](docs/api.md) | [`docs/openapi.yaml`](docs/openapi.yaml)

---

## NewsPredict 预测市场

配套的预测市场 PWA，位于 [`newspredict/`](newspredict/)。基于 Next.js 构建，使用 LMSR 评分、Web3 钱包集成（BSC）和 Drizzle ORM。从 CHAOS 公共 API 消费实时情报数据，自动生成和解决预测市场。

- **经济模型**：[`docs/economics-zh.md`](docs/economics-zh.md) | [`docs/economics.md`](docs/economics.md)
- **智能合约**：ChaosToken (C.H.A.O.S.) + ChaosPredictionMarket — 已部署在 BSC 主网和测试网，源码在 BscScan 已验证

### BSC 主网合约地址

| 合约 | 地址 |
|------|------|
| ChaosToken | [`0xcE3fbb08D72BEd7F645F59FE0f031659b5B298c4`](https://bscscan.com/address/0xcE3fbb08D72BEd7F645F59FE0f031659b5B298c4) |
| ChaosPredictionMarket | [`0xAa7208Cf64078756fB58698fbE748DC3c9b4Cb88`](https://bscscan.com/address/0xAa7208Cf64078756fB58698fbE748DC3c9b4Cb88) |

---

## 添加自定义数据源

CHAOS 使用插件化数据源架构。每个数据源实现 `IntelSource` trait：

```rust
#[async_trait]
pub trait IntelSource: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn tier(&self) -> u8;           // 1-5
    async fn sweep(&self) -> Result<Value>;
}
```

添加步骤：
1. 在 `src/sources/` 创建新文件，实现 `IntelSource`
2. 在 `src/sources/mod.rs` 的 `build_sources()` 中注册
3. 完成 — 简报引擎、CLI、仪表板、Delta 引擎全部自动适配

模板文件：`src/sources/_template.rs`。详见 [`docs/source-plugin-guide.md`](docs/source-plugin-guide.md)。

---

## 许可证

[AGPL v3](LICENSE)

---

*Copyright (c) 2026 ChaosDevOps@BKK&Estonia. All rights reserved.*
