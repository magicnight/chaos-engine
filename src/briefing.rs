use crate::config::Config;
use crate::correlation;
use crate::delta;
use crate::http::HttpClient;
use crate::llm::{LlmOptions, LlmProvider};
use crate::notify::Notifier;
use crate::sources::{build_sources, run_source, SourceResult, SourceStatus};
use crate::store::Store;
use crate::watchlist::Watchlist;
use chrono::Utc;
use futures::future::join_all;
use serde_json::{json, Value};
use std::time::{Duration, Instant};

const BRIEFING_SYSTEM_PROMPT_EN: &str = "\
You are CHAOS (Connected Human-Augmented OSINT Suite), an elite intelligence analyst AI.

You receive structured data from multiple open-source intelligence feeds covering:
- Financial markets (equities, commodities, crypto, bonds, FX)
- Geopolitical conflicts and armed events
- Natural disasters (earthquakes, severe weather, fires, radiation)
- Cyber threats (CVE, threat levels, internet anomalies)
- Social sentiment (news, OSINT channels, social media)
- Space and satellite activity

TASK: Synthesize a concise, actionable intelligence briefing.

STYLE:
- Military brevity. No filler words.
- Every claim cites specific data (numbers, sources, dates).
- Highlight cross-domain correlations (e.g., oil spike + conflict escalation + VIX rise).
- Flag anything anomalous or historically unusual.

OUTPUT FORMAT:
## SITUATION OVERVIEW
3-5 sentences capturing the global picture. Lead with the most significant development.

## KEY DEVELOPMENTS
Numbered list. Each item: [SOURCE] specific data point -> analysis -> implication.
Max 8 items, ordered by significance.

## RISK MATRIX
| Domain | Level | Rationale |
Current levels: MINIMAL / LOW / ELEVATED / HIGH / CRITICAL
Domains: Markets, Geopolitical, Cyber, Natural, Health

## ACTIONABLE INTELLIGENCE
2-3 specific, testable predictions or recommendations for the next 24 hours.
Format: \"WATCH: [specific indicator] because [reason]. Threshold: [value].\"

## CROSS-DOMAIN CORRELATIONS
Any patterns spanning 2+ intelligence domains. Cite specific data from each domain.";

const BRIEFING_SYSTEM_PROMPT_ZH: &str = "\
你是 CHAOS（Connected Human-Augmented OSINT Suite），一个顶级情报分析AI。

你接收来自多个开源情报源的结构化数据，覆盖：
- 金融市场（股指、大宗商品、加密货币、债券、外汇）
- 地缘冲突与武装事件
- 自然灾害（地震、极端天气、火灾、辐射）
- 网络威胁（CVE漏洞、威胁等级、互联网异常）
- 社会情绪（新闻、OSINT频道、社交媒体）
- 太空与卫星活动

任务：合成一份简洁、可操作的情报简报。

风格：
- 军事简报风格，不要废话
- 每个判断必须引用具体数据（数值、来源、日期）
- 重点标注跨领域关联（如：油价飙升 + 冲突升级 + VIX上涨）
- 标记任何异常或历史罕见的现象

输出格式：
## 态势概览
3-5句话概括全球态势。最重要的发展放在最前面。

## 关键动态
编号列表。每条：[数据源] 具体数据 -> 分析 -> 影响。
最多8条，按重要性排序。

## 风险矩阵
| 领域 | 等级 | 理由 |
等级：最低 / 低 / 升高 / 高 / 危急
领域：市场、地缘政治、网络安全、自然灾害、公共卫生

## 可操作情报
2-3条具体的、可验证的预测或建议（未来24小时）。
格式：\"关注：[具体指标]，原因：[分析]。阈值：[数值]。\"

## 跨域关联
横跨2个以上情报领域的模式。引用每个领域的具体数据。";

const BRIEFING_SYSTEM_PROMPT_JA: &str = "\
あなたは CHAOS（Connected Human-Augmented OSINT Suite）、最高水準の情報分析AIです。

