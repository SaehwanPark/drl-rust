# DRL-Rust Project Roadmap

Last reviewed: 2026-08-22
Current project version: `0.2.12`

---

## 1. Product Direction

DRL-Rust is a ground-up Rust reimplementation of *Doom the Roguelike* (DRL).
The project replaces the legacy Pascal and Lua codebase with a modern,
memory-safe, and deterministic architecture while faithfully preserving
canonical gameplay semantics.

### Primary Goals

- **Browser-First 1.0 Target**: Playable in desktop Chromium browsers
  (Chrome/Edge) via WebAssembly and WebGPU, packaged as a high-performance
  static HTTPS bundle with an accessible DOM shell and offline PWA support.
- **Deterministic Headless Core**: The simulation core (`drl-core`) is pure,
  reproducible, and completely decoupled from rendering, audio, browser, OS,
  and filesystem APIs.
- **First-Class Agent and Tooling Support**: Rich Model Context Protocol (MCP)
  interfaces, deterministic headless replay engines, automated bots, and
  statistical evaluation suites.
- **Attributable Asset Provenance**: Built-in tracking for licenses, checksums,
  and rights clearance. No runtime Lua engine in the browser bundle.

### Portability Scope

- **1.0 Scope**: Desktop Chromium (WebGPU), static web hosting, headless CLI,
  and stdio MCP server.
- **Post-1.0 Scope**: WebGL2 fallback, cross-browser support (Firefox/Safari),
  mobile/touch controls, gamepad navigation, and native desktop packaging.

---

## 2. Status Vocabulary

To maintain strict truthfulness in progress tracking, every milestone and
verification item uses explicit status semantics:

- `[x]` — **Delivered and Verified**: Fully implemented and validated by
  repository tests, CI runs, or checked artifacts.
- `[ ]` — **Planned or Open**: In progress, planned, or awaiting verification.
- `NOT_RUN` — **Environment Unavailable**: Required execution prerequisites
  were not present in the execution environment (e.g., Linux x86-64 binary
  probes on macOS arm64, or headless Chrome on minimal CI runners). This is
  recorded neutrally and is never treated as an inferred pass or failure.
- `INCONCLUSIVE` — **Unresolved Evidence**: Output exists but cannot
  definitively satisfy acceptance criteria without further evidence or rights.

---

## 3. Current Progress Summary (`VERSION` 0.2.12)

### Delivered Foundations

- **Core Simulation (M0–M2, M4)**: Pure deterministic grid maps, PRNG, turn
  economy, melee/ranged combat, armor mitigation, kinetic knockback, FOV/fog,
  inventory/equipment, tactical monster AI, and procedural level generation.
- **Tooling & Replays (M5, M6)**: Versioned replay engine (`V1`), declarative
  ASCII scenario runners, scripted bots, batch sweep runners, and a pure Rust
  zero-dependency MCP server.
- **Asset Pipeline (M3)**: Tracked CC BY-SA 4.0 legacy graphics import from
  pinned Git revision with SHA-256 validation; audio/music/fonts remain gated.
- **Browser Playable Slice (M7)**: WASM/WebGPU shell with square-cell layout,
  DOM accessibility shell, synthesized Web Audio cues, keyboard/numpad input,
  and remote web CI acceptance.
- **Audiovisual Contracts (M8)**: Measured 32px atlas slots, normalized UVs,
  layer draw plans, emissive floor sampling, `0.1` alpha cutoff, evidence-backed
  tints (Green Armor, Phase Device, StairsDown), outline-mask compositing,
  elapsed-time animation scheduling, pure effect/missile math, and bounded
  particle-decal insertion and storage contracts.
- **Typed Content & Persistence (M9, M10)**: Rust-owned definitions for current
  monsters, items, tiles, and levels; versioned fixed-session snapshot codec
  with localStorage persistence, bounded rejected-save quarantine, and static
  service-worker cache.
