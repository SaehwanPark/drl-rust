# Changelog

All notable contributor- and user-visible changes to DRL-Rust will be
documented in this file.

## [0.2.12]

- Added fail-closed browser-save recovery: malformed, unsupported, oversized,
  and replay-invalid tokens are recorded in a bounded rejected-save slot and
  removed from the active load path when storage permits.
- Kept boot, load, and restart playable when storage cleanup fails; explicit
  replay-compatible format migration remains open.

## [0.2.11]

- Added renderer-neutral particle-decal draw planning with opaque,
  caller-resolved sprite handles, stored-pixel sub-cell placement, stable
  insertion/duplicate ordering, caller-resolved lighting, viewport filtering,
  and floor-level scene ordering.
- Added browser WebGPU entry points that consume retained decal plans without
  touching simulation state; capture-backed visual parity remains `NOT_RUN`.

## [0.2.10]

- Added deterministic, caller-bounded `ParticleDecalStore` retention for
  accepted decal requests. Insertion order and duplicates are preserved, and
  capacity overflow is reported explicitly without dropping prior entries.

## [0.2.9]

- Added the bounded M8 particle-decal insertion request: accepted placement
  coordinates now travel with the unchanged caller-provided sprite identifier,
  while sprite selection, decal storage, particle spawning, and rendering stay
  caller-owned.

## [0.2.8]

- Added the bounded M8 particle-decal eligibility contract: caller-resolved
  cells are eligible only when in bounds and neither liquid nor movement-
  blocking, while lookup, decal storage, and rendering remain caller-owned.

## [0.2.7]

- Added the bounded M8 particle-decal placement contract: the pure callback
  math now returns both one-based cell and pixel insertion coordinates.
- Continued retrospective project versioning at `0.2.7` from the untagged
  `0.1.0` baseline. `VERSION`, Cargo/MCP metadata, and release manifests are
  checked by the agent harness; code changes require one valid component
  increment while document- and setting-only changes do not.

## [0.2.6]

- Added the bounded M8 particle-decal cell contract: caller-rounded world
  positions use the legacy 16-pixel offset and truncating 32-pixel mapping,
  while map/flag eligibility and decal storage remain caller-owned.

## [0.2.5]

- Added the bounded M8 particle-burst range contract: caller-owned unit
  samples use the legacy affine min/max calculation, including reversed bounds
  and without hidden clamping or RNG ownership.

## [0.2.4]

- Added the bounded M8 particle-burst direction contract: requested XY
  directions are normalized, zero vectors clear only XY, and positive distance
  scales apply the legacy arc-to-Z adjustment without owning random sampling,
  decals, or a particle engine.

## [0.2.3]

- Added compatible M11 telemetry comparisons with separate caller-owned shot
  accuracy and per-episode average tolerances; deltas remain descriptive and
  do not claim balance or statistical significance.

## [0.2.2]

- Added an M11 cohort telemetry distribution for validated shot accuracy,
  damage, enemy-kill, pickup, and item-use totals plus descriptive rates;
  it makes no balance or statistical-significance claim.

## [0.2.1]

- Added M12 checkout-identity binding: release-manifest validation now matches
  a source revision to Git `HEAD` (or `DRL_BUILD_REVISION`) when available and
  retains `unknown` only for unverifiable source archives.

## [0.2.0]

- Added the bounded M8 outline-mask compositing pass: optional shadow layers
  now resolve behind base sprites with tested straight-alpha weights in the
  renderer-neutral contract and WebGPU shader. Exact legacy glow/outline
  equation and capture parity remain open.

## [0.1.7]

- Added an M12 release-manifest source-identity audit that accepts only the
  explicit `unknown` fallback or a lowercase 40-character Git object identity.

## [0.1.6]

- Added a dependency-free M12 service-worker lifecycle/fetch contract harness
  for precache, stale-cache cleanup, navigation fallback, and same-origin GET
  gating without claiming full browser-offline acceptance.

## [0.1.5]

- Added an M12 `release-manifest.sha256` sidecar with exact-byte verification
  and service-worker precaching; this is packaging integrity evidence, not a
  release signature or offline acceptance claim.

## [0.1.4]

- Strengthened M12 generated cache invalidation names to include both the
  project version and source-revision prefix; release-manifest checks verify
  the combined policy without claiming offline acceptance.

## [0.1.3]

- Added a caller-owned M11 outcome-rate tolerance gate that accepts only finite
  non-negative bounds and reports whether every category delta is within it.

## [0.1.2]

- Added pure compatible M11 outcome comparisons with absolute per-category
  rate deltas. Policy/sample mismatches and invalid reports are rejected, and
  the comparison does not add tolerance or significance claims.

## [0.1.1]

- Added pure M11 cohort outcome distributions with distinct victory, death,
  turn-limit, stalled, and in-progress counts plus sample-normalized rates;
  invalid reports are rejected before projection and no balance claim is made.

## [0.1.0] — Retrospective baseline (untagged history)

