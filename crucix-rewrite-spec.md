# Crucix 本地化 OSINT 情报引擎重构方案

> **用途说明**：本文档是提供给 Claude Code 的完整技术规格书，用于指导从零开始构建一个类 Crucix 的本地化 OSINT（开源情报）聚合与分析系统。基于对原版 JavaScript Crucix（calesthio/Crucix）和 Rust 重写版（coder-brzhang/rust-crucix）三次迭代的深度分析编写。

---

## 一、项目背景与设计哲学

### 1.1 什么是 Crucix

Crucix 是一个开源情报（OSINT）聚合系统，能从多个公开数据源实时抓取全球情报（卫星火灾检测、航班追踪、核辐射监测、经济指标、武装冲突、网络安全漏洞等），通过 LLM 合成统一的情报简报，并提供 Web 仪表板实时展示。

### 1.2 核心设计原则

这些原则贯穿整个架构，**所有技术决策都必须服从这些原则**：

1. **零云依赖（Zero Cloud）**：支持完全本地运行，包括 LLM 分析（通过 Ollama）
2. **单二进制分发**：一个可执行文件包含一切——Web 服务器、仪表板 HTML、数据库引擎
3. **渐进式增强**：零 API 密钥即可运行（18+ 免费数据源），每增加一个密钥解锁更多功能
4. **故障隔离**：单个数据源失败不影响其他源，LLM 故障自动回退规则引擎
5. **极简依赖**：不引入不必要的框架或 SDK

### 1.3 技术栈选型（Rust 重写版，推荐方案）

| 组件 | 技术选择 | 说明 |
|------|---------|------|
| 语言 | **Rust** (edition 2021) | 零 GC、真并发、单二进制 |
| 异步运行时 | **tokio** (multi-thread) | 所有数据源并行抓取 |
| HTTP 客户端 | **reqwest** (with rustls) | 连接池复用、指数退避重试 |
| Web 框架 | **axum** | SSE 实时推送、API 端点 |
| 序列化 | **serde + serde_json** | JSON 处理核心 |
| 数据库 | **rusqlite** (bundled feature) | SQLite 编译进二进制，零外部依赖 |
| CLI | **clap** (derive) | 结构化命令行，自动帮助/补全 |
| 错误处理 | **thiserror + anyhow** | 类型安全错误链 |
| 时间处理 | **chrono** | UTC 时间戳、日期计算 |
| HTML 解析 | **自实现 tag stripper** | 不依赖外部 HTML 解析库 |
| LLM 集成 | **原生 fetch（reqwest）** | 不使用任何厂商 SDK |

### 1.4 Cargo.toml 核心依赖

```toml
[dependencies]
tokio = { version = "1", features = ["full"] }
reqwest = { version = "0.12", features = ["json", "rustls-tls"], default-features = false }
axum = { version = "0.8", features = ["ws"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
rusqlite = { version = "0.32", features = ["bundled"] }
clap = { version = "4", features = ["derive"] }
thiserror = "2"
anyhow = "1"
chrono = { version = "0.4", features = ["serde"] }
async-trait = "0.1"
tower-http = { version = "0.6", features = ["cors"] }
```

---

## 二、项目目录结构（v3 完整版，19 源 + 7000 行）