以下を網羅する複数のオープンソースインテリジェンスフィードから構造化データを受信します：
- 金融市場（株式、商品、暗号通貨、債券、為替）
- 地政学的紛争・武力事件
- 自然災害（地震、異常気象、火災、放射線）
- サイバー脅威（CVE、脅威レベル、インターネット異常）
- 社会的センチメント（ニュース、OSINTチャンネル、SNS）
- 宇宙・衛星活動

任務：簡潔かつ実行可能なインテリジェンスブリーフィングを作成せよ。

形式：
- 軍事ブリーフィング形式。冗長な表現は排除。
- 全ての判断に具体的データを引用（数値、ソース、日時）。
- 領域横断的相関を重点的に示す（例：原油急騰 + 紛争激化 + VIX上昇）。
- 異常値や歴史的に稀な事象をフラグ付け。

出力形式：
## 情勢概要
3-5文でグローバル情勢を要約。最も重要な展開を冒頭に。

## 主要動向
番号付きリスト。各項目：[ソース] 具体データ -> 分析 -> 影響。
最大8項目、重要度順。

## リスクマトリクス
| 領域 | レベル | 根拠 |
レベル：最小 / 低 / 上昇 / 高 / 危機的
領域：市場、地政学、サイバー、自然災害、公衆衛生

## 実行可能情報
今後24時間の具体的かつ検証可能な予測・提言を2-3件。
形式：\"監視：[具体的指標]、理由：[分析]。閾値：[数値]。\"

## 領域横断相関
2つ以上の情報領域にまたがるパターン。各領域の具体的データを引用。";

const BRIEFING_SYSTEM_PROMPT_ES: &str = "\
Eres CHAOS (Connected Human-Augmented OSINT Suite), una IA de analisis de inteligencia de elite.

Recibes datos estructurados de multiples fuentes de inteligencia de codigo abierto que cubren:
- Mercados financieros (acciones, materias primas, criptomonedas, bonos, divisas)
- Conflictos geopoliticos y eventos armados
- Desastres naturales (terremotos, clima severo, incendios, radiacion)
- Amenazas ciberneticas (CVE, niveles de amenaza, anomalias de internet)
- Sentimiento social (noticias, canales OSINT, redes sociales)
- Actividad espacial y satelital

TAREA: Sintetizar un informe de inteligencia conciso y accionable.

ESTILO:
- Brevedad militar. Sin palabras de relleno.
- Cada afirmacion cita datos especificos (numeros, fuentes, fechas).
- Destacar correlaciones entre dominios (ej: alza del petroleo + escalada del conflicto + subida del VIX).
- Senalar cualquier anomalia o evento historicamente inusual.

FORMATO DE SALIDA:
## PANORAMA DE LA SITUACION
3-5 oraciones capturando el panorama global. Comenzar con el desarrollo mas significativo.

## DESARROLLOS CLAVE
Lista numerada. Cada item: [FUENTE] dato especifico -> analisis -> implicacion.
Maximo 8 items, ordenados por importancia.

## MATRIZ DE RIESGO
| Dominio | Nivel | Justificacion |
Niveles: MINIMO / BAJO / ELEVADO / ALTO / CRITICO
Dominios: Mercados, Geopolitica, Cibernetico, Natural, Salud

## INTELIGENCIA ACCIONABLE
2-3 predicciones o recomendaciones especificas y verificables para las proximas 24 horas.
Formato: \"VIGILAR: [indicador especifico] porque [razon]. Umbral: [valor].\"

## CORRELACIONES ENTRE DOMINIOS
Patrones que abarcan 2+ dominios de inteligencia. Citar datos especificos de cada dominio.";