- Added M12 static-bundle release manifests generated during web builds, with
  source revision, sorted artifact SHA-256 hashes, generated-file declarations,
  graphics rights metadata, and a service-worker coverage check. The manifest
  is unsigned packaging evidence, not offline or cross-browser acceptance.
- Derived generated service-worker cache versions from the manifest source
  revision prefix and verified the generated worker against that manifest.
- Added a local accessible browser-support and startup-diagnostics panel for
  WebGPU, rendering, audio, and offline-cache failures without telemetry.
- Added a deterministic static accessibility audit for shell landmarks, named
  controls, labels, focus semantics, and live regions; dynamic WCAG acceptance
  remains open.
- Added pure M11 cohort regression comparisons with caller-declared finite
  non-negative win-rate and average-turn tolerances. Policy/sample mismatches
  are rejected, and no statistical or balance claim is inferred.
- Added pure M11 cohort-report integrity validation for record count, wrapping
  seed order, replay seed identity, and aggregate-summary coherence.
- Added M11 fixed-seed cohort reports around the existing headless batch
  runner. `CohortConfig` records seed range, sample size, and turn budget;
  `CohortReport` retains policy identity, aggregate metrics, and per-seed
  replay evidence for reproducible evaluation without claiming balance
  conclusions.
- Browser-first steering and playable-slice implementation: accepted ADRs
  0007/0008, reconciled proposal/roadmap/spec/architecture/README/contributor
  guidance, browser-aware agent harness rules, and dynamic repo-local skill
  validation.
- Added `drl-assets` with the complete tracked legacy graphics import,
  CC BY-SA attribution, pinned source revision, and SHA-256 manifest. Legacy
  audio/music/fonts remain redistribution-gated; controlled runtime captures
  are recorded as `NOT_RUN` on the arm64 macOS host, with a capture-to-M7/M8
  fidelity matrix.
- Added additive fair observation identifiers, pure `PresentationStep` and
  `RenderScene` builders, semantic audio cues, transactional `BrowserSession`,
  WASM/WebGPU/Web Audio bindings, accessible static browser shell, and web
  build/check/serve scripts. Native headless and MCP contracts remain intact.
- Verified the M7 functional gate with a Chrome 151 WebGPU smoke playthrough
  (Apple Metal-3, 1280x720, DPR 1; explicit gesture-gated audio state) and
  remote CI run `32538527707`; fixed startup and mute-control status races so
  the visible status reflects the actual or applied audio state, and serialized
  rapid audio-control events to prevent stale settings. Legacy reference-capture
  comparison remains open and explicitly `NOT_RUN`.
- Added the first bounded M8 presentation slice: pure `PixelViewport` layout
  math chooses centered integer square cells, and the WebGPU scene uses those
  rectangles for deterministic letterboxing. Focused render tests, local WASM
  compilation, native web contracts, asset checks, local browser smoke, and
  hosted run `32539486760` pass; capture-backed audiovisual parity remains
  `NOT_RUN`.
- Added the follow-up M8 lighting slice: pure `LightingBand`/`shade_color`
  rules derive full light versus fixed explored-tile fog from fair scene data,
  and WebGPU consumes the shared rule. Capture-backed lighting equivalence is
  still `NOT_RUN`.
- Moved the existing quarter-health WebGPU clear-color threshold into pure
  `drl-render::SceneTone`/`scene_clear_color` planning with focused tests; the
  browser renderer now consumes the shared tone rule.
- Added pure event-ordered `drl-render::EffectSpan` timing with fixed logical
  durations for presentation effects; frontend ticks cannot advance gameplay,
  and capture-backed animation timing remains `NOT_RUN`.
- Carried those ordered effect spans through successful browser
  `PresentationStep` results, so future frame mapping does not rebuild raw
  event semantics or cross the simulation boundary.
- Filtered browser effect spans against before/after visible actors while
  retaining direct player transitions, preventing hidden monster events from
  becoming presentation timing.
- Replaced placeholder atlas cells with measured 32-pixel legacy sprite slots
  for all current tile, actor, and item semantics. `drl-assets` now exposes
  imported PNG dimensions and pure rectangle bounds checks; texture compositing
  and capture-backed audiovisual parity remain open.
- Added deterministic registered source-layer metadata for every imported
  atlas and aligned semantic descriptors with their atlas-specific layer
  order. This is compositor input only; it does not claim blending or capture
  parity.
- Corrected the pinned legacy graphics revision everywhere to the exact
  40-character Git commit, keeping asset provenance and capture scripts
  reproducible.
- Added renderer-neutral normalized UV conversion for bounded sprite cells,
  with explicit top-left image origin and invalid-dimension rejection. Texture
  sampling remains a future compositor concern.
- Added `drl-render::layer_draw_plan`, a deterministic renderer-neutral plan of
  atlas layers, pixel destinations, and normalized UVs for fair scene sprites.
  It preserves explored tile memory and keeps texture upload/blending and
  capture-backed parity as future work.
- Added pure `AtlasTextureSource` bindings for every registered atlas layer;
  draw-plan entries now carry imported relative paths and measured dimensions
  for a future frontend upload boundary. No image loading or compositing is
  claimed.