```
crucix/
├── Cargo.toml
├── Dockerfile                    # node:22-alpine → rust:alpine 多阶段构建
├── docker-compose.yml
├── .env.example                  # 所有可配置环境变量模板
│
├── src/
│   ├── main.rs                   # CLI 入口，clap derive，命令分发
│   ├── config.rs                 # .env 配置加载（自实现，无 dotenv）
│   ├── error.rs                  # thiserror 统一错误类型
│   ├── http.rs                   # reqwest 客户端 + 指数退避重试 + 连接池
│   ├── briefing.rs               # 情报编排器：并行扫描 → Delta → LLM → 告警 → SSE
│   ├── store.rs                  # SQLite 持久化（sweeps/analyses/source_health 三张表）
│   ├── correlation.rs            # 跨源关联引擎（6 条规则）
│   ├── notify.rs                 # 告警推送（Webhook + 桌面通知）
│   ├── watchlist.rs              # 用户自定义关注过滤
│   ├── report.rs                 # Markdown 报告 / CSV 导出
│   │
│   ├── llm/
│   │   ├── mod.rs                # LlmProvider trait 抽象 + 工厂函数
│   │   ├── openai.rs             # OpenAI 兼容实现（覆盖 OpenAI/DeepSeek/Moonshot/GLM-4/Qwen）
│   │   └── ollama.rs             # 本地 Ollama（真正零云端）
│   │
│   ├── sources/
│   │   ├── mod.rs                # IntelSource trait 定义 + build_sources() 注册 + run_source()
│   │   ├── gdelt.rs              # [T1] 全球新闻（100+ 语言）
│   │   ├── acled.rs              # [T1] 武装冲突事件
│   │   ├── who.rs                # [T1] WHO 疫情爆发
│   │   ├── sanctions.rs          # [T1] 全球制裁数据库（OFAC SDN）
│   │   ├── opensky.rs            # [T1] 实时航班追踪（ADS-B）
│   │   ├── firms.rs              # [T1] NASA 卫星热点/火灾检测
│   │   ├── telegram.rs           # [T1] Telegram OSINT 频道（HTML 爬取，无 Bot API）
│   │   ├── usgs.rs               # [T1] USGS 地震（M2.5+）
│   │   ├── swpc.rs               # [T1] NOAA 太空天气（R/S/G 等级）
│   │   ├── gdacs.rs              # [T1] 联合国全球灾害预警
│   │   ├── fred.rs               # [T2] 美联储经济指标（需免费 Key）
│   │   ├── eia.rs                # [T2] 美国能源情报署
│   │   ├── cve.rs                # [T2] NVD/CVE 漏洞情报
│   │   ├── isc.rs                # [T2] ISC/SANS 网络威胁等级
│   │   ├── noaa.rs               # [T3] 美国气象预警
│   │   ├── reddit.rs             # [T3] 社交情绪（需 OAuth）
│   │   ├── uspto.rs              # [T3] 专利授权追踪（PatentsView API）
│   │   ├── celestrak.rs          # [T4] CelesTrak 卫星轨道追踪
│   │   └── yfinance.rs           # [T5] 金融市场行情（SPY/BTC/VIX/黄金/原油）
│   │
│   ├── delta/
│   │   ├── mod.rs                # Delta 变化检测引擎：阈值评分 + 语义去重
│   │   └── memory.rs             # 热/冷内存管理（从 SQLite 读取历史数据）
│   │
│   └── dashboard/
│       └── mod.rs                # Axum Web 服务器 + SSE + 静态 HTML 嵌入
│
├── static/
│   └── dashboard.html            # 自包含 HUD 仪表板（D3.js 3D 地球 + Canvas 趋势图）
│
└── runs/                         # 运行时数据（.gitignore）
    ├── crucix.db                 # SQLite 数据库
    └── reports/                  # 导出的报告
```

---

## 三、核心抽象：Trait 驱动的数据源系统

### 3.1 IntelSource Trait（最关键的抽象）

```rust
use async_trait::async_trait;
use anyhow::Result;
use serde_json::Value;

#[async_trait]
pub trait IntelSource: Send + Sync {
    /// 源标识符（如 "gdelt", "acled"）
    fn name(&self) -> &str;
    
    /// 人类可读描述
    fn description(&self) -> &str;
    
    /// 情报层级 1-5（1=最高优先级）
    fn tier(&self) -> u8;
    
    /// 执行一次数据抓取，返回结构化 JSON
    async fn sweep(&self) -> Result<Value>;
}
```

**添加新数据源的步骤（仅需 3 步）**：

1. 在 `sources/` 目录创建新文件，实现 `IntelSource` trait
2. 在 `sources/mod.rs` 的 `build_sources()` 中注册
3. 完成——编排器、CLI、仪表板、Delta 引擎全部自动适配

### 3.2 数据源执行模型

```rust
use tokio::time::{timeout, Duration, Instant};
use futures::future::join_all;

pub struct SourceResult {
    pub name: String,
    pub status: SourceStatus,      // Ok | Error | Timeout
    pub data: Option<Value>,
    pub error: Option<String>,
    pub duration_ms: u64,
    pub tier: u8,
}

pub enum SourceStatus { Ok, Error, Timeout }

/// 执行单个源，带独立超时
pub async fn run_source(source: &dyn IntelSource, max_timeout: Duration) -> SourceResult {
    let start = Instant::now();
    let result = timeout(max_timeout, source.sweep()).await;
    let duration_ms = start.elapsed().as_millis() as u64;
    
    match result {
        Ok(Ok(data)) => SourceResult {
            name: source.name().to_string(),
            status: SourceStatus::Ok,
            data: Some(data),
            error: None,
            duration_ms,
            tier: source.tier(),
        },
        Ok(Err(e)) => SourceResult {
            name: source.name().to_string(),
            status: SourceStatus::Error,
            data: None,
            error: Some(e.to_string()),
            duration_ms,
            tier: source.tier(),
        },
        Err(_) => SourceResult {
            name: source.name().to_string(),
            status: SourceStatus::Timeout,
            data: None,
            error: Some("Timeout".to_string()),
            duration_ms,
            tier: source.tier(),
        },
    }
}

/// 并行执行所有源
pub async fn run_sweep(timeout_duration: Duration) -> (Vec<SourceResult>, Duration) {
    let sources = build_sources();
    let start = Instant::now();
    
    let futures: Vec<_> = sources.iter()
        .map(|src| run_source(src.as_ref(), timeout_duration))
        .collect();
    let results = join_all(futures).await;
    
    (results, start.elapsed())
}
```

关键点：使用 `join_all` 而非 `try_join_all`——等价于 JS 的 `Promise.allSettled()`，确保单个源失败不中断整个扫描。