- **Evaluation & Release Hardening (M11, M12)**: Fixed-seed cohort reports with
  integrity validation and descriptive outcome/telemetry projections; release
  manifests with SHA-256 sidecars, cache invalidation, and checkout binding.

### Active & Open Work

- **Active Milestone Slice (M10)**: Browser-save corruption recovery is
  delivered and verified; explicit replay-compatible format migration remains
  open.
- **Open Audiovisual Parity (M8)**: Exact legacy outline/glow and lighting/LUT
  equations from reference captures, HUD typography, and replacement audio.
- **Controlled Reference Captures (M3, M7, M8)**: Runtime captures are `NOT_RUN`
  on macOS arm64; pending execution in a controlled Linux x86-64 environment.
- **Content Breadth & Balance (M9, M11)**: Build-time Lua conversion tooling,
  expanded content tables, and statistical gameplay balance studies.
- **PWA & Release Hardening (M10, M12, M13)**: Full offline-after-first-load
  acceptance, signed releases, and 1.0 desktop Chromium deployment.

---

## 4. Milestone Checklists

### M0 — Truthful Steering, Documentation, and Harness

Establish repository structure, documentation governance, and deterministic
agent workflow.

- [x] Align project proposal, roadmap, SPEC, architecture, README, and ADRs to
  a browser-first product direction.
- [x] Establish repository-local agent harness and skills
  (`drl-milestone-delivery`, `drl-test-play`, `drl-determinism-review`,
  `drl-legacy-archaeology`).
- [x] Enforce single active slice in `SPEC.md` and serialized canonical writes.
- [x] Establish evidence-based testing vocabulary (`PASS`, `FAIL`,
  `INCONCLUSIVE`, `NOT_RUN`).
- [x] Record legacy behavior shells in `docs/legacy-behavior/` (`combat.md`,
  `movement.md`, `turn-economy.md`).
- [x] Enforce 2-space formatting, tab prohibition, and automated checks via
  `sh scripts/check-repository.sh`.
- [x] Implement strict `VERSION` tracking and transition checks via
  `scripts/check-version.sh`.

---

### M1 — Deterministic Simulation Kernel

Build the standalone, pure Rust game state and turn execution loop.

- [x] Pure Rust 2D grid maps (`Map`, `Tile`, `Position`, `Direction`).
- [x] Explicit, seedable PRNG (`GameRng` wrapping SplitMix64 + Xoshiro256++).
- [x] Semantic player commands (`Command::Move`, `Command::Wait`) and typed
  errors (`CommandError`).
- [x] Deterministic turn step execution (`Game::step`) with collision checks.
- [x] Ordered simulation event stream (`GameEvent`).
- [x] Replay execution engine (`ReplayEngine`) with bit-exact reproducibility.

---

### M2 — Turn Economy and Combat

Implement energy-based scheduling and deterministic tactical combat mechanics.

- [x] Energy-based actor scheduler (`Scheduler`) supporting variable actor
  speeds.
- [x] Pure combat calculation module (`CombatResolver`) for melee and ranged
  attacks.
- [x] Combat domain models (`HitPoints`, `Speed`, `ActionCost`, `DamageAmount`,
  `DamageType`, `DeathCause`).
- [x] Combat and turn events (`AttackResolved`, `DamageApplied`, `ActorDied`,
  `ActionCostPaid`).
- [x] Health management, armor mitigation, damage clamping, and death
  transitions.
- [x] Headless combat demonstration and deterministic replay verification in
  `drl-app`.

---

### M3 — Browser-Compatible Assets, Provenance, and Fidelity Evidence

Establish asset pipelines, licensing boundaries, and legacy capture manifests.

- [x] Dedicated `drl-assets` crate for platform-neutral atlas descriptors and
  geometry.
- [x] Import tracked legacy graphics from pinned Git revision
  (`17d9be1204751899b2d69d8d3a2dde247bd0cc5c`).
- [x] Complete CC BY-SA 4.0 licensing records, attribution, and SHA-256
  checksums in `MANIFEST.txt`.