- Carried the shared fair `LightingBand` into every `LayerDraw`, preserving the
  fixed explored-memory fog factor and full-light visible-sprite factor for a
  future compositor without exposing hidden state.
- Added explicit renderer-neutral `LayerRole` metadata for the legacy shader's
  base-color, colorization-mask, outline-mask, and emissive-mask inputs;
  texture upload, blend equations, and capture parity remain future work.
- Added `drl-render::sprite_composite_plan`, grouping complete role sets into
  one deterministic compositor input per fair scene sprite while rejecting
  malformed groups; GPU sampling and blend equations remain future work.
- Added a subpath-safe `drl-web::browser_asset_url` and WASM
  `load_texture_source` preflight that decodes same-origin imported PNGs and
  validates their manifest dimensions before any future GPU upload.
- Added a deterministic 24-source manifest and renderer-owned WASM WebGPU
  texture/view cache. Each validated decoded PNG is uploaded once with the
  external-image copy API; shader sampling and role compositing remain open.
- Added the first nearest-filtered base-color WGSL pass using grouped fair
  sprite UVs, source-specific bind groups, alpha blending, and shared lighting;
  geometry remains the fallback and mask/outline/emissive compositing remains
  future work.
- Added the bounded emissive-role follow-up: each registered sprite pairs its
  base source with an optional emissive source, samples the emissive red channel
  as a lighting floor, and uses a transparent 1x1 fallback when absent. Mask,
  colorization, outline/glow, and capture-backed shader parity remain open.
- Added the verified legacy `0.1` fragment-alpha cutoff to the textured WGSL
  pass; transparent edge fragments are discarded before source-alpha blending,
  while the fair/emissive lighting floor remains unchanged.
- Aligned renderer-owned atlas and transparent role-fallback storage with the
  observed legacy `GL_RGBA8` contract by using linear normalized
  `Rgba8Unorm`; browser display color-space parity remains capture-gated.
- Added a native contract test over the shared textured WGSL source, guarding
  base/emissive sampling, fair-lighting `max`, alpha cutout, and output terms
  while the runtime shader remains WASM-only.
- Added the bounded colorization-mask role to the textured WGSL pass. Optional
  mask views use the retained transparent fallback, and the current fair scene
  path supplies a neutral zero tint until per-sprite tint provenance is
  implemented; outline/glow and capture-backed shader parity remain open.
- Added pure `active_effect_frames` progress mapping for fair effect spans;
  frontend ticks receive stable normalized progress without advancing gameplay
  or claiming legacy animation-frame parity.
- Added pure `drl-render::animation_frame_index` groundwork that maps bounded
  normalized progress to a caller-supplied frame count; asset frame metadata,
  legacy timing, and capture-backed animation parity remain open.
- Added the bounded evidence-backed Green Armor colorization tint: visible
  ground items and the player with observed equipped Green Armor carry the
  pinned legacy green value through the existing mask vertex input, while
  other current archetypes and non-player actors remain neutral. Additional
  tint sources and capture-backed color/display parity remain open.
- Extended the colorization boundary to visible Phase Device ground items
  using the pinned legacy blue `coscolor`, explicitly quantized at the existing
  byte vertex boundary; player equipment and other roles remain unchanged, and
  capture-backed display-color parity remains open.
- Added bounded outline-mask GPU transport: optional shadow sources now travel
  from fair sprite composites into source-specific WebGPU bindings, with the
  transparent fallback for atlases without shadows. The WGSL contract receives
  the resource without blending it; visible outline/glow equations remain
  capture- and shader-evidence-gated.
- Added renderer-neutral animation metadata for the current player, actors, and
  Phase Device: pinned two-frame/500 ms descriptors, bounds-checked adjacent
  frame rectangles, and metadata preservation through layer draws/composites.
  Browser timing, broader content animation, and capture-backed parity remain
  open.
- Added caller-supplied normalized-progress layer planning: evidenced animated
  descriptors can select frame-specific UVs deterministically, while the
  existing frame-zero plan and browser output remain unchanged.
- Added pure elapsed-millisecond animation frame selection with explicit loop
  and clamp policies over validated sprite metadata; callers retain ownership
  of clocks, browser scheduling, and sprite/effect association.
- Added caller-supplied elapsed-time layer planning: animated descriptors select
  frame UVs under explicit playback policy, static descriptors remain on frame
  zero, and grouped composites retain one selected UV per sprite.
- Added a caller-driven `WebGpuRenderer::render_at_elapsed` entrypoint that
  forwards elapsed layer plans into textured vertex generation while preserving
  the existing frame-zero render and geometry fallback paths.
- Added a bounded browser `requestAnimationFrame` loop that converts callback
  timestamps to elapsed milliseconds, skips hidden-document presentation, and
  never advances simulation or owns effect timing.
- Hardened the browser animation lifecycle with one idempotent
  `visibilitychange` listener that rebases the presentation clock even when
  hidden RAF callbacks are throttled; listener failure remains non-fatal to
  event-driven gameplay.