---

## 四、19 个数据源完整规格

### 4.1 数据源总览表

| Tier | 源名称 | 文件 | API 端点 | 认证 | 刷新频率 |
|------|--------|------|---------|------|---------|
| T1 | GDELT | gdelt.rs | `api.gdeltproject.org/api/v2/doc/doc` | 无需 | 15min |
| T1 | ACLED | acled.rs | `acleddata.com/acled/curated/v1` | 免费 OAuth2 | 15min |
| T1 | WHO | who.rs | `www.who.int/feeds/entity/don/en/rss.xml` | 无需 | 15min |
| T1 | Sanctions | sanctions.rs | `sanctionslist.ofac.treas.gov/api/...` | 无需 | 15min |
| T1 | OpenSky | opensky.rs | `opensky-network.org/api/states/all` | 无需（限速） | 15min |
| T1 | FIRMS | firms.rs | `firms.modaps.eosdis.nasa.gov/api/...` | 免费 Key | 15min |
| T1 | Telegram | telegram.rs | `t.me/s/{channel}` HTML 爬取 | **无需** | 15min |
| T1 | USGS 地震 | usgs.rs | `earthquake.usgs.gov/earthquakes/feed/v1.0/summary/2.5_day.geojson` | 无需 | 15min |
| T1 | 太空天气 | swpc.rs | `services.swpc.noaa.gov/products/noaa-scales.json` | 无需 | 15min |
| T1 | GDACS | gdacs.rs | `www.gdacs.org/xml/rss.xml` | 无需 | 15min |
| T2 | FRED | fred.rs | `api.stlouisfed.org/fred/series/observations` | 免费 Key | 15min |
| T2 | EIA | eia.rs | `api.eia.gov/v2/petroleum/...` | DEMO_KEY | 15min |
| T2 | CVE/NVD | cve.rs | `services.nvd.nist.gov/rest/json/cves/2.0` | 可选免费 Key | 15min |
| T2 | ISC/SANS | isc.rs | `isc.sans.edu/api/infocon?json` | 无需 | 15min |
| T3 | NOAA 天气 | noaa.rs | `api.weather.gov/alerts/active` | 无需 | 15min |
| T3 | Reddit | reddit.rs | `oauth.reddit.com/r/...` | OAuth | 15min |
| T3 | USPTO | uspto.rs | `api.patentsview.org/patents/query` | 无需 | 15min |
| T4 | CelesTrak | celestrak.rs | `celestrak.org/NORAD/elements/gp.php` | 无需 | 15min |
| T5 | YFinance | yfinance.rs | `query1.finance.yahoo.com/v8/finance/chart/...` | 无需 | 15min |

### 4.2 无需任何认证的数据源（零配置即用，15 个）

以下数据源 `git clone` 后即可使用，无需注册任何 API Key：

**GDELT**, **WHO**, **Sanctions/OFAC**, **OpenSky**, **Telegram**（HTML 爬取）, **USGS 地震**, **太空天气 SWPC**, **GDACS 灾害**, **EIA**（用 DEMO_KEY）, **ISC/SANS**, **NOAA 天气**, **USPTO/PatentsView**, **CelesTrak**, **YFinance**, **CVE/NVD**（无 Key 限 5 请求/30 秒）

### 4.3 重点数据源实现细节

#### Telegram OSINT 频道（关键创新：无需 Bot API）

```rust
// 核心思路：通过 t.me/s/{channel} 公开 Web 预览页面爬取
// 不需要 Bot Token、不需要注册、不需要认证

const OSINT_CHANNELS: &[&str] = &[
    "intelslava",        // Intel Slava Z - 冲突实时更新
    "ryaborig",          // Rybar - 军事态势分析
    "SputnikInt",        // Sputnik International
    "nexaborig",         // NEXTA - 东欧动态
    "UkraineNow",        // Ukraine NOW (EN)
    "OSINTdefender",     // OSINT defender
    "MilitaryBBa",       // BBa OSINT 综合
    "liveuamap",         // LiveMap 冲突地图
];

async fn fetch_channel(&self, channel: &str) -> Result<Value> {
    let url = format!("https://t.me/s/{}", channel);
    let html = self.client.fetch_text(&url).await?;
    let posts = parse_telegram_posts(&html, channel);
    Ok(json!(posts))
}

// HTML 解析：提取 tgme_widget_message_text 内容
// 紧急度检测：包含以下关键词标记为 urgent
fn is_urgent(text: &str) -> bool {
    let lower = text.to_lowercase();
    lower.contains("breaking") || lower.contains("urgent") 
        || lower.contains("missile") || lower.contains("attack")
        || lower.contains("explosion") || lower.contains("nuclear")
}

// 自实现 HTML tag stripper（零依赖）
fn strip_html_tags(html: &str) -> String {
    let mut result = String::with_capacity(html.len());
    let mut in_tag = false;
    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => result.push(ch),
            _ => {}
        }
    }
    result.replace("&amp;", "&").replace("&lt;", "<")
        .replace("&gt;", ">").replace("&quot;", "\"")
}
```