- [x] Reference capture manifest tooling (`scripts/check-reference-capture.sh`).
- [x] Automated capture manifest preflight fixture tests
  (`scripts/test-reference-capture.sh`).
- [ ] Controlled legacy runtime captures in a rights-cleared Linux x86-64
  environment (currently `NOT_RUN` on macOS arm64).
- [ ] Validated capture-to-game fidelity comparison matrix.
- [ ] Rights clearance and asset tracking for audio, music, and fonts.

---

### M4 — Perception and Content Foundations

Implement perception rules, core items, monsters, and procedural level
progression.

- [x] Deterministic Field of View (FOV) and Line of Sight (LOS) raycasting.
- [x] Fog-of-war exploration memory for revealed tiles.
- [x] Fair observation filtering in `PlayerObservation` hiding unseen entities.
- [x] Bounded player inventory (`Inventory`) and equipment slots (`Equipment`).
- [x] Weapon mechanics: magazine clips, ammo consumption, reloading, and firing.
- [x] Consumable items: Small/Large MedPacks and emergency Phase Device.
- [x] Kinetic knockback pushing targets along firing vectors without obstacle
  clipping.
- [x] Tactical monster archetypes (`FormerHuman`, `FormerSergeant`, `Imp`,
  `Demon`) with AI and loot drops.
- [x] Procedural dungeon generator with BFS reachability and room connectivity.
- [x] Exit stairs interaction (`Command::Descend`) and multi-level player
  persistence.

---

### M5 — Replays, Scenarios, and Test Agents

Build headless infrastructure for testing, bot exploration, and scenario
validation.

- [x] Versioned replay log schema (`ReplayVersion::V1`) with diagnostic error
  locations.
- [x] Replay consistency validation (`ReplayEngine::validate`).
- [x] Declarative ASCII scenario fixture framework (`Scenario`,
  `ScenarioRunner`).
- [x] Observation-only test bot policies (`RandomBot`, `GreedyCombatBot`,
  `ExplorerBot`).
- [x] Headless batch simulation runner (`BatchRunner`) with statistical metric
  aggregation.
- [x] Deterministic batch sweep test suites and multi-turn scenario tests.

---

### M6 — MCP Semantic Interface

Provide standard Model Context Protocol interfaces for autonomous agents and
tooling.

- [x] Zero-dependency pure Rust JSON-RPC 2.0 protocol engine (`crates/drl-mcp`).
- [x] Standard MCP lifecycle methods (`initialize`, `ping`, `tools/list`,
  `tools/call`, `resources/list`, `resources/read`).
- [x] Complete semantic tool suite (`game_start`, `game_load_scenario`,
  `game_get_observation`, `game_list_actions`, `game_step_action`, `game_reset`,
  `game_get_metrics`, `game_save_replay`).
- [x] Strict information boundaries with explicit `dev_mode` flag for omniscient
  inspection.
- [x] Stdio transport runner and CLI integration (`drl-rust --mcp`).
- [x] Virtual AI player integration tests verifying determinism and safety.

---

### M7 — Browser-Playable M4 Slice

Deliver the initial interactive WebAssembly and WebGPU browser presentation.

- [x] `drl-web` WASM/WebGPU crate compiled via pinned `wasm-pack 0.15.0`.
- [x] Host-agnostic build, check, and serve scripts (`build-web.sh`,
  `serve-web.sh`, `check-web.sh`).
- [x] Pure `PresentationStep` and `RenderScene` transformations from simulation
  data.
- [x] Square-cell integer grid layout and viewport centering.
- [x] Accessible HTML/DOM shell with keyboard/numpad bindings and inventory
  controls.
- [x] Synthesized semantic audio cues and gesture-unlocked Web Audio mixer.
- [x] Graceful error recovery for unsupported WebGPU and suspended audio.
- [x] Remote web CI acceptance in Ubuntu headless Chrome.
- [ ] Capture-backed reference-scene comparison (pending reference captures).

---

### M8 — Audiovisual Parity

