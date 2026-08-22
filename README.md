# DRL-Rust

DRL-Rust is a ground-up Rust reimplementation of [Doom the Roguelike (DRL)](https://drl.chaosforge.org/),
originally created by Kornel Kisielewicz and [ChaosForge](https://chaosforge.org/).
The product direction is browser-first: a deterministic Rust/WASM game rendered
with WebGPU in desktop Chrome/Edge, with an accessible HTML shell and
gesture-unlocked Web Audio. Headless Rust and MCP remain supported for agents,
replays, and regression testing.

## Current capabilities

- Deterministic simulation:
  - Complete M4 headless game loop with combat, FOV/fog, AI, levels, replay,
    scenarios, bots, batches, inventory, and MCP tooling.
  - Stable tile, item, monster, and standard-level definitions, canonical item
    factories, and table-driven generated item/monster selection with preserved
    RNG boundaries.
  - Fixed-seed cohort reports preserve sample definitions, policy identity,
    aggregate metrics, and per-seed replay evidence for evaluation.
  - Cohort regression math applies explicit win-rate and average-turn
    tolerances without mutating simulation or claiming balance parity.
  - Cohort report validation rejects inconsistent sample/evidence metadata
    before a regression comparison is used.
  - Cohort outcome distributions preserve distinct terminal counts and
    sample-normalized rates without interpreting balance or significance.
  - Compatible cohort comparisons report absolute per-outcome rate deltas
    after integrity validation without adding tolerance or significance claims.
  - Outcome comparisons accept one finite, non-negative per-category rate
    tolerance and expose a deterministic pass/fail gate.
  - Cohort telemetry projections and compatible comparisons expose validated
    shot accuracy, damage, kill, pickup, and item-use totals/rates without
    inferring balance conclusions.
- Versioned delivery:
  - `VERSION` is the canonical `x.y.z` project value (currently `0.2.12`),
    projected into Cargo, MCP, and release manifests; the agent harness rejects
    invalid code-change transitions and ignores document/setting-only diffs.
- Browser and presentation slice:
  - M7 functional checks pass locally and in remote web CI.
  - M8 provides square pixel-grid letterboxing, shared lighting bands, measured
    atlas slots, normalized UVs, renderer-neutral layer/composite plans, fair
    observation-derived presentation, and validated texture-source loading.
  - Native-tested base/mask/emissive rendering includes evidence-backed Green
    Armor, Phase Device, and StairsDown tint boundaries, an emissive lighting
    floor, optional outline-mask compositing, and animation frame
    metadata/selection.
  - Pure contracts cover effect timing, low-health tone/pulse, explosion marks,
    movement and missile progress, screen-shake fade, particle origins,
    burst directions/range sampling, decal cell/placement/eligibility,
    caller-owned insertion requests, deterministic bounded decal storage,
    stored-pixel decal draw planning with opaque sprite-handle resolution, and
    BrowserSession-to-WebGPU decal consumption without claiming full backend
    fidelity.
- Staged work:
  - Full audiovisual equivalence, broader content migration, offline PWA
    acceptance, and support for other browsers remain roadmap work.
  - Release builds emit a hashed static-bundle manifest with graphics rights
    metadata; signing and offline/cross-browser acceptance remain open.
  - Placeholder M7 atlas rectangles are not a fidelity claim.

## Quick start

### Headless and MCP

```sh
cargo run -p drl-app
cargo run -p drl-app -- --mcp
```

### Browser slice

Prerequisites: Rust, the `wasm32-unknown-unknown` target, and `wasm-pack
0.15.0`.

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-pack --version 0.15.0 --locked
scripts/build-web.sh
scripts/serve-web.sh
```

Open `http://localhost:8080` over the local static server, press Start to
unlock audio/WebGPU, and focus the canvas. Arrows/WASD and numpad move;
Space/`.` waits; G picks up; R reloads; F selects the nearest visible enemy,
Enter fires, and Escape cancels; `>` descends. Numpad 7/9/1/3 are the
documented diagonal bindings. Inventory controls are exposed through the
semantic DOM region.

If WebGPU is unavailable, the page shows an explicit unsupported-device
status. Audio may remain suspended until a trusted user gesture; that state is
recoverable and never advances the game.

## Verification

```sh
sh scripts/check-repository.sh
sh scripts/check-assets.sh
scripts/check-version.sh
scripts/check-web.sh
scripts/check-browser-diagnostics.sh # also run by check-web.sh
scripts/check-browser-accessibility.sh # also run by check-web.sh
scripts/check-reference-capture.sh
scripts/check-release-manifest.sh  # after scripts/build-web.sh
```

`check-web.sh` compiles the WASM target and runs native contract tests. It runs
headless Chrome WASM tests when Chrome is installed; otherwise it reports the
browser run as `NOT_RUN`. Remote Ubuntu CI owns the required web-CI evidence.

## Architecture

```text
DOM / keyboard -> drl-protocol::Command -> drl-core::Game
               -> PlayerObservation + GameEvent
               -> drl-render / drl-audio -> WebGPU canvas / Web Audio
```

The core has no rendering, audio, browser, filesystem, network, or MCP
dependency. Frontends consume fair player observations, never `World`.
`drl-assets` contains platform-neutral semantic atlas descriptors and licensed
legacy graphics metadata; it is not a dependency of the core.

## Workspace layout

- `crates/drl-core`: deterministic simulation, combat, FOV, AI, levels, items,
  scenarios, bots, batches, and replays.
- `crates/drl-protocol`: commands, observations, events, identifiers, and
  compatibility-sensitive MCP/replay contracts.
- `crates/drl-assets`:
  - Atlas IDs/dimensions, measured rectangles, registered layers and shader
    roles, normalized UVs, texture-source bindings, and semantic asset mapping.
  - Pinned legacy revision identity and licensing metadata.
- `crates/drl-render`:
  - Pure scene construction, pixel viewport layout, layer/composite plans,
    lighting, animation selection, and observed tint mappings.
  - Source-derived contracts for health tone/pulse, effect and missile timing,
    screen shake, particle origins, decal placement/eligibility/insertion,
    bounded decal storage, and post-process glow/LUT math.
  - Renderer/backend and full audiovisual equivalence remain staged work.
- `crates/drl-audio`: semantic cues and WASM Web Audio mixer.
- `crates/drl-web`:
  - Browser session, fair observation boundary, validated texture loading,
    renderer-owned WebGPU uploads, and the partial textured pass.
  - Animation playback, bounded scheduling, fixed-session snapshots,
    best-effort localStorage, bounded rejected-save quarantine,
    generated-bundle service-worker cache, and the project-version/source-
    revision cache policy and manifest digest sidecar recorded by release
    manifests, with a mocked service-worker lifecycle contract and source-
    identity audit.
  - Local accessible browser-support/startup diagnostics with recovery guidance;
    no telemetry or untested-browser support claim.
  - Static shell accessibility audit for names, labels, focus, and live regions;
    dynamic WCAG/screen-reader acceptance remains open.
- `crates/drl-mcp`: JSON-RPC/MCP server and fairness boundary.
- `crates/drl-app`: native headless demo and MCP stdio runner.
- `docs/DRL-Rust_Project_Roadmap.md`: canonical milestones and gates.
- `SPEC.md`: the one active implementation slice.
- `ARCHITECTURE.md`, `CHANGELOG.md`, `docs/adr/`, `docs/legacy-behavior/`, and
  `docs/reference-captures/`, `docs/harness/`: verified structure, history,
  decisions, evidence, and agent workflow.

## Acknowledgements and credits

- **Original Game**: [DRL](https://drl.chaosforge.org/) (formerly *Doom, the Roguelike*)
  was created by Kornel Kisielewicz and developed by [ChaosForge](https://chaosforge.org/).
  The upstream open-source FreePascal codebase is hosted on GitHub at
  [ChaosForge/drl](https://github.com/chaosforgeorg/drl).
- **Art & Sprites**: Original sprite art and tiles (0.9.9.7) were created by
  Derek Yu (CC BY-SA 4.0). Additions and modifications (0.9.9.8+) were created
  by Łukasz Śliwiński (CC BY-SA 4.0).
- **Spiritual Successors**: ChaosForge has since created the modern 3D
  spiritual successor [Jupiter Hell](https://store.steampowered.com/app/811320/Jupiter_Hell/)
  and [Jupiter Hell Classic](https://store.steampowered.com/app/3126530/Jupiter_Hell_Classic/).

## Legacy assets and licensing

The imported graphics under `assets/legacy/drl/graphics/` come from the pinned
legacy Git revision recorded in `MANIFEST.txt` and `SHA256SUMS`, with the
upstream CC BY-SA 4.0 license and attribution. The repository's MIT license
does not relicense them. Legacy code is GPL; audio, music, and fonts are not
bundled until their separate redistribution rights are recorded. See
`docs/legacy-behavior/asset-provenance.md` and
`docs/reference-captures/manifest.md`, which records checkout dirty-state and
evidence classification, rights, and media hashes while keeping capture
promotion gated on a clean controlled checkout with directly observed evidence.

### Downloading original assets

This repository tracks only the 32 CC BY-SA 4.0 graphics sprite sheets in
`assets/legacy/drl/graphics/`. Untracked assets such as sound effects, music,
fonts, and WAD packages can be downloaded from the original sources:

- **Official binary downloads (audio, music, WADs)**:
  1. Visit the ChaosForge downloads page at
     [https://drl.chaosforge.org/downloads](https://drl.chaosforge.org/downloads).
  2. Download the official game release archive for your platform (Windows, Linux,
     or macOS) along with the optional MP3 music pack and HQ sound pack.
  3. Extract the downloaded archives to locate:
     - Sound effects: `sound/` and `soundhq/` (or `data/drlhq/sounds/` and
       `data/drllq/sounds/`).
     - Music: `music/` (MIDI) and `mp3/` (HQ audio), or `data/drlhq/music/`.
     - Data packages: `drl.wad` and `core.wad`.
- **GitHub repository (source, data, and definitions)**:
  1. Clone the upstream repository:
     ```sh
     git clone https://github.com/chaosforgeorg/drl.git
     ```
  2. Lua gameplay scripts, definitions, and raw data are located under
     `bin/data/` (`drl/`, `drlhq/`, and `drllq/`).
  3. The Valkyrie engine source is available at
     [https://github.com/ChaosForge/fpcvalkyrie](https://github.com/ChaosForge/fpcvalkyrie).

## Contributing

Read `AGENTS.md`, `CONTRIBUTING.md`, the active `SPEC.md`, and the roadmap
before changing a milestone. Preserve deterministic headless behavior and run
`sh scripts/check-repository.sh`. Browser changes also need WASM/build
evidence, browser metadata, and an explicit statement of any unavailable
WebGPU/audio/reference-capture checks.
