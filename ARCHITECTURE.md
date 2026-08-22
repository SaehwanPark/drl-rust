# Architecture

Last reviewed: 2026-08-22
Current project version: `0.2.12`

Status: Verified for current deterministic headless core, MCP tooling, and
browser-playable WebGPU slice; full audiovisual parity remains planned.

---

## 1. Core Architectural Principles

DRL-Rust reimplements *Doom the Roguelike* with modern software engineering
invariants:

- **Functional Core, Imperative Shell**: Pure, deterministic game logic in
  `drl-core`; all side effects (WebGPU, Web Audio, DOM, MCP, I/O) are confined
  to the outer boundary crates (`drl-web`, `drl-audio`, `drl-app`).
- **Strict Determinism & Replayability**: Seedable PRNG (`GameRng`), explicit
  command-driven turn execution, zero ambient state, and bit-exact replay
  verification.
- **Fair Information Boundaries**: Frontends and AI agents consume only fair
  `PlayerObservation` views (active FOV, explored fog memory, visible entities);
  internal `World` state is never exposed to clients.
- **Zero External Dependencies in Core**: `drl-core` and `drl-protocol` are pure
  Rust crates with zero dependencies on WebGPU, Web Audio, DOM, filesystem,
  network, or MCP libraries.
- **No Runtime Scripting**: Lua is treated as build-time reference and
  conversion evidence only; no Lua runtime exists in the WASM browser bundle.

---

## 2. System Boundaries & Data Flow

```text
HTML / DOM UI / Keyboard / MCP Tool Call
  │
  ▼
drl-protocol::Command (semantic input)
  │
  ▼
drl-core::Game::step (deterministic simulation authority)
  │
  ├─► drl-protocol::PlayerObservation (fair FOV/fog/entity state)
  ├─► drl-protocol::GameEvent (ordered simulation events)
  │
  ▼
Presentation Boundary
  ├─► drl-render::RenderScene ──► drl-web WebGPU Canvas
  └─► drl-audio::AudioCue   ──► drl-audio Web Audio Mixer
```

### Data Flow Invariants

1. **Client Input**: All clients (headless tests, MCP agents, browser DOM,
   future native apps) interact with the game exclusively by submitting
   `drl-protocol::Command` values.
2. **Simulation Authority**: `drl-core` is the sole authority for world state,
   action legality, action costs, energy scheduling, PRNG consumption, and
   event emission.
3. **One-Way Presentation**: Presentation layers (`drl-render`, `drl-audio`,
   `drl-web`) consume observations and events only. Rendering, animation, audio,
   tab visibility, viewport resize, or GPU device loss **never** advance the
   simulation or alter PRNG streams.
4. **Transactional Rollback**: Illegal or rejected commands roll back the
   session checkpoint without advancing turn counters or modifying world state.

---

## 3. Workspace Crates

### `drl-protocol` — Shared Semantic Contracts
- **Role**: Stable semantic boundary shared across core, renderers, MCP, and
  frontends.
- **Key Modules & Types**:
  - Domain primitives: `Position`, `Direction`, `Turn`, `EntityId`, `ItemId`,
    `LevelId`.
  - Commands & Errors: `Command`, `CommandError`.
  - Observations: `PlayerObservation`, `TileView`, `ActorView`, `ItemView`.
  - Events: `GameEvent` stream (combat, movement, items, levels).
  - Typed Content: `MonsterKind::definition()`, `TileKind::definition()`.
  - Replay contracts: `ReplayVersion::V1`, `ReplayLog`.
- **Dependencies**: Pure `std` only; zero dependencies on any other workspace
  crate.

### `drl-core` — Deterministic Simulation Kernel
- **Role**: Pure simulation authority and headless test/evaluation engine.
- **Key Modules & Subsystems**:
  - Simulation & Maps: `Map`, `Tile`, `World`, `Game::step`.
  - PRNG: `GameRng` (deterministic SplitMix64 + Xoshiro256++).
  - Combat & Scheduling: `Scheduler`, `CombatResolver`, kinetic knockback.
  - Perception & AI: Field of View (`fov`), Line of Sight, `MonsterAi`.
  - Items & Inventory: `Inventory`, `Equipment`, `Item::from_spawn_kind`.
  - Level Generation: `generator` (BFS reachability, room connectivity).
  - Content Definitions: `item_definition`, `loot_definition`,
    `monster_roll_definition`, `level_definition`.
  - Evaluation & Cohorts: `CohortConfig`, `CohortReport`, `BatchRunner`,
    integrity validation, outcome distributions, telemetry projections.