输出格式：
```json
{
  "source": "Telegram",
  "channelsMonitored": 8,
  "channelsReachable": 7,
  "totalPosts": 35,
  "urgent": [
    { "channel": "intelslava", "text": "BREAKING: ...", "urgent": true }
  ],
  "recentPosts": [...]
}
```

#### USGS 地震

```rust
// API: earthquake.usgs.gov/earthquakes/feed/v1.0/summary/2.5_day.geojson
// 返回 GeoJSON，解析 features 数组
// 提取: 总数、M5+ 显著地震、最大震级、海啸预警、区域分布

async fn sweep(&self) -> Result<Value> {
    let url = "https://earthquake.usgs.gov/earthquakes/feed/v1.0/summary/2.5_day.geojson";
    let data = self.client.fetch_json(url).await?;
    
    let features = data["features"].as_array().unwrap_or(&vec![]);
    let mut significant = Vec::new();
    let mut max_mag: f64 = 0.0;
    let mut tsunami_count = 0;
    
    for f in features {
        let mag = f["properties"]["mag"].as_f64().unwrap_or(0.0);
        let place = f["properties"]["place"].as_str().unwrap_or("");
        let tsunami = f["properties"]["tsunami"].as_i64().unwrap_or(0);
        let geo = f["geometry"]["coordinates"].as_array();
        
        if mag > max_mag { max_mag = mag; }
        if tsunami > 0 { tsunami_count += 1; }
        
        if mag >= 5.0 && significant.len() < 20 {
            significant.push(json!({
                "magnitude": mag,
                "place": place,
                "tsunami": tsunami > 0,
                "lat": geo.and_then(|g| g.get(1)),
                "lon": geo.and_then(|g| g.get(0)),
            }));
        }
    }
    
    Ok(json!({
        "source": "USGS",
        "totalQuakes": features.len(),
        "maxMagnitude": max_mag,
        "significantQuakes": significant,
        "tsunamiWarnings": tsunami_count,
    }))
}
```

#### CVE/NVD 漏洞情报

```rust
// API: services.nvd.nist.gov/rest/json/cves/2.0
// 查询最近 7 天的 CVE，提取 Critical/High 漏洞
// CVSS 评分优先级：v3.1 → v3.0 → v2

fn extract_severity(metrics: &Value) -> (f64, String) {
    // 优先 v3.1
    if let Some(v31) = metrics["cvssMetricV31"].as_array().and_then(|a| a.first()) {
        let score = v31["cvssData"]["baseScore"].as_f64().unwrap_or(0.0);
        let severity = v31["cvssData"]["baseSeverity"].as_str().unwrap_or("UNKNOWN");
        return (score, severity.to_string());
    }
    // fallback v3.0, v2...
    (0.0, "UNKNOWN".to_string())
}
```

#### GDACS 全球灾害预警（RSS/XML）

```rust
// API: www.gdacs.org/xml/rss.xml (RSS/XML 格式)
// 不引入 XML 解析库，自实现极简 tag 提取器

fn extract_tag(xml: &str, tag: &str) -> Option<String> {
    let open = format!("<{}", tag);
    let close = format!("</{}>", tag);
    let start = xml.find(&open)?;
    let content_start = xml[start..].find('>')? + start + 1;
    let content = &xml[content_start..];
    let end = content.find(&close)?;
    let raw = &content[..end];
    // Handle CDATA sections
    if raw.contains("<![CDATA[") {
        Some(raw.replace("<![CDATA[", "").replace("]]>", "").trim().to_string())
    } else {
        Some(raw.trim().to_string())
    }
}
```

---

## 五、HTTP 客户端：指数退避重试

```rust
pub struct HttpClient {
    client: reqwest::Client,
    max_retries: u32,
}

impl HttpClient {
    pub fn new(timeout_secs: u64, max_retries: u32) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .pool_max_idle_per_host(10)  // 连接池复用
            .build()
            .expect("Failed to build HTTP client");
        Self { client, max_retries }
    }
    
    pub async fn fetch_json(&self, url: &str) -> Result<Value> {
        let mut last_err = None;
        
        for attempt in 0..=self.max_retries {
            if attempt > 0 {
                // 指数退避：500ms, 1s, 2s
                let backoff = Duration::from_millis(500 * 2u64.pow(attempt - 1));
                tokio::time::sleep(backoff).await;
            }
            match self.do_fetch(url).await {
                Ok(val) => return Ok(val),
                Err(e) => last_err = Some(e),
            }
        }
        Err(last_err.unwrap())
    }
    
    pub async fn fetch_text(&self, url: &str) -> Result<String> {
        // 同上逻辑，返回 String 而非 Value
        // 用于 Telegram HTML 页面、RSS/XML 等非 JSON 响应
    }
    
    async fn do_fetch(&self, url: &str) -> Result<Value> {
        let resp = self.client.get(url)
            .header("User-Agent", "Crucix/1.0 IntelligenceTerminal")
            .send()
            .await?;
        
        if !resp.status().is_success() {
            anyhow::bail!("HTTP {} for {}", resp.status(), url);
        }
        
        Ok(resp.json().await?)
    }
}
```

