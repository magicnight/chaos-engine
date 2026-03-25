# Contributing to CHAOS

CHAOS moves quickly, but review bandwidth is limited. The easiest way to get a change merged is to keep it small, well-scoped, and aligned with the project's direction.

## What Contributions Are Most Helpful

- Focused bug fixes with a clear reproduction and validation path
- Documentation improvements that reduce setup friction
- Dashboard usability improvements with a small review surface
- New OSINT sources that add clear signal, degrade gracefully, and fit the existing architecture

## Changes That Should Start With an Issue First

Open an issue before writing code if your change would:

- add a new external provider or paid API
- add a new feature family or dashboard surface
- change the project scope or roadmap
- change licensing, distribution, or deployment model
- introduce new dependencies

## Development Baseline

- Rust stable (edition 2021)
- `cargo build` / `cargo test` must pass
- Keep the minimal-dependency approach unless there is a strong reason not to
- Do not commit secrets, `.env` files, or generated runtime data (`runs/`)

## Adding a New Source

Each source is a Rust module in `src/sources/` implementing the `IntelSource` trait. A template is at `src/sources/_template.rs`.

Three steps to add a source:

1. Create `src/sources/your_source.rs` implementing `IntelSource` (name, description, tier, sweep)
2. Register it in `src/sources/mod.rs` (`pub mod` + add to `build_sources()`)
3. Done -- the briefing engine, CLI, dashboard, delta engine, and API all pick it up automatically

Minimum expectations:

- implement `IntelSource` trait (`name`, `description`, `tier`, `sweep`)
- return structured JSON via `serde_json::json!()` with consistent field naming
- handle upstream errors and rate limits cleanly (return error JSON, don't panic)
- degrade gracefully when API keys are missing
- avoid breaking the full sweep if the source fails
- document any required environment variables in `.env.example` and `README.md`
- explain why the source improves signal quality, not just source count

If your source should display in a dashboard panel:

- add data extraction in `synthesize()` function in `static/dashboard.html`
- add or update an `update*()` rendering function
- register in `updateAllPanels()` and the `panelUpdaters` map
- explain the user-facing impact in the PR
- include a screenshot when the UI changes materially

## Frontend and Security Expectations

Frontend changes are reviewed carefully because the dashboard renders mixed-source data.

- do not render untrusted content directly with `innerHTML` unless it is sanitized first
- only allow safe external URL schemes such as `http:` and `https:`
- escape JSON injected into inline `<script>` tags
- prefer text rendering over HTML rendering when possible

## Pull Request Scope

One bugfix or one feature family per PR.

Good:

- fix one parser bug
- add one source and its minimal wiring
- improve one dashboard interaction

Bad:

- add a source, redesign the dashboard, and change config behavior in the same PR
- mix bug fixes with unrelated product expansion
- bundle license or provider changes into unrelated work

Large changes may be asked to split before review.

## Pull Request Checklist

Before opening a PR, make sure you have:

- explained the problem and why the change is needed
- kept the diff focused
- listed any setup or environment variable changes
- validated the changed path locally
- included screenshots for visible dashboard changes
- updated docs when behavior or config changed

## Review Priorities

Review is primarily about:

- correctness
- regression risk
- security of mixed-source content rendering
- maintainability
- fit with project direction

Not every technically correct change will be merged. Scope and long-term maintenance cost matter.

## Commit and PR Hygiene

- Use clear commit messages
- Avoid unrelated file churn
- Do not include generated metadata or tool signatures in PR descriptions unless they are actually useful
- Keep PR titles specific and concrete

## Security Reports

If you believe you found a security issue, do not open a public issue first. See `SECURITY.md` for reporting instructions.