- **Dependencies**: Depends only on `drl-protocol`.

### `drl-assets` — Atlas Descriptors & Provenance
- **Role**: Platform-neutral graphics atlas descriptors, geometry, and license
  metadata.
- **Key Responsibilities**:
  - Measured 16-column / 32-pixel sprite sheet cell coordinates.
  - Normalized UV math with top-left origin.
  - Registered source-layer metadata and legacy shader roles (`base`,
    `colorization`, `outline`, `emissive`).
  - CC BY-SA 4.0 licensing records and SHA-256 asset checksums.
- **Dependencies**: No image decoders or platform libraries; core does not
  depend on it.

### `drl-render` — Pure Presentation Planning
- **Role**: Deterministic renderer-neutral scene construction, layout, and
  timing math.
- **Key Responsibilities**:
  - Scene Construction: `PresentationStep`, `RenderScene`, target selection.
  - Viewport Layout: `PixelViewport`, `PixelRect` integer square-cell scaling.
  - Shading & Tone: `LightingBand` (FOV vs fog), `SceneTone` (player health),
    `low_health_pulse_target_alpha`, `LowHealthPulseState`.
  - Draw Plans: `layer_draw_plan`, `sprite_composite_plan`, `AtlasTextureSource`.
  - Animation & Effects: `active_effect_frames`, elapsed-time frame selection,
    pure math for explosion marks, cell effects, kill segments, FX, movement,
    missile steps/rays, and screen shake.
  - Particles & Decals: Burst origins, directions, range sampling, decal cell
    mapping, decal placement/eligibility, `ParticleDecalInsertion`, and
    caller-bounded `ParticleDecalStore`.
- **Dependencies**: Depends on `drl-protocol` and `drl-assets`. No GPU or window
  dependencies.

### `drl-audio` — Semantic Audio Engine
- **Role**: Deterministic event-to-audio mapping and Web Audio mixer.
- **Key Responsibilities**:
  - Pure mapping from `GameEvent` to semantic `AudioCue`.
  - WASM Web Audio synthesizer with gesture unlock, volume, and mute controls.
- **Dependencies**: Depends on `drl-protocol`.

### `drl-web` — Browser Shell & WebGPU Presentation
- **Role**: WASM `cdylib` / `rlib` browser host, WebGPU renderer, and PWA shell.
- **Key Responsibilities**:
  - Browser session management (`BrowserSession`) and DOM/keyboard mapping.
  - WebGPU pipeline: texture cache, linear `Rgba8Unorm` storage, nearest base
    sampling, emissive lighting floor, `0.1` alpha cutoff, colorization tints,
    and outline-mask straight-alpha compositing.
  - Browser animation loop: `requestAnimationFrame` driving elapsed rendering
    with `visibilitychange` clock rebasing.
  - State Persistence: `SessionSnapshot` codec with localStorage save/load.
    Rejected values are quarantined in a bounded browser-owned slot before
    active storage cleanup; future version migration is explicit and gated.
  - Release Packaging: Service worker caching, release manifest validation,
    digest sidecars, and checkout-identity verification.
  - Accessibility: Accessible DOM shell, keyboard/numpad navigation, and
    diagnostics panel.
- **Dependencies**: Depends on `drl-protocol`, `drl-render`, `drl-assets`,
  `drl-audio`, and web-sys/wasm-bindgen.

### `drl-mcp` — Model Context Protocol Server
- **Role**: Zero-dependency JSON-RPC 2.0 MCP server for AI agents and test
  automation.
- **Key Responsibilities**:
  - Full MCP method suite (`initialize`, `tools/*`, `resources/*`).
  - Semantic tools for game control, observation, action enumeration, and
    replays.
  - Strict observation boundaries with explicit `dev_mode` flag for omniscient
    inspection.