---

## 六、LLM 集成：Provider 抽象 + OpenAI 兼容协议

### 6.1 LlmProvider Trait

```rust
#[async_trait]
pub trait LlmProvider: Send + Sync {
    fn name(&self) -> &str;
    fn is_configured(&self) -> bool;
    async fn complete(
        &self,
        system_prompt: &str,
        user_message: &str,
        opts: &LlmOptions,
    ) -> Result<LlmResponse>;
}

pub struct LlmOptions {
    pub max_tokens: u32,
    pub temperature: f32,
    pub model_override: Option<String>,
}

pub struct LlmResponse {
    pub text: String,
    pub model: String,
    pub input_tokens: u32,
    pub output_tokens: u32,
}
```

### 6.2 OpenAI 兼容实现（覆盖 95% 模型）

```rust
pub struct OpenAiProvider {
    base_url: String,    // https://api.deepseek.com/v1, https://api.openai.com/v1, etc.
    api_key: String,
    default_model: String,
    think_model: String, // 用于深度推理的模型（如 deepseek-reasoner）
    client: reqwest::Client,
}

// 通过 .env 配置切换任何 OpenAI 兼容模型：
// BASE_URL=https://api.deepseek.com/v1
// API_KEY=sk-xxx
// DEFAULT_MODEL=deepseek-chat
// THINK_MODEL=deepseek-reasoner
//
// 换成 OpenAI/Moonshot/GLM-4/Qwen：只需改 BASE_URL + API_KEY，零代码改动
```

### 6.3 本地 Ollama 支持（真正 Zero Cloud）

```rust
pub struct OllamaProvider {
    base_url: String,    // http://localhost:11434
    model: String,       // qwen3.5:9b, llama3, mistral 等
    client: reqwest::Client,
}

#[async_trait]
impl LlmProvider for OllamaProvider {
    fn name(&self) -> &str { "ollama" }
    fn is_configured(&self) -> bool { true } // 本地不需要 Key
    
    async fn complete(&self, system: &str, user: &str, opts: &LlmOptions) -> Result<LlmResponse> {
        let url = format!("{}/v1/chat/completions", self.base_url);
        // Ollama 暴露 OpenAI 兼容端点，请求体格式完全一致
    }
}
```

### 6.4 工厂函数路由

```rust
pub fn create_provider(config: &Config) -> Box<dyn LlmProvider> {
    match config.llm_provider.as_str() {
        "ollama" => Box::new(OllamaProvider::new(
            &config.ollama_url,      // 默认 http://localhost:11434
            &config.ollama_model,    // 默认 qwen3.5:9b
        )),
        _ => Box::new(OpenAiProvider::new(
            &config.base_url,
            &config.api_key,
            &config.default_model,
            &config.think_model,
        )),
    }
}
```

### 6.5 中英双语 System Prompt

```rust
const BRIEFING_SYSTEM_PROMPT_ZH: &str = r#"
你是 Crucix，一个情报分析 AI。你接收来自多个开源情报源的结构化数据。
任务：合成一份简洁的情报简报。
要求：直接、专业、避免臆测。引用具体数据点。使用简洁的军事风格措辞。全程使用中文输出。
输出结构：
1. 态势概览（3-5 句）
2. 关键动态（编号列表，每条引用数据）
3. 风险评估（LOW/ELEVATED/HIGH/CRITICAL + 理由）
4. 建议关注（未来 24 小时重点）
"#;

const BRIEFING_SYSTEM_PROMPT_EN: &str = r#"
You are Crucix, an intelligence analysis AI...
Be direct, analytical, avoid speculation. Cite specific data points. Use military-style brevity.
"#;
```

---

## 七、Delta 变化检测引擎

### 7.1 阈值定义

```rust
// 数值型指标（百分比变化触发）
const NUMERIC_THRESHOLDS: &[(&str, f64)] = &[
    ("vix", 5.0),          // VIX 恐慌指数变化 > 5%
    ("wti", 3.0),          // WTI 原油 > 3%
    ("brent", 3.0),        // 布伦特原油 > 3%
    ("fed_funds", 1.0),    // 联邦基金利率 > 1%
    ("10y_yield", 3.0),    // 10 年期国债收益率 > 3%
    ("spy", 2.0),          // S&P 500 > 2%
    ("btc", 5.0),          // 比特币 > 5%
    ("gold", 2.0),         // 黄金 > 2%
    ("nat_gas", 5.0),      // 天然气 > 5%
];

// 计数型指标（绝对值变化触发）
const COUNT_THRESHOLDS: &[(&str, u64)] = &[
    ("thermal_total", 500),    // 卫星火点 > 500
    ("air_total", 50),         // 航空器数量变化 > 50
    ("conflict_events", 5),    // 冲突事件 > 5
    ("quakes", 10),            // 地震数量 > 10
    ("cve_critical", 3),       // Critical CVE > 3
];
```