- Added the pinned yellow `StairsDown` tile `coscolor` through the existing
  colorization-mask path. Other current tile kinds, actors, and item
  archetypes remain neutral unless already evidence-backed.
- Added a read-only legacy-capture manifest preflight and fixture suite. It
  checks the pinned revision, required fidelity scenes, executable/hash
  metadata, status vocabulary, placeholder policy, and recorded checkout
  dirty-state while preserving `NOT_RUN` on hosts without the controlled Linux
  x86-64 runtime. Promotable capture statuses now require a clean checkout and
  directly observed evidence classification.
- Added capture-attestation checks for explicit rights status and
  `sha256:<64-hex>` media hashes; a `PASS` capture now requires cleared rights
  and valid hashes while unresolved environments remain `NOT_RUN` or
  `INCONCLUSIVE`.
- Added pure `drl-render::low_health_pulse_target_alpha` planning from the
  pinned legacy one-third-health threshold and five-radian-per-second phase.
  The helper consumes caller-supplied elapsed time and exposes only the
  instantaneous bounded target; legacy smoothing, low-life compositing, and
  capture-backed LUT/glow parity remain open.
- Added pure `drl-render::low_health_pulse_state_step` planning for the pinned
  `aMSec / 500` move-toward smoothing and independent pending-target decay.
  State and elapsed time remain caller-owned; draw-time clamping, texture
  compositing, and capture-backed parity remain open.