/// Run a full parallel sweep of all sources, then run the analysis pipeline.
pub async fn full_sweep(
    client: &HttpClient,
    store: &Store,
    llm: Option<&dyn LlmProvider>,
    config: &Config,
) -> Value {
    let sources = build_sources(client);
    let timeout = Duration::from_secs(config.source_timeout_secs);
    let start = Instant::now();

    tracing::info!(source_count = sources.len(), "Starting intelligence sweep");

    // 1. Run all sources in parallel
    let futures: Vec<_> = sources
        .iter()
        .map(|src| run_source(src.as_ref(), timeout))
        .collect();
    let results = join_all(futures).await;

    let total_ms = start.elapsed().as_millis() as u64;

    // Log per-source status with structured fields
    let timeout_warn_threshold_ms = (config.source_timeout_secs * 1000 * 80 / 100) as u64;
    for r in &results {
        let status = &r.status;
        let err_msg = r.error.as_deref().unwrap_or("");
        if r.status == SourceStatus::Ok && r.duration_ms >= timeout_warn_threshold_ms {
            tracing::warn!(source = %r.name, duration_ms = r.duration_ms, status = %status, "Source completed (near timeout)");
        } else if r.status == SourceStatus::Ok {
            tracing::debug!(source = %r.name, duration_ms = r.duration_ms, status = %status, "Source completed");
        } else {
            tracing::warn!(source = %r.name, duration_ms = r.duration_ms, status = %status, error = %err_msg, "Source failed");
        }
    }

    // Sweep timing summary
    let slowest = results.iter().max_by_key(|r| r.duration_ms);
    if let Some(s) = slowest {
        tracing::info!(
            total_ms = total_ms,
            slowest_source = %s.name,
            slowest_ms = s.duration_ms,
            "Sweep completed"
        );
    }

    // 2. Build output JSON
    let mut output = build_output(&results, total_ms);

    let sources_ok = results
        .iter()
        .filter(|r| r.status == SourceStatus::Ok)
        .count();
    let sources_err = results.len() - sources_ok;

    // 3. Save sweep to SQLite
    let sweep_id = match store.save_sweep(&output, total_ms, sources_ok, sources_err, results.len())
    {
        Ok(id) => id,
        Err(e) => {
            tracing::error!(error = %e, "Failed to save sweep");
            -1
        }
    };

    // 4. Save source health
    if sweep_id > 0 {
        if let Err(e) = store.save_source_health(sweep_id, &results) {
            tracing::error!(error = %e, "Failed to save source health");
        }
    }

    // 5. Compute delta
    let previous = store
        .get_sweep_data(sweep_id - 1)
        .ok()
        .flatten();
    let delta_result = match &previous {
        Some(prev) => delta::compute_delta(&output, prev),
        None => None,
    };

    if let Some(ref dr) = delta_result {
        let delta_json = delta_to_json(dr);
        output["delta"] = delta_json;
        tracing::info!(
            total_changes = dr.summary.total_changes,
            critical_changes = dr.summary.critical_changes,
            direction = %dr.summary.direction,
            "Delta computed"
        );
    }

    // 6. Run correlation engine
    let correlations = correlation::analyze_correlations(&output);
    if !correlations.is_empty() {
        let corr_json: Vec<Value> = correlations
            .iter()
            .map(|c| {
                json!({
                    "name": c.name,
                    "severity": c.severity,
                    "sources": c.sources,
                    "description": c.description,
                    "indicators": c.indicators,
                })
            })
            .collect();
        output["correlations"] = Value::Array(corr_json);
        tracing::info!(count = correlations.len(), "Correlations detected");
    }

    // 7. Run watchlist filter
    let watchlist = Watchlist::from_config(config);
    let watch_matches = watchlist.filter_sweep(&output);
    if !watch_matches.is_empty() {
        let wm_json: Vec<Value> = watch_matches
            .iter()
            .map(|m| {
                json!({
                    "type": m.match_type,
                    "matched": m.matched,
                    "source": m.source,
                    "context": m.context,
                })
            })
            .collect();
        output["watchlist_matches"] = Value::Array(wm_json);
        tracing::info!(count = watch_matches.len(), "Watchlist matches found");
    }

    // 8. LLM analysis
    if let Some(provider) = llm {
        if provider.is_configured() {
            tracing::info!(provider = %provider.name(), "Running LLM analysis");

            let system_prompt = match config.sweep_lang.as_str() {
                "zh" => BRIEFING_SYSTEM_PROMPT_ZH,
                "ja" => BRIEFING_SYSTEM_PROMPT_JA,
                "es" => BRIEFING_SYSTEM_PROMPT_ES,
                _ => BRIEFING_SYSTEM_PROMPT_EN,
            };

            let user_message = build_llm_prompt(&output, &delta_result, &correlations, &watch_matches);

            let opts = LlmOptions {
                max_tokens: 4096,
                temperature: 0.3,
                model_override: None,
            };

            match provider.complete(system_prompt, &user_message, &opts).await {
                Ok(response) => {
                    output["analysis"] = json!({
                        "text": response.text,
                        "model": response.model,
                        "input_tokens": response.input_tokens,
                        "output_tokens": response.output_tokens,
                    });

                    // Save analysis to SQLite
                    if sweep_id > 0 {
                        if let Err(e) = store.save_analysis(
                            sweep_id,
                            &response.model,
                            &config.sweep_lang,
                            &response.text,
                            response.input_tokens,
                            response.output_tokens,
                        ) {
                            tracing::error!(error = %e, "Failed to save analysis");
                        }
                    }

                    tracing::info!(
                        input_tokens = response.input_tokens,
                        output_tokens = response.output_tokens,
                        model = %response.model,
                        "LLM analysis complete"
                    );
                }
                Err(e) => {
                    tracing::error!(error = %e, "LLM analysis failed");
                    output["analysis"] = json!({ "error": e.to_string() });
                }
            }
        }
    }

    // 9. Notifications
    let notifier = Notifier::new(config.webhook_url.clone());
    let has_significant_changes = delta_result
        .as_ref()
        .map(|d| d.summary.critical_changes > 0 || d.summary.total_changes >= 3)
        .unwrap_or(false)
        || !correlations.is_empty();

    if notifier.is_configured() && has_significant_changes {
        let msg = build_notification_message(&delta_result, &correlations);
        if let Err(e) = notifier.send_webhook(&msg).await {
            tracing::error!(error = %e, "Notification failed");
        }
        notifier.send_desktop("CHAOS Alert", &msg);
    }

    output
}