### 7.2 严重度评分

```rust
pub enum Severity { Moderate, High, Critical }

fn calculate_severity(pct_change: f64, threshold: f64) -> Severity {
    if pct_change.abs() > threshold * 3.0 {
        Severity::Critical
    } else if pct_change.abs() > threshold * 2.0 {
        Severity::High
    } else {
        Severity::Moderate
    }
}
```

### 7.3 风险方向综合判断

VIX 上升 + 高收益利差扩大 + 冲突事件增加 → **risk-off（避险）**
VIX 下降 + SPY 上涨 + 冲突减少 → **risk-on（追险）**

---

## 八、跨源关联引擎（6 条规则）

这是系统最有分析价值的部分——单个源的告警是"信号"，多个源同时告警是"模式"。

```rust
pub struct CorrelationSignal {
    pub name: String,           // 如 "GEOPOLITICAL_RISK_CONVERGENCE"
    pub severity: String,       // "high", "critical"
    pub sources: Vec<String>,   // 关联的数据源
    pub description: String,    // 人类可读描述
    pub indicators: Vec<String>,// 具体指标值
}

pub fn analyze_correlations(data: &Value) -> Vec<CorrelationSignal> {
    let mut signals = Vec::new();
    
    // 规则 1: 自然灾害级联
    // 显著地震 + (海啸预警 OR GDACS Red) → NATURAL_DISASTER_CASCADE
    
    // 规则 2: 地缘风险收敛
    // VIX > 25 + 冲突事件 > 50 + WTI > $85 → GEOPOLITICAL_RISK_CONVERGENCE
    
    // 规则 3: 网络威胁收敛
    // Critical CVE ≥ 3 + ISC 威胁升级 → CYBER_THREAT_CONVERGENCE
    
    // 规则 4: 基础设施压力
    // 太空天气 ≥ 2 级 + (火灾 > 1000 OR 极端天气 > 5) → INFRASTRUCTURE_STRESS
    
    // 规则 5: 市场恐慌
    // VIX > 30 + 国债收益率下跌(避险) → MARKET_PANIC
    
    // 规则 6: 人道主义危机
    // WHO 告警 ≥ 3 + 冲突伤亡 > 50 → HUMANITARIAN_CRISIS
    
    signals
}
```

关联信号附加到 LLM 分析的上下文中，让 AI 重点关注复合风险模式。

---

## 九、SQLite 持久化存储

### 9.1 Schema（三张表）

```sql
-- 扫描记录
CREATE TABLE IF NOT EXISTS sweeps (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp     TEXT NOT NULL,
    duration_ms   INTEGER NOT NULL,
    sources_ok    INTEGER NOT NULL,
    sources_err   INTEGER NOT NULL,
    total_sources INTEGER NOT NULL,
    data_json     TEXT NOT NULL          -- 完整扫描 JSON
);

-- LLM 分析结果
CREATE TABLE IF NOT EXISTS analyses (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    sweep_id      INTEGER NOT NULL REFERENCES sweeps(id),
    model         TEXT NOT NULL,
    language      TEXT NOT NULL DEFAULT 'en',
    content       TEXT NOT NULL,
    input_tokens  INTEGER NOT NULL DEFAULT 0,
    output_tokens INTEGER NOT NULL DEFAULT 0
);

-- 数据源健康记录
CREATE TABLE IF NOT EXISTS source_health (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    sweep_id      INTEGER NOT NULL REFERENCES sweeps(id),
    source_name   TEXT NOT NULL,
    status        TEXT NOT NULL,          -- "ok", "error", "timeout"
    duration_ms   INTEGER NOT NULL,
    error         TEXT
);
```

### 9.2 关键设计决策：为什么是 SQLite 而非 PostgreSQL

- **单二进制原则**：`rusqlite` 的 `bundled` feature 将 SQLite 编译进二进制，用户无需安装任何数据库
- **性能足够**：每 15 分钟一次扫描，每次几十 KB JSON，SQLite 轻松应对
- **零运维**：不需要 DBA、不需要备份策略、不需要连接池配置
- **可移植**：整个数据库是一个文件（`runs/crucix.db`），复制即备份

### 9.3 趋势数据提取

```rust
pub struct TrendPoint {
    pub timestamp: String,
    pub sweep_id: i64,
    pub metrics: HashMap<String, f64>,  // vix, wti, spy, btc, aircraft, fires, conflicts, quakes...
}

impl Store {
    pub fn trend_data(&self, limit: usize) -> Result<Vec<TrendPoint>> {
        // 从最近 N 次 sweep 的 data_json 中提取关键指标
    }
    
    pub fn source_reliability(&self, lookback: usize) -> Result<Vec<SourceReliability>> {
        // 按源分组统计成功率和连续失败次数
    }
    
    pub fn degraded_sources(&self, threshold: u32) -> Result<Vec<String>> {
        // 连续失败 >= threshold 的源（自动标记 DEGRADED）
    }
}
```