- Added pure post-process glow contracts: the pinned declared/effective blur
  weights (including the source's unused declared entries),
  `post_process_glow_color`'s `1.6 * blur alpha` add, and the channel-swizzled
  clamped LUT coordinate. Offscreen blur/LUT integration, outline blending,
  and capture-backed color parity remain open.
- Added `post_process_blur_taps` with explicit horizontal/vertical normalized
  offsets, effective weights, center-alpha index, and zero-dimension rejection;
  it remains a renderer-neutral plan with no texture sampling or render-pass
  ownership.
- Added pure `post_process_blur_rgba` reduction for weighted RGB and
  center-only alpha, preserving the tracked shader's no-renormalization and
  no-clamp behavior.
- Added pure `post_process_pass_plan` sequencing for direct scene draws and the
  observed captured-scene, optional horizontal/vertical blur, and composite
  stages across glow/LUT gate combinations. It owns no GPU resources, sampling,
  or capture-parity claim.
- Added pure `drl-render::explosion_mark_phase` selection for the pinned
  three-bucket integer effect rule, including zero-duration normalization,
  overflow-safe arithmetic, and the observed post-duration second-phase
  fallback. Delay scheduling, palette mapping, sprite rendering, and capture
  parity remain outside the helper.
- Added pure `drl-render::effect_segment_index_at_elapsed` arithmetic for the
  pinned cell/item signed quotient and sign correction, with explicit
  zero-duration and out-of-range rejection. Sprite, level, item, lifecycle,
  and capture behavior remain outside the helper.
- Added pure `drl-render::kill_animation_segment_index_at_elapsed` arithmetic
  for the pinned lead-delay/reverse branch, integer segment quotient, and
  terminal clamp, with explicit invalid metadata rejection. Actor, sprite,
  light, lifecycle, and capture behavior remain outside the helper.
- Added pure `drl-render::fx_animation_frame_index_at_elapsed` arithmetic for
  the pinned FX frame quotient and terminal clamp, with explicit zero-duration
  and zero-frame rejection. Sprite IDs, atlas columns, lifecycle, and capture
  behavior remain outside the helper.
- Added pure `drl-render::move_animation_progress_at_elapsed` arithmetic for
  the pinned normalized elapsed movement ratio and `[0, 1]` clamp, with
  explicit zero-duration rejection. Coordinates, lighting, entity state,
  lifecycle, interpolation, and capture behavior remain outside the helper.
- Added pure `drl-render::missile_step_index_at_elapsed` arithmetic for the
  pinned minimum-normalized step delay and elapsed quotient, with explicit
  rejection of unrepresentable `u16` step indexes. Path traversal, visibility,
  particles, lifecycle, and capture behavior remain outside the helper.
- Added pure `drl-render::missile_ray_sample_distance_at_index` arithmetic for
  the pinned strict pre-increment half-grid test and fixed 20-unit ray spacing,
  preserving the source's possible endpoint overshoot with checked arithmetic.
  Endpoint metrics, visibility, particles, rendering, lifecycle, and capture
  behavior remain outside the helper.
- Added pure `drl-render::screen_shake_fade_at_elapsed` timing for the pinned
  quadratic active envelope and zero-at-expiry guard. Random frequencies,
  offsets, strength/direction scaling, scheduling, rendering, lifecycle, and
  capture behavior remain outside the helper.
- Added pure `drl-render::particle_burst_origin_at_legacy_cell` arithmetic for
  the pinned one-based 32-pixel cell-center conversion with checked signed
  overflow handling. Direction, random burst parameters, decals, particle
  engine state, rendering, lifecycle, and capture behavior remain outside.
- Added the Rust-owned `MonsterKind::definition()` table for the four current
  archetypes and routed actor factories/generated spawns through it, including
  knockback and death-drop metadata. Existing values and replay schemas remain
  unchanged; divergent legacy numeric values are not claimed as migrated.
- Routed game death drops through the existing `Item::from_spawn_kind` factory
  and added nine-variant coverage; item payloads, IDs, events, and replay
  behavior remain unchanged.
- Added an immutable Rust-owned definition table for the nine current item
  spawn families and routed convenience factories through it. Ammo payloads,
  current properties, replay V1, and protocol schemas remain unchanged; this
  does not claim legacy numeric parity.
- Centralized the six current procedural room-loot outcomes in an immutable
  roll-bound table while preserving thresholds, fixed ammo payloads, RNG/ID
  ordering, and canonical item construction. This is not a balance or legacy
  parity claim.
- Centralized the four current procedural monster-roll outcomes in an immutable
  roll-bound table while preserving one-roll RNG consumption, spawn metadata,
  ordering, and `MonsterKind` definitions. This is not a balance or legacy
  parity claim.
- Added a protocol-owned immutable definition table for the five current tile
  semantics and routed core tile physical flags through it. Map behavior,
  observations, replay V1, and protocol schemas remain unchanged; no legacy
  parity is claimed.
- Added one immutable Rust-owned `standard-procedural` level definition for
  the existing core default and level-descent policy. Custom generator
  configurations and the MCP five-room policy remain unchanged; no topology,
  balance, or legacy-parity claim is made.
- Added a versioned fixed-session snapshot codec with strict size/corruption
  handling, deterministic replay restore, and best-effort WASM localStorage
  save/load controls. The static bundle now includes a manually versioned
  same-origin service-worker cache and manifest. Replay V1 and authoritative
  simulation schemas remain unchanged.

- `CONTRIBUTING.md` added at the repository root, covering workspace crate
  map, prerequisites, code style (2-space indent, `rustfmt`, `clippy`), branch
  naming and commit conventions, pull request workflow, local check procedure
  (`sh scripts/check-repository.sh`), and architectural do-not-cross rules.
- `docs/adr/` directory created with six initial Architecture Decision Records:
  - `0001` — Project architecture principles (functional-core/imperative-shell,
    typed domain, ADTs, explicit state, no ambient state, clean boundaries,
    testability, no premature abstraction);
  - `0002` — No legacy backward compatibility (no saves, mods, WAD, or RNG
    stream compatibility with the Pascal implementation);
  - `0003` — Semantic command model (all clients submit `Command` through the
    same simulation API; no privileged mutation paths);
  - `0004` — Explicit deterministic RNG (`GameRng` wraps SplitMix64 +
    Xoshiro256++; no global or ambient RNG in `drl-core`);
  - `0005` — Lua transitional strategy (Lua behind a narrow typed boundary;
    Rust owns all simulation invariants; Lua errors are isolated);
  - `0006` — MCP semantic interface strategy (MCP as first-class agent/test
    interface via JSON-RPC 2.0 stdio; not a simulation bypass; player
    information boundaries enforced; replay determinism preserved).
- `docs/legacy-behavior/` directory created with four documents:
  - `_template.md` — reusable template distinguishing verified behaviors,
    inferred design intent, legacy implementation artifacts, deliberate
    DRL-Rust decisions, and open questions;
  - `movement.md` — movement semantics shell covering grid movement, bounds
    enforcement, occupancy, diagonal movement, level exit, and action cost;
  - `turn-economy.md` — action-cost semantics shell covering the energy-based
    scheduling model, actor speed, action cost uniformity, and dead actor
    handling;
  - `combat.md` — combat semantics shell covering hit resolution (accuracy
    roll, range penalty, LOS requirement), damage calculation (uniform roll,
    armor mitigation, HP clamping), death, knockback, and loot drops.
- Roadmap progress table updated: M0 status corrected to "Complete";
  M1, M2, M4, M5, M6 statuses updated to "Complete" with delivery summaries.
- M0 roadmap checklist items marked complete: `CONTRIBUTING.md`, `docs/adr/`,
  `docs/legacy-behavior/`, the three implemented behavior shells
  (`combat.md`, `movement.md`, `turn-economy.md`), behavior-spec template, and
  the harness/documentation checks. The earlier “six behavior areas” wording
  was corrected; three shells are present.
- `ARCHITECTURE.md` updated to document `docs/adr/` and `docs/legacy-behavior/`
  as recognized structural components.

- Full Model Context Protocol (MCP) server implementation (`crates/drl-mcp`) providing machine-operable semantic
  gameplay interfaces for AI-driven testing, playtesting, and evaluation.
- Zero-dependency JSON-RPC 2.0 communication engine in pure Rust `std` (`drl_mcp::json`, `drl_mcp::protocol`) supporting
  standard MCP protocol methods (`initialize`, `ping`, `tools/list`, `tools/call`, `resources/list`, `resources/read`).
- Semantic tool suite:
  - `game_start`: initialize seeded procedural dungeon sessions with configurable dimensions and turn limits;
  - `game_load_scenario`: parse and load declarative ASCII scenario layouts;
  - `game_get_observation`: retrieve fair player-visible world views (FOV tiles, visible actors, inventory, equipment);
  - `game_list_actions`: dynamically synthesize available legal actions (`Move`, `AttackRanged`, `Reload`, `Pickup`,
    `Use`, `Equip`, `Unequip`, `Drop`, `Wait`, `Descend`);
  - `game_step_action`: execute semantic actions directly through the simulation core;
  - `game_reset`: reset session back to starting configuration;
  - `game_get_metrics`: fetch real-time episode telemetry and terminal outcomes;
  - `game_save_replay`: export deterministic session replay logs;
  - `game_get_dev_state`: developer-only omniscient world state inspection gated by explicit `dev_mode` flag.
- Static and dynamic game resources (`drl://rules/game`, `drl://rules/actions`, `drl://session/metrics`, `drl://session/events`).
- Stdio transport runner (`McpServer::run_stdio`) and CLI integration in `drl-app` (`drl-rust --mcp` or `drl-rust mcp`).
- Comprehensive MCP integration test suites (`protocol_jsonrpc.rs`, `tools_gameplay.rs`, `security_and_fairness.rs`,
  `virtual_ai_player.rs`) verifying information boundaries, error handling, tool workflows, and bit-exact replay determinism.
- Completion of Milestone 6: MCP Game Interface deliverables and exit criteria.

- Versioned replay log schema (`ReplayVersion::V1`, `ReplayMetadata`) in `drl-protocol` and
  `drl-core` supporting engine version headers, custom player spawn configurations (`PlayerSpawnConfig`),
  and explicit tile override maps.
- Diagnostic replay error reporting with `ReplayExecutionError` capturing exact turn numbers,
  0-based command indices, failed commands, and underlying simulation error contexts.
- Replay validation engine (`ReplayEngine::validate`) ensuring all coordinates, bounds, entities,
  items, and stairs are physically consistent prior to execution.
- Declarative scenario fixture framework (`Scenario`, `ScenarioFixture`, `ScenarioMap`) in `drl-protocol`
  and `drl-core` supporting multi-room ASCII map parsing (`Scenario::from_ascii`), custom monster/item placements,
  starting equipment, stairs configurations, and fluent assertion runners (`ScenarioRunner`).
- Scripted test agent policies (`AgentPolicy` trait) consuming strictly `PlayerObservation` and emitting
  `Command`s without information leakage:
  - `RandomBot`: uniform random selection among legal walkable directions and interactions;
  - `GreedyCombatBot`: tactical survival bot prioritizing health restoration, weapon reloading,
    line-of-fire checks, ranged and melee engagements, item looting, and exit stairs descent;
  - `ExplorerBot`: goal-directed exploration bot navigating uncharted maze corridors and descending stairs.
- Headless batch simulation runner (`BatchRunner`) executing high-throughput procedural and scenario sweeps
  across arbitrary seeds with configurable episode limits, recording `EpisodeRecord` artifacts, and aggregating
  statistical summaries (`BatchSummary`: win rates, average turns, total kills, damage dealt/taken).
- Runtime metrics accumulation (`RunOutcome`, `EpisodeMetrics`) tracking completion status, damage telemetry,
  kill distributions, item pickups, ammo expenditure, and level progression.
- Integration test suites in `crates/drl-core/tests/`:
  - `scenarios.rs`: ASCII grid parsing, custom hero loadouts, and multi-step scenario metrics;
  - `agents.rs`: headless policy execution, combat room clearing, maze navigation, and bit-exact replay determinism;
  - `batch_simulation.rs`: multi-seed procedural batches, statistical validation, and sweep determinism;
  - `replay_versioning.rs`: metadata header validation, boundary rejection, and diagnostic error locations.
- Headless application runner update in `drl-app` executing declarative scenario fixtures, automated bot play,
  batch sweeps, and replay determinism verification.
- Completion of Milestone 5: Replay, Scenario, and Test-Agent Infrastructure deliverables and exit criteria.

- Weapon kinetic knockback mechanics in `drl-core` (`apply_knockback`) and `drl-protocol`
  (`GameEvent::ActorKnockedBack { entity_id, from, to }`), enabling pump-action Shotgun
  and Former Sergeant shotgun attacks to push surviving targets 1 tile backwards along the firing vector.
- Map boundary, terrain obstacle, and actor collision checks for knockback resolution, ensuring
  actors never clip into walls, out-of-bounds cells, or occupied tiles.
- Weapon property `knockback: u32` in `WeaponProperties` and `ItemView`, configured with 1 for
  `Item::shotgun` and `Actor::former_sergeant`.
- Immediate FOV and fog-of-war exploration updates when the player character is knocked back by enemies.
- Comprehensive statistical test suite in `crates/drl-core/tests/stochastic_combat.rs` validating
  empirical accuracy scaling across distances, 3-sigma confidence intervals, uniform damage distributions,
  and bit-exact multi-turn knockback replay determinism.
- Completion of Milestone 4: Core DRL Gameplay Vertical Slice roadmap deliverables and exit criteria.
- Headless demo runner update in `drl-app` displaying real-time kinetic knockback event telemetry.

- Enemy archetypes domain and factory constructors in `drl-protocol` and `drl-core`

  (`FormerHuman`, `FormerSergeant`, `Imp`, `Demon`) with distinct health, speed,
  melee, ranged attack ranges, accuracies, and death loot drop tables.
- Tactical Monster AI decision module (`ai`) in `drl-core` (`MonsterAi::decide_action`)
  supporting adjacent melee attacks, ranged projectile/fireball attacks with line-of-sight checks,
  and pathfinding pursuit towards the player.
- Targeting system module (`targeting`) in `drl-core` (`TargetingSystem`) providing
  pure validation for `Target::Position`, `Target::Entity`, and `Target::Direction`
  with out-of-bounds, range limit, and line-of-sight obstruction checks, as well as visible
  target listing and nearest enemy auto-targeting.
- Special-use consumable item `Phase Device` in `drl-protocol` and `drl-core` allowing
  emergency spatial relocation to random walkable unoccupied cells, updating FOV and fog of war.
- Monster death loot drop mechanics spawning floor items upon lethal combat resolution
  at the monster's exact position and emitting `GameEvent::ItemDropped`.
- Semantic protocol event `GameEvent::PlayerTeleported { from, to }`, `Target` enum,
  `MonsterKind` enum, and `ItemCategory::PhaseDevice`.
- Replay logging support for ranged monster specs with builder `with_ranged_combat`
  and `ItemSpawnKind::PhaseDevice`.
- Integration test suites in `crates/drl-core/tests/monsters_ai.rs`, `crates/drl-core/tests/special_items.rs`,
  and `crates/drl-core/tests/targeting.rs` verifying tactical AI behaviors, Phase Device safety,
  target validation, and bit-exact replay determinism.
- Headless demo runner update in `drl-app` demonstrating tactical ranged monster combat,
  loot drops, Phase Device emergency teleportation, and replay determinism.


- Procedural dungeon level generator (`generator`) in `drl-core` producing bounded
  maps with non-overlapping rectangular rooms connected by walkable L-shaped and
  straight corridors, border walls, and entry/exit placements.
- Invariant reachability and connectivity validation using Breadth-First Search (BFS)
  guaranteeing walkable paths from player spawn to down-stairs and between all rooms.
- Room-based entity and loot distribution spawning representative monsters (Former
  Humans, Imps) and floor items (9mm ammo, Shotgun shells, MedPacks, Shotguns, Armor).
- Exit stairs interaction and level transitions via `Command::Descend`, validating
  stairs presence with `CommandError::NotOnStairs` and transitioning the world to `LevelId(n + 1)`.
- Player state preservation across level transitions, carrying over player health,
  inventory backpack, equipped weapons/armor, clip ammunition, and energy into new levels.
- Replay recording and playback support for down-stairs positions (`ReplayLog.initial_stairs`)
  and multi-level command streams with bit-exact reproducibility.
- Semantic protocol event `GameEvent::LevelTransitioned { from_level, to_level }`
  and command `Command::Descend`.
- Comprehensive integration test suite in `crates/drl-core/tests/level_progression.rs`
  verifying procedural generator connectivity, stairs validation, player state retention,
  and multi-level replay determinism.
- Headless demo runner update in `drl-app` demonstrating combat, floor looting, stairs
  descent, level transition from Level 1 to Level 2, and replay determinism.

- Item domain model (`item`) in `drl-core` and `drl-protocol` with physical item
  properties, weapons, body armor, ammunition stacks, and consumables.
- Bounded player inventory (`Inventory`) with automatic ammunition stacking, stack
  draining, and capacity enforcement.
- Equipment system (`Equipment`) supporting dedicated weapon and armor slots,
  equipment swapping, and unequip validation.
- Weapon and ammunition mechanics with magazine clip tracking, ammo consumption
  on ranged attacks, clip exhaustion errors (`CommandError::NoAmmoInClip`), and
  reloading (`Command::Reload`) from reserve inventory ammo stacks.
- Representative weapons: Pistol (9mm caliber, 10-round clip), Shotgun (Shells,
  8-round clip), Combat Knife (melee).
- Representative items: 9mm Ammo, Shotgun Shells, Small MedPack (+10 HP), Large
  MedPack (+25 HP), Green Armor (+5 armor protection).
- Ground item tracking in `World` with deterministic `BTreeMap` storage, floor loot
  spawning, pickup (`Command::Pickup`), and dropping (`Command::Drop`).
- Perception filtering for ground items, exposing only floor items on explored
  fog-of-war tiles in `PlayerObservation.ground_items`.
- Armor damage protection mitigation reducing raw incoming damage in combat.
- Replay recording and playback support for initial item spawns (`ItemSpawnSpec`).
- Comprehensive integration test suite in `crates/drl-core/tests/inventory.rs`
  verifying pickups, drops, capacity limits, equip/unequip cycles, medpack use,
  weapon firing, and reload cycles.
- Headless demo runner update in `drl-app` demonstrating item pickups, weapon
  swapping, ranged combat, and healing with bit-exact replay determinism.

- Field of View (FOV) calculation and Line of Sight (LOS) ray tracing module
  (`fov`) in `drl-core` supporting deterministic perimeter raycasting, obstacle
  occlusion, and transparency checks.
- Fog-of-war map exploration memory in `World` tracking explored tiles and
  revealing previously seen terrain.
- Perception filtering in `PlayerObservation` strictly hiding unobserved entities
  and monsters behind obstacles or outside the active field of view.
- Line-of-fire obstacle checks for ranged attacks (`Command::AttackRanged`),
  rejecting blocked shots with `CommandError::LineOfSightBlocked`.
- Extended `TileView` in `drl-protocol` with `is_visible` flag distinguishing
  active FOV cells from remembered fog-of-war cells.
- End-to-end integration test suite in `crates/drl-core/tests/visibility.rs`
  verifying shadowcasting, fog-of-war exploration persistence, entity filtering,
  and line-of-fire validation.
- Headless demo update in `drl-app` displaying active FOV and explored fog-of-war
  tile metrics per turn.
- Action economy and energy-based actor scheduling system (`Scheduler`) in `drl-core`
  supporting relative actor speeds and deterministic turn ordering.
- Pure, deterministic combat calculation module (`CombatResolver`) in `drl-core`
  resolving melee and ranged attacks with explicit seedable RNG.
- Melee bump-attacks, direct melee attacks (`Command::AttackMelee`), and targeted
  ranged attacks (`Command::AttackRanged`) with range and obstacle validation.
- Domain models in `drl-protocol` for combat stats (`HitPoints`, `Speed`, `ActionCost`,
  `DamageAmount`, `DamageType`, `DamageSource`, `DeathCause`, `AttackOutcome`).
- Combat and scheduling events (`GameEvent::AttackResolved`, `GameEvent::DamageApplied`,
  `GameEvent::ActorDied`, `GameEvent::ActionCostPaid`).
- Autonomous monster AI turn execution during scheduled energy intervals, reacting
  to player positions and executing attacks.
- Actor health tracking, damage deduction with clamping, death state transitions,
  and dead actor occupancy unblocking.
- Replay support for monster spawns and combat command streams via `MonsterSpawnSpec`.
- Headless combat demonstration in `drl-app` running a multi-turn tactical scenario
  and verifying bit-for-bit replay determinism.
- Comprehensive unit and end-to-end integration test suites in `crates/drl-core/tests/combat.rs`.
- Headless simulation kernel (`drl-core`) with deterministic seedable `GameRng`
  (SplitMix64 + Xoshiro256++), 2D bounded tile maps (`Map`, `Tile`), and physical
  world state (`World`) with deterministic entity storage.
- Shared semantic protocol contracts (`drl-protocol`) including domain types
  (`Position`, `Direction`, `Turn`, `EntityId`, `ItemId`, `LevelId`), commands
  (`Command::Move`, `Command::Wait`), typed errors (`CommandError`), events
  (`GameEvent`), observations (`Observation`, `TileView`, `ActorView`), and replay
  logs (`ReplayLog`).
- Deterministic turn loop execution kernel (`Game::step`) with movement validation,
  collision detection against terrain and entities, and ordered event emission.
- Deterministic replay execution engine (`ReplayEngine`) and validation tests
  verifying bit-for-bit identical state reproduction across independent runs.
- Executable headless simulation demonstration in `drl-app` running a multi-step
  scenario and verifying replay determinism.
- Comprehensive unit and end-to-end integration tests for movement, terrain bounds,
  occupancy collisions, PRNG reproducibility, and observation snapshots.
- Multi-crate Cargo workspace managing `drl-core`, `drl-protocol`, `drl-app`,
  `drl-script`, `drl-mcp`, `drl-render`, and `drl-audio`.
- Deterministic headless simulation core library (`drl-core`) and shared
  protocol contract library (`drl-protocol`).
- Default workspace application executable (`drl-app` / `drl-rust`).
- Automated architectural boundary tests ensuring `drl-core` and `drl-protocol`
  remain free of presentation, audio, and MCP dependencies.
- A repo-local milestone-delivery harness with durable repository guidance.
- A staged development and test-play team contract with explicit ownership,
  deterministic handoffs, and bounded delegation.
- Reusable legacy-archaeology, capability-gated test-play, and independent
  determinism-review skills.
- Repository checks for skill structure, required harness paths, and handoff
  and result-status vocabulary.
- Lightweight specification, architecture, and changelog documents governed by
  the canonical project roadmap.
- Dependency-light two-space formatting checks shared by local development and
  macOS CI.
- Contributor-facing README guidance for the current scaffold, project
  direction, legacy research setup, and licensing boundaries.