- **Dependencies**: Pure `std` + `drl-protocol` + `drl-core`.

### `drl-app` — Headless CLI & MCP Runner
- **Role**: Native executable for running headless demos, batch sweeps, and stdio
  MCP sessions.
- **Dependencies**: Depends on `drl-core`, `drl-protocol`, `drl-mcp`.

### `drl-script` — Content Conversion Placeholder
- **Role**: Build-time conversion boundary for legacy content; placeholder for
  future offline migration tools.

---

## 4. Subsystem Architecture & Rules

### 4.1 Simulation & Turn Economy
- **Energy-Based Scheduler**: Actors accumulate energy based on their `Speed`.
  When an actor reaches the action threshold, it executes one action costing
  standard energy units.
- **Deterministic PRNG**: All randomness flows through `GameRng`. No ambient or
  thread-local RNG is permitted.
- **Combat Resolution**: `CombatResolver` evaluates melee bump attacks and
  targeted ranged attacks with explicit distance accuracy scaling, uniform
  damage rolls, armor protection mitigation, and health clamping.
- **Kinetic Knockback**: Shotgun blasts push surviving targets along firing
  vectors with collision checks against map borders, solid walls, and other
  actors.

### 4.2 Content Tables & Definitions
- **Monster Definitions**: `MonsterKind::definition()` in `drl-protocol` owns
  the authoritative immutable stats, speeds, attack ranges, accuracies, and
  drop tables for all current archetypes.
- **Item Definitions**: `drl-core::item_definition` owns item definitions;
  `Item::from_spawn_kind` serves as canonical item factory.
- **Loot & Monster Rolls**: Pure roll-bound tables map caller-supplied PRNG
  rolls to procedural room loot and monster spawns.
- **Tile Definitions**: `TileKind::definition()` in `drl-protocol` defines
  physical walkability, transparency, and liquid properties for all tile kinds.
- **Level Policy**: `drl-core::level_definition` provides standard procedural
  generation parameters.

### 4.3 Rendering Pipeline & Viewport
- **Square-Cell Integer Viewport**: `PixelViewport` computes integer square
  cell dimensions from canvas dimensions and applies deterministic centered
  letterboxing. Non-uniform axis stretching is prohibited.
- **Visibility-Derived Lighting**: `LightingBand` assigns full light (`1.0`) to
  active FOV tiles and a fixed fog factor (`0.3`) to explored memory tiles.
  Hidden tiles are omitted.
- **Scene Tone & Low-Health Pulse**: `SceneTone` applies a low-health clear tone
  below 25% HP; `low_health_pulse_target_alpha` and `LowHealthPulseState`
  provide smooth, bounded alpha modulation.
- **Layer & Composite Plans**: `layer_draw_plan` generates ordered scene draws
  (tiles, items, actors); `sprite_composite_plan` groups all registered layer
  roles per sprite for WebGPU shader bind groups.

### 4.4 Animation & Visual Timing
- **Decoupled Effect Timing**: `EffectSpan` assigns fixed logical durations to
  events. Presentation timing never drives or advances gameplay turns.
- **Elapsed-Time Frame Selection**: `animation_frame_index_at_elapsed` selects
  animation frames based on elapsed milliseconds and explicit loop/clamp
  policies.
- **Pure Effect Arithmetic**: Explosion marks, cell animations, kill segments,
  FX frames, movement interpolation, missile steps/rays, and screen shake fade
  are pure mathematical functions without GPU or simulation dependencies.
- **Visibility Lifecycle Clock**: Browser animation loop listens to
  `visibilitychange` and rebases presentation clocks when background tabs resume.

### 4.5 Particles & Decals
- **Pure Arithmetic Contracts**: Burst origins, direction normalization, arc
  adjustments, range interpolation, decal cell mapping, and placement are pure
  functions.
- **Eligibility Filter**: `particle_decal_cell_is_eligible` enforces map bounds,
  non-liquid, and non-blocking rules.
- **Insertion Request**: `ParticleDecalInsertion` packages placement with the
  caller-provided sprite ID.