Achieve visual and acoustic equivalence with legacy presentation through
rigorous contracts.

#### Delivered Contracts

- [x] Pure `PixelViewport` integer cell layout and deterministic letterboxing.
- [x] Visibility-derived `LightingBand` (full light vs explored fog factor).
- [x] Fair health-derived `SceneTone` and pure low-health pulse alpha
  equations.
- [x] Event-ordered `EffectSpan` timing with visibility filtering.
- [x] Measured 16-column/32-pixel atlas slots and normalized UV conversion.
- [x] Registered layer metadata and shader input roles (base, colorization,
  outline, emissive).
- [x] Renderer-neutral `layer_draw_plan` and `sprite_composite_plan`.
- [x] Subpath-safe same-origin texture loading and manifest validation.
- [x] WebGPU linear `Rgba8Unorm` texture cache and external-image copy uploads.
- [x] Nearest-filtered base texture WGSL pass with emissive floor sampling.
- [x] Verified legacy `0.1` fragment alpha cutoff in textured shader.
- [x] Colorization-mask pass with evidence-backed tints (Green Armor, Phase
  Device, StairsDown).
- [x] Outline-mask GPU resource transport and straight-alpha compositing pass.
- [x] Pure sprite animation metadata (player, actors, Phase Device) and elapsed
  UV selection.
- [x] Browser `requestAnimationFrame` loop with visibility-lifecycle clock
  rebasing.
- [x] Pure post-process glow/LUT coordinate math, blur taps, blur reduction,
  and pass planning.
- [x] Pure animation arithmetic: explosion marks, cell effects, kill segments,
  FX frames, movement progress, missile steps/rays, and screen shake fade.
- [x] Pure particle contracts: burst origins, burst direction normalization/arc,
  range sampling, decal cell mapping, decal placement, and map eligibility.
- [x] Pure particle-decal insertion request (`ParticleDecalInsertion`).
- [x] Deterministic, caller-bounded `ParticleDecalStore` with explicit capacity
  enforcement.
- [x] Renderer-neutral particle-decal draw planning with opaque caller-resolved
  handles, stored-pixel placement, stable ordering, viewport filtering, and
  floor-level insertion into scene plans.

#### Present Slice (Expanded in `SPEC.md`)

- [x] Consume retained requests from `ParticleDecalStore::entries()` into
  renderer draw passes without mutating store or simulation state. Full
  capture-backed visual parity remains `NOT_RUN`.

#### Open Work

- [ ] Exact legacy outline/glow and lighting/LUT equations from approved
  reference captures.
- [ ] Broader tint sources and content animation/effect timing.
- [ ] Capture-backed particle decal visual regressions.
- [ ] HUD typography, layout, and minimap parity.
- [ ] Rights-cleared replacement audio and music tracks.
- [ ] Automated pixel-level and audio regression test harness.

---

### M9 — Typed Content Migration and Gameplay Breadth

Migrate legacy content into typed, immutable Rust definitions without runtime
scripting.

- [x] Rust-owned definitions for current monster archetypes
  (`MonsterKind::definition()`).
- [x] Immutable definition table for item spawn families and death drops.
- [x] Immutable roll-bound tables for procedural room loot and monster spawns.
- [x] Protocol-owned immutable tile definitions (`TileKind::definition()`).
- [x] Rust-owned standard procedural level generation policy.
- [ ] Build-time conversion tooling for legacy Lua content tables.
- [ ] Full migration of legacy monsters, weapons, armor, mods, and consumable
  items.
- [ ] Full migration of special levels, vaults, and dungeon branches.
- [ ] Validation gates for content fairness, determinism, replayability, and
  asset mappings.

---

### M10 — Browser Persistence and PWA State

Implement robust client-side save state and offline browser capabilities.