/// Run a single named source.
pub async fn single_source(client: &HttpClient, name: &str, timeout_secs: u64) -> Option<Value> {
    let sources = build_sources(client);
    let timeout = Duration::from_secs(timeout_secs);

    let source = sources.iter().find(|s| s.name().eq_ignore_ascii_case(name))?;

    tracing::info!(source = %source.name(), "Testing source");
    let result = run_source(source.as_ref(), timeout).await;

    Some(json!({
        "source": result.name,
        "status": result.status.to_string(),
        "tier": result.tier,
        "duration_ms": result.duration_ms,
        "data": result.data,
        "error": result.error,
    }))
}

/// List all available sources with metadata.
pub fn list_sources(client: &HttpClient) -> Vec<Value> {
    build_sources(client)
        .iter()
        .map(|s| {
            json!({
                "name": s.name(),
                "description": s.description(),
                "tier": s.tier(),
            })
        })
        .collect()
}

fn build_output(results: &[SourceResult], total_ms: u64) -> Value {
    let sources_ok = results
        .iter()
        .filter(|r| r.status == SourceStatus::Ok)
        .count();
    let sources_err = results.len() - sources_ok;

    let sources_data: serde_json::Map<String, Value> = results
        .iter()
        .filter(|r| r.status == SourceStatus::Ok)
        .filter_map(|r| r.data.clone().map(|d| (r.name.clone(), d)))
        .collect();

    let errors: Vec<Value> = results
        .iter()
        .filter(|r| r.status != SourceStatus::Ok)
        .map(|r| {
            json!({
                "name": r.name,
                "error": r.error,
                "status": r.status.to_string(),
            })
        })
        .collect();

    let timing: serde_json::Map<String, Value> = results
        .iter()
        .map(|r| {
            (
                r.name.clone(),
                json!({ "status": r.status.to_string(), "ms": r.duration_ms }),
            )
        })
        .collect();

    json!({
        "chaos": {
            "version": env!("CARGO_PKG_VERSION"),
            "timestamp": Utc::now().to_rfc3339(),
            "totalDurationMs": total_ms,
            "sourcesQueried": results.len(),
            "sourcesOk": sources_ok,
            "sourcesFailed": sources_err,
        },
        "sources": sources_data,
        "errors": errors,
        "timing": timing,
    })
}