---

## 十、智能告警推送

### 10.1 告警触发条件

```rust
const ALERT_CONDITIONS: &[AlertCondition] = &[
    ("VIX > 30",                "市场恐慌"),
    ("地震 M6+",                "重大地震"),
    ("海啸预警",                "紧急"),
    ("GDACS Red",              "全球灾害"),
    ("太空天气 Scale 3+",       "基础设施风险"),
    ("ISC Orange/Red",         "网络攻击升级"),
    ("Critical CVE ≥ 5",      "漏洞风暴"),
    ("Delta Critical 信号",    "指标剧变"),
];
```

### 10.2 推送渠道

```rust
// Slack / Discord Webhook
json!({ "content": message, "text": message })

// 飞书 / Lark Webhook
json!({ "msg_type": "text", "content": { "text": message } })

// macOS 桌面通知
std::process::Command::new("osascript")
    .args(["-e", &format!(r#"display notification "{}" with title "Crucix Alert""#, body)])
    .spawn().ok();

// Linux 桌面通知
std::process::Command::new("notify-send")
    .args(["Crucix Alert", &body])
    .spawn().ok();
```

---

## 十一、用户自定义 Watchlist

通过 `.env` 配置三个维度的过滤：

```env
# 关注区域——匹配 ACLED/GDELT/USGS/GDACS 中的地理区域
WATCH_REGIONS=Middle East,Taiwan Strait,South China Sea,Southeast Asia

# 关键词——扫描新闻标题、Telegram 帖子、Reddit、CVE 描述
ALERT_KEYWORDS=nuclear,zero-day,sanctions,semiconductor,optical

# 关注资产——从 YFinance 提取特定标的
WATCH_TICKERS=NVDA,TSLA,ETH-USD,CL=F
```

匹配结果标记为 **USER PRIORITY** 发送给 LLM，让分析重点覆盖用户关心的领域。

---

## 十二、Web 仪表板（Axum + SSE + 3D 地球）

### 12.1 Axum 路由

```rust
// 仪表板 HTML 编译进二进制（include_str!）
const DASHBOARD_HTML: &str = include_str!("../../static/dashboard.html");

let app = Router::new()
    .route("/", get(|| async { Html(DASHBOARD_HTML) }))
    .route("/api/data", get(data_handler))          // 最新扫描数据
    .route("/api/analysis", get(analysis_handler))  // LLM 分析结果
    .route("/api/trends", get(trends_handler))      // 趋势数据（最近 50 次）
    .route("/api/health", get(health_handler))      // 源健康状态
    .route("/api/sse", get(sse_handler))            // Server-Sent Events 实时推送
    .with_state(state);
```

### 12.2 仪表板特性

- 深色 HUD 风格（赛博朋克情报终端）
- D3.js 3D 地球 + Natural Earth 平面地图
- 实时市场数据面板（SPY/BTC/VIX/黄金/原油，带涨跌色）
- 数据源健康监控（绿/红/黄三色指示 + DEGRADED 标记）
- 告警面板（汇聚 ACLED 冲突、WHO 疫情、气象预警、网络威胁）
- AI 分析面板 + 趋势面板（标签页切换）
- 趋势图表（纯 Canvas API，零图表库）
- SSE 实时推送（无需手动刷新）
- **自包含**：单个 HTML 文件，不依赖任何 CDN

---

## 十三、CLI 命令体系

```bash
# 查看系统状态（源列表 + 配置 + 数据库状态 + 源健康度）
crucix status

# 执行完整情报扫描（19 源并行 + Delta + LLM + 告警）
crucix sweep
crucix sweep --lang zh          # 中文分析
crucix sweep --json --no-llm    # 仅 JSON，跳过 AI

# 测试单个数据源
crucix source yfinance
crucix source telegram

# 测试 LLM 连接
crucix test-llm

# 启动 Web 仪表板（持续运行 + 定时扫描）
crucix serve --port 3117

# 持续监控模式（每 N 分钟扫描）
crucix watch --interval 15

# 查看扫描历史
crucix history
crucix history --limit 20
crucix history --show 42        # 查看某次扫描的完整分析

# 趋势分析（ASCII 火花图）
crucix trends

# 报告与导出
crucix report                   # 生成 Markdown 情报报告
crucix export --limit 50        # 导出 CSV 趋势数据
```

---

## 十四、.env 配置完整模板