- [x] Versioned fixed-session command snapshot codec (`SessionSnapshot`).
- [x] Deterministic transactional replay restore from snapshots.
- [x] Best-effort browser `localStorage` save/load controls in `drl-web`.
- [x] Versioned static service-worker caching boundary.
- [ ] Full offline-after-first-load PWA lifecycle acceptance.
- [x] Corruption recovery policy: fail-closed restore, bounded quarantine, and
  playable boot/load when storage cleanup is unavailable.
- [ ] Replay-compatible save migration for explicitly recognized older formats.
- [x] Explicit non-goal: No online accounts or centralized backend services.

---

### M11 — Balance and Evaluation

Provide statistical evaluation tools to benchmark bot performance and gameplay
balance.

- [x] Headless fixed-seed cohort configuration (`CohortConfig`) and report
  generation (`CohortReport`).
- [x] Cohort report integrity validation (record count, seed order, summary
  coherence).
- [x] Descriptive cohort outcome distributions (victory, death, turn limit,
  stalled, in progress).
- [x] Pure compatible cohort comparisons and caller-owned outcome-rate tolerance
  gates.
- [x] Telemetry distributions (shot accuracy, damage, kills, pickups, items
  used) and delta comparisons.
- [ ] Automated large-scale balance and economic evaluations.
- [ ] Difficulty curve validation against canonical target metrics.
- [ ] Strict isolation between player observations and evaluation telemetry.

---

### M12 — Static Web Productization and Release Hardening

Harden the browser deployment for production hosting, accessibility, and
diagnostics.

- [x] Deterministic release manifest generation (`release-manifest.json`)
  recording version, source revision, sorted SHA-256 hashes, and license
  metadata.
- [x] Manifest SHA-256 digest sidecar (`release-manifest.sha256`).
- [x] Source-derived service worker cache naming and invalidation policy.
- [x] Mocked service-worker lifecycle and fetch contract test harness.
- [x] Git checkout-identity binding and source-identity validation.
- [x] Static HTML shell accessibility audit (landmarks, named controls,
  labels, focus, live regions).
- [x] Local accessible browser-support and startup diagnostics panel.
- [ ] Cryptographic release signing and integrity verification.
- [ ] Dynamic WCAG 2.1 AA and screen-reader accessibility acceptance.
- [ ] Real-world browser offline installation acceptance tests.
- [ ] Graceful fallback and diagnostics for untested browser environments.

---

### M13 — Browser-First 1.0 Release

Final release readiness, documentation, and static distribution.

- [ ] Production static HTTPS deployment for desktop Chromium (Chrome/Edge) with
  WebGPU.
- [ ] Approved audiovisual parity matrix verified against reference captures.
- [ ] Fully functional offline PWA installation.
- [ ] Complete deterministic headless/MCP agent tooling suite.
- [ ] Comprehensive public rights inventory and release documentation.

---

## 5. Post-1.0 Portability

Future work expanding platform and input support without compromising core
invariants:

- [ ] WebGL2 fallback renderer for older devices and unsupported browsers.
- [ ] Cross-browser validation for Firefox and Safari.
- [ ] Mobile/touch interface and responsive on-screen controls.
- [ ] Gamepad / controller input support.
- [ ] Native desktop application packaging for Linux, macOS, and Windows.

---

## 6. Delivery Gates & Verification

Every milestone and pull request must satisfy these automated gates:

- **Repository Integrity**: `sh scripts/check-repository.sh` (formatting,
  clippy, unit/integration tests, agent harness checks).
- **Asset Integrity**: `sh scripts/check-assets.sh` (manifest checksums and
  licensing).
- **Version Contract**: `scripts/check-version.sh` (valid `x.y.z` transition;
  no bumps on doc-only changes).
- **Web Contracts**: `scripts/check-web.sh` (WASM target build and native/WASM
  contract tests).
- **Release Manifest**: `sh scripts/build-web.sh && sh scripts/check-release-manifest.sh`
  (static artifact bundle validation).
- **Evidence**: `PASS`, `FAIL`, `INCONCLUSIVE`, and `NOT_RUN` are explicit; remote criteria are never marked complete from local
  inference.