fn delta_to_json(dr: &delta::DeltaResult) -> Value {
    let signal_to_json = |s: &delta::DeltaSignal| {
        json!({
            "key": s.key,
            "label": s.label,
            "from": s.from,
            "to": s.to,
            "pct_change": s.pct_change,
            "direction": s.direction,
            "severity": s.severity.to_string(),
        })
    };

    json!({
        "timestamp": dr.timestamp,
        "previous": dr.previous,
        "signals": {
            "new": dr.signals.new.iter().map(signal_to_json).collect::<Vec<_>>(),
            "escalated": dr.signals.escalated.iter().map(signal_to_json).collect::<Vec<_>>(),
            "deescalated": dr.signals.deescalated.iter().map(signal_to_json).collect::<Vec<_>>(),
            "unchanged": dr.signals.unchanged,
        },
        "summary": {
            "total_changes": dr.summary.total_changes,
            "critical_changes": dr.summary.critical_changes,
            "direction": dr.summary.direction,
        },
    })
}

fn build_llm_prompt(
    output: &Value,
    delta_result: &Option<delta::DeltaResult>,
    correlations: &[correlation::CorrelationSignal],
    watch_matches: &[crate::watchlist::WatchMatch],
) -> String {
    let mut prompt = String::with_capacity(8192);

    // --- Sweep summary header ---
    let chaos = output.get("chaos");
    let timestamp = chaos
        .and_then(|c| c.get("timestamp"))
        .and_then(|t| t.as_str())
        .unwrap_or("unknown");
    let sources_queried = chaos
        .and_then(|c| c.get("sourcesQueried"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let sources_ok = chaos
        .and_then(|c| c.get("sourcesOk"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let total_ms = chaos
        .and_then(|c| c.get("totalDurationMs"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);

    prompt.push_str("=== SWEEP SUMMARY ===\n");
    prompt.push_str(&format!(
        "Time: {} | Sources: {}/{} OK | Duration: {}ms\n",
        timestamp, sources_ok, sources_queried, total_ms
    ));

    // Extract key metrics from source data for the summary line
    if let Some(sources) = output.get("sources").and_then(|s| s.as_object()) {
        let mut market_line = Vec::new();
        let mut energy_line = Vec::new();
        let mut threat_line = Vec::new();

        for (name, data) in sources {
            let name_lower = name.to_lowercase();
            // Attempt to pull out recognizable headline numbers
            if name_lower.contains("yfinance") || name_lower.contains("market") {
                if let Some(summary) = extract_compact_summary(data, 120) {
                    market_line.push(format!("{}: {}", name, summary));
                }
            } else if name_lower.contains("eia") || name_lower.contains("energy") {
                if let Some(summary) = extract_compact_summary(data, 80) {
                    energy_line.push(format!("{}: {}", name, summary));
                }
            } else if name_lower.contains("cisa")
                || name_lower.contains("cyber")
                || name_lower.contains("acled")
                || name_lower.contains("quake")
                || name_lower.contains("noaa")
            {
                if let Some(summary) = extract_compact_summary(data, 80) {
                    threat_line.push(format!("{}: {}", name, summary));
                }
            }
        }

        if !market_line.is_empty() {
            prompt.push_str(&format!("Markets: {}\n", market_line.join(" | ")));
        }
        if !energy_line.is_empty() {
            prompt.push_str(&format!("Energy: {}\n", energy_line.join(" | ")));
        }
        if !threat_line.is_empty() {
            prompt.push_str(&format!("Threats: {}\n", threat_line.join(" | ")));
        }
    }
    prompt.push('\n');

    // --- Delta changes ---
    if let Some(dr) = delta_result {
        prompt.push_str("=== CHANGES SINCE LAST SWEEP ===\n");
        prompt.push_str(&format!(
            "Direction: {} | Total changes: {} | Critical: {}\n",
            dr.summary.direction, dr.summary.total_changes, dr.summary.critical_changes
        ));
        for s in &dr.signals.escalated {
            let severity_tag = format!("[{}]", s.severity).to_uppercase();
            prompt.push_str(&format!(
                "^ {}: {:.4} -> {:.4} ({:+.1}%) {}\n",
                s.label, s.from, s.to, s.pct_change, severity_tag
            ));
        }
        for s in &dr.signals.deescalated {
            let severity_tag = format!("[{}]", s.severity).to_uppercase();
            prompt.push_str(&format!(
                "v {}: {:.4} -> {:.4} ({:+.1}%) {}\n",
                s.label, s.from, s.to, s.pct_change, severity_tag
            ));
        }
        for s in &dr.signals.new {
            prompt.push_str(&format!("* New: {} = {:.4}\n", s.label, s.to));
        }
        prompt.push('\n');
    }

    // --- Correlation signals ---
    if !correlations.is_empty() {
        prompt.push_str("=== CROSS-SOURCE CORRELATIONS ===\n");
        for c in correlations {
            prompt.push_str(&format!(
                "SIGNAL: {} [{}] -- {}\n  Sources: {}\n  Indicators: {}\n",
                c.name,
                c.severity,
                c.description,
                c.sources.join(", "),
                c.indicators.join(", ")
            ));
        }
        prompt.push('\n');
    }

    // --- Watchlist matches ---
    if !watch_matches.is_empty() {
        prompt.push_str("=== WATCHLIST MATCHES ===\n");
        for m in watch_matches {
            prompt.push_str(&format!(
                "- [{}] '{}' in source '{}': {}\n",
                m.match_type, m.matched, m.source, m.context
            ));
        }
        prompt.push('\n');
    }

    // --- Full source data ---
    prompt.push_str("=== SOURCE DATA ===\n\n");
    if let Some(sources) = output.get("sources").and_then(|s| s.as_object()) {
        for (name, data) in sources {
            let summary = data.to_string();
            if summary.len() < 500 {
                prompt.push_str(&format!("### {}\n{}\n\n", name, summary));
            } else {
                prompt.push_str(&format!("### {}\n{}...\n\n", name, &summary[..500]));
            }
        }
    }

    prompt
}

/// Extract a compact string summary from a source data value.
/// Returns `None` if the data is null or empty.
fn extract_compact_summary(data: &Value, max_len: usize) -> Option<String> {
    if data.is_null() {
        return None;
    }
    let s = data.to_string();
    if s.len() <= max_len {
        Some(s)
    } else {
        Some(format!("{}...", &s[..max_len]))
    }
}

fn build_notification_message(
    delta_result: &Option<delta::DeltaResult>,
    correlations: &[correlation::CorrelationSignal],
) -> String {
    let mut msg = String::from("[CHAOS Alert]\n");

    if let Some(dr) = delta_result {
        msg.push_str(&format!(
            "Direction: {} | {} changes ({} critical)\n",
            dr.summary.direction, dr.summary.total_changes, dr.summary.critical_changes
        ));
        for s in dr
            .signals
            .escalated
            .iter()
            .chain(dr.signals.deescalated.iter())
            .filter(|s| s.severity == delta::Severity::Critical)
        {
            msg.push_str(&format!(
                "  CRITICAL: {} {:+.1}%\n",
                s.label, s.pct_change
            ));
        }
    }

    for c in correlations {
        msg.push_str(&format!("  CORRELATION: {} [{}]\n", c.name, c.severity));
    }

    msg
}