```env
# === LLM 配置 ===
LLM_PROVIDER=ollama              # ollama | openai（OpenAI 兼容）
# OpenAI 兼容提供商配置
BASE_URL=https://api.deepseek.com/v1
API_KEY=sk-xxx
DEFAULT_MODEL=deepseek-chat
THINK_MODEL=deepseek-reasoner
# Ollama 本地配置
OLLAMA_URL=http://localhost:11434
OLLAMA_MODEL=qwen3.5:9b

# === 可选 API Keys（渐进式增强）===
FRED_API_KEY=                    # 美联储 FRED（免费注册）
FIRMS_MAP_KEY=                   # NASA FIRMS（免费注册）
EIA_API_KEY=DEMO_KEY             # EIA 能源（DEMO_KEY 即可）
NVD_API_KEY=                     # NVD CVE（可选，加速请求）
ACLED_EMAIL=                     # ACLED 冲突（免费 OAuth2）
ACLED_KEY=
REDDIT_CLIENT_ID=                # Reddit（OAuth）
REDDIT_CLIENT_SECRET=

# === 扫描配置 ===
REFRESH_INTERVAL_MINUTES=15      # 扫描间隔
SOURCE_TIMEOUT_SECS=30           # 单源超时
SWEEP_LANG=zh                    # 默认分析语言 zh|en

# === 告警推送 ===
WEBHOOK_URL=                     # Slack/Discord/飞书 Webhook URL

# === 用户关注列表 ===
WATCH_REGIONS=Middle East,Taiwan Strait,South China Sea,Southeast Asia
ALERT_KEYWORDS=nuclear,zero-day,sanctions,semiconductor
WATCH_TICKERS=NVDA,TSLA,ETH-USD

# === Web 仪表板 ===
DASHBOARD_PORT=3117
```

---

## 十五、Docker 部署

### Dockerfile（多阶段构建）

```dockerfile
# 构建阶段
FROM rust:1.82-alpine AS builder
RUN apk add --no-cache musl-dev openssl-dev pkgconf
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src/ src/
COPY static/ static/
RUN cargo build --release

# 运行阶段
FROM alpine:3.20
RUN apk add --no-cache ca-certificates
COPY --from=builder /app/target/release/crucix /usr/local/bin/
VOLUME /data/runs
ENV DATABASE_PATH=/data/runs/crucix.db
EXPOSE 3117
HEALTHCHECK --interval=60s CMD wget -q --spider http://localhost:3117/api/health || exit 1
ENTRYPOINT ["crucix"]
CMD ["serve", "--port", "3117"]
```

### docker-compose.yml

```yaml
version: "3.8"
services:
  crucix:
    build: .
    ports:
      - "3117:3117"
    volumes:
      - ./runs:/data/runs
    env_file: .env
    restart: unless-stopped
```

---

## 十六、三次迭代演进总结

| 维度 | v1（11 源） | v2（14 源） | v3（19 源） |
|------|-----------|-----------|-----------|
| 数据源 | 11 | +Telegram/EIA/USPTO = 14 | +USGS/SWPC/GDACS/CVE/ISC = 19 |
| 存储 | 纯内存 | SQLite 持久化 | +趋势/健康度/可靠性 |
| 分析语言 | 仅英文 | 中英双语 | 中英双语 |
| LLM | 远程 OpenAI 兼容 | 同上 | +本地 Ollama |
| Delta | 内存比较 | 跨进程（SQLite） | +关联引擎（6 规则） |
| 告警 | 无 | 无 | Webhook + 桌面通知 |
| 可视化 | 基础 HUD | +3D 地球 | +Canvas 趋势图 |
| 自定义 | 无 | 无 | Watchlist（地区/关键词/资产） |
| 代码量 | ~4100 行 | ~5000 行 | ~7000 行 |

---

## 十七、给 Claude Code 的实施建议

### 推荐开发顺序

1. **Phase 1（骨架）**：`main.rs` + `config.rs` + `error.rs` + `http.rs` + `IntelSource` trait + 3 个最简单的源（YFinance/USGS/NOAA）→ 验证并行扫描工作
2. **Phase 2（核心源）**：逐步添加其余 16 个数据源，每个独立文件独立测试
3. **Phase 3（分析）**：`briefing.rs` + `llm/` + 中英双语 prompt → 验证 LLM 分析流程
4. **Phase 4（持久化）**：`store.rs` + SQLite schema → `delta/` 变化检测
5. **Phase 5（关联）**：`correlation.rs` + `watchlist.rs` + `notify.rs`
6. **Phase 6（可视化）**：`dashboard/` + `static/dashboard.html` + SSE
7. **Phase 7（CLI 完善）**：`history`/`trends`/`report`/`export` 命令

### 关键设计约束（请严格遵循）

- **不引入 HTML 解析库**——Telegram 和 GDACS 的 HTML/XML 用自实现的 tag stripper
- **不引入任何 LLM SDK**——所有 LLM 调用通过 reqwest 直接发 HTTP
- **不引入 dotenv crate**——自实现 .env 加载器
- **SQLite 必须用 bundled feature**——编译进二进制
- **仪表板 HTML 必须 include_str!**——嵌入二进制，零外部文件依赖
- **每个数据源一个文件**——独立可测试
- **所有数据源的输出 JSON 必须包含 `"source": "SourceName"` 字段**