- **Bounded Decal Store**: `ParticleDecalStore` retains requests in strict
  insertion order with caller-configured capacity, reporting overflow without
  dropping prior entries.
- **Decal Draw Planning**: `particle_decal_draw_plan` resolves opaque,
  caller-provided sprite handles to complete atlas layer groups, uses stored
  pixel placement for sub-cell offsets, carries caller-resolved lighting,
  omits unknown/out-of-viewport entries, and leaves the store unchanged.
  Combined scene plans place decals between terrain and ordinary objects;
  WebGPU owns batching and resource binding.

### 4.6 Browser & WebGPU Integration
- **Texture Cache**: Imported atlas PNGs are loaded same-origin, dimension-
  checked, and uploaded once into linear `Rgba8Unorm` WebGPU 2D textures.
- **Nearest Base Filtering**: Base sprite pixels are sampled with nearest
  filtering.
- **Emissive Floor**: Emissive mask red channel acts as a minimum lighting
  floor.
- **Alpha Cutoff**: WGSL textured shader discards fragments below the legacy
  `0.1` alpha threshold.
- **Colorization Tints**: Pinned vertex tints apply to Green Armor, Phase
  Device, and StairsDown.
- **Outline Straight-Alpha**: Optional outline-mask shadow layers composite
  behind base pixels with tested straight-alpha weights.

### 4.7 Replays, Cohorts & Evaluation
- **Replay V1 Engine**: `ReplayEngine` records and executes versioned replay
  logs with exact initial spawn metadata and command streams.
- **Cohort Reports**: `CohortConfig` / `CohortReport` execute multi-seed sweeps,
  validating seed order, record counts, and summary metrics.
- **Descriptive Telemetry**: Cohort outcome distributions and telemetry
  comparisons report exact win/loss rates, accuracy, damage, and kills with
  caller-owned tolerances.

### 4.8 Persistence & Release Packaging
- **Session Snapshots**: `SessionSnapshot` encodes complete command histories
  with version headers, checksums, and strict corruption handling.
- **Service Worker Cache**: Versioned same-origin worker caches static bundles
  keyed by project version and commit hash.
- **Release Manifests**: Build tooling generates `release-manifest.json` with
  sorted artifact SHA-256 hashes and a `.sha256` sidecar digest.

---

## 5. Architectural Invariants

Every change to the codebase must preserve these non-negotiable invariants:

1. **No Ambient State in Core**: No global variables, thread-local state,
   ambient RNG, wall clocks, or non-deterministic hash iteration in `drl-core`.
2. **Deterministic Replay**: Given identical seed and command stream, the
   simulation must produce bit-exact identical observations, events, and final
   world state across all platforms.
3. **Observation Decoupling**: Renderers, bots, and MCP agents consume only
   fair `PlayerObservation` views; they must never access `World` or inspect
   unexplored tiles.
4. **Presentation Decoupling**: Audio cues, rendering frames, WebGPU shaders,
   canvas resizing, and tab visibility transitions must never mutate simulation
   state or advance game turns.
5. **No Runtime Scripting**: No Lua VM or JavaScript gameplay interpreters.
6. **Explicit Error Handling**: Illegal commands and corrupt saves fail
   transactionally without partial mutations.

---

## 6. Verification & Automated Boundary Enforcement

The repository enforces architectural boundaries via automated test suites:

- **Boundary Enforcement**: `crates/drl-core/tests/boundaries.rs` validates
  dependency direction and ensures core remains free of presentation or platform
  crates.
- **Repository Health**: `sh scripts/check-repository.sh` runs formatting,
  clippy, unit tests, integration tests, and harness checks.
- **Asset Manifest**: `sh scripts/check-assets.sh` verifies graphics licensing
  and SHA-256 checksums.
- **Web Contracts**: `sh scripts/check-web.sh` checks WASM compilation and web
  contracts.
- **Release Validation**: `scripts/check-release-manifest.sh` validates release
  artifacts and service worker coverage.
- **Version Projections**: `scripts/check-version.sh` enforces valid `x.y.z`
  transitions.
