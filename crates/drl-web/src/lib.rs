//! Browser-first DRL-Rust session boundary.
//!
//! `BrowserSession` is intentionally usable on native hosts for deterministic
//! tests. The WASM exports are a thin boot/input shell; gameplay state stays in
//! Rust and is never mirrored into a parallel JavaScript model.

use drl_assets::{AtlasId, AtlasTextureSource, SpriteUv};
use drl_core::item::Item;
use drl_core::{Game, Tile};
use drl_protocol::{
  Command, Direction, ItemId, ItemSpawnKind, ItemSpawnSpec, MonsterKind, MonsterSpawnSpec,
  PlayerObservation, Position, ReplayLog,
};
use drl_render::{
  LightingBand, ParticleDecalSprite, ParticleDecalStorageError, ParticleDecalStore, PixelRect,
  PresentationStep, RenderScene, effect_timeline_for_observations,
};

mod persistence;
pub use persistence::SnapshotError;

/// Returns the six UV coordinates for a top-left-origin textured quad.
#[must_use]
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
const fn base_texture_uvs(uv: SpriteUv) -> [[f32; 2]; 6] {
  [
    [uv.u_min, uv.v_max],
    [uv.u_max, uv.v_max],
    [uv.u_max, uv.v_min],
    [uv.u_min, uv.v_max],
    [uv.u_max, uv.v_min],
    [uv.u_min, uv.v_min],
  ]
}

/// Returns the shared fair lighting factor used by the textured pass.
#[must_use]
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
fn base_texture_lighting_factor(band: LightingBand) -> f32 {
  band.factor() as f32 / 100.0
}

/// Applies the legacy emissive floor to a fair RGB lighting scalar.
#[allow(dead_code)]
fn emissive_lighting_floor(lighting: f32, emissive: f32) -> f32 {
  lighting.max(emissive)
}

/// Matches the legacy shader's minimum surviving fragment alpha.
#[allow(dead_code)]
fn retains_textured_fragment(alpha: f32) -> bool {
  alpha >= 0.1
}

/// Shared WGSL source for the bounded base/mask/emissive/outline textured pass.
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
const BASE_TEXTURE_SHADER: &str = r#"
struct VertexInput {
  @location(0) position: vec2<f32>,
  @location(1) uv: vec2<f32>,
  @location(2) lighting: vec4<f32>,
  @location(3) colorization: vec4<f32>,
};

struct VertexOutput {
  @builtin(position) position: vec4<f32>,
  @location(0) uv: vec2<f32>,
  @location(1) lighting: vec4<f32>,
  @location(2) colorization: vec4<f32>,
};

@group(0) @binding(0) var base_texture: texture_2d<f32>;
@group(0) @binding(1) var emissive_texture: texture_2d<f32>;
@group(0) @binding(2) var mask_texture: texture_2d<f32>;
@group(0) @binding(3) var outline_texture: texture_2d<f32>;
@group(0) @binding(4) var base_sampler: sampler;

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
  var output: VertexOutput;
  output.position = vec4<f32>(input.position, 0.0, 1.0);
  output.uv = input.uv;
  output.lighting = input.lighting;
  output.colorization = input.colorization;
  return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
  let sampled = textureSample(base_texture, base_sampler, input.uv);
  let emissive = textureSample(emissive_texture, base_sampler, input.uv).r;
  let mask = textureSample(mask_texture, base_sampler, input.uv);
  let outline = textureSample(outline_texture, base_sampler, input.uv);
  let colorized = sampled.rgb + mask.rgb * input.colorization.rgb;
  let lighting = max(input.lighting.rgb, vec3<f32>(emissive));
  let outline_alpha = outline.a * (1.0 - sampled.a);
  let output_alpha = sampled.a + outline_alpha;
  let output_rgb = (colorized * sampled.a + outline.rgb * outline_alpha)
    / max(output_alpha, 0.0001);
  let output = vec4<f32>(output_rgb * lighting, output_alpha);
  if (output.a < 0.1) {
    discard;
  }
  return output;
}
"#;

/// Converts a physical destination rectangle into clip-space bounds.
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
const fn base_texture_ndc_rect(rect: PixelRect, canvas_width: u32, canvas_height: u32) -> [f32; 4] {
  let width = if canvas_width == 0 { 1 } else { canvas_width } as f32;
  let height = if canvas_height == 0 { 1 } else { canvas_height } as f32;
  [
    -1.0 + 2.0 * rect.x as f32 / width,
    1.0 - 2.0 * rect.y.saturating_add(rect.height) as f32 / height,
    -1.0 + 2.0 * rect.x.saturating_add(rect.width) as f32 / width,
    1.0 - 2.0 * rect.y as f32 / height,
  ]
}

/// Converts a browser animation timestamp into bounded elapsed milliseconds.
///
/// The timestamp source and scheduling policy remain outside this pure helper.
#[must_use]
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
fn animation_elapsed_ms(start_ms: f64, timestamp_ms: f64) -> Option<u64> {
  if !start_ms.is_finite() || !timestamp_ms.is_finite() || timestamp_ms < start_ms {
    return None;
  }
  let elapsed_ms = (timestamp_ms - start_ms).floor();
  if elapsed_ms >= u64::MAX as f64 {
    Some(u64::MAX)
  } else {
    Some(elapsed_ms.max(0.0) as u64)
  }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
struct AnimationClock {
  start_ms: Option<f64>,
}

#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
impl AnimationClock {
  fn reset(&mut self) {
    self.start_ms = None;
  }

  fn visibility_changed(&mut self) {
    self.reset();
  }

  fn elapsed_ms(&mut self, hidden: bool, timestamp_ms: f64) -> Option<u64> {
    if hidden {
      self.reset();
      return None;
    }
    let start_ms = *self.start_ms.get_or_insert(timestamp_ms);
    animation_elapsed_ms(start_ms, timestamp_ms)
  }
}

/// Fixed deterministic content slice used by the first browser playthrough.
pub const M4_SEED: u64 = 0x4452_4c5f_4d34;
pub const M4_WIDTH: u32 = 24;
pub const M4_HEIGHT: u32 = 16;
pub const M4_START: Position = Position::new(4, 8);

/// Static bundle root used by the browser texture loader.
pub const GRAPHICS_ASSET_ROOT: &str = "assets/legacy/drl/graphics/";

const REGISTERED_ATLASES: [AtlasId; 7] = [
  AtlasId::Dguy,
  AtlasId::Enemies,
  AtlasId::EnemiesBig,
  AtlasId::GunsAndPickups,
  AtlasId::Levels,
  AtlasId::DoorsAndDecorations,
  AtlasId::Fx,
];

/// Returns every unique imported layer source in stable atlas registration
/// order. A browser uploader can use this manifest without inspecting scenes.
#[must_use]
pub fn texture_source_manifest() -> Vec<AtlasTextureSource> {
  let mut sources = Vec::new();
  for atlas in REGISTERED_ATLASES {
    for &layer in atlas.layers() {
      let source = atlas.texture_source(layer);
      if !sources.contains(&source) {
        sources.push(source);
      }
    }
  }
  sources
}

/// A rejected browser asset path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextureSourcePathError {
  pub path: String,
}

impl std::fmt::Display for TextureSourcePathError {
  fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(formatter, "invalid texture source path: {}", self.path)
  }
}

/// Validates a relative imported-asset basename for subpath-safe loading.
pub fn browser_asset_url(path: &str) -> Result<String, TextureSourcePathError> {
  let valid = !path.is_empty()
    && !path.starts_with('/')
    && !path.contains("..")
    && !path.contains(['\\', '/', '?', '#'])
    && !path.contains("://")
    && path
      .bytes()
      .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'));
  if !valid {
    return Err(TextureSourcePathError {
      path: path.to_string(),
    });
  }
  Ok(format!("{GRAPHICS_ASSET_ROOT}{path}"))
}

/// Returns the same-origin URL for an imported atlas layer.
pub fn texture_source_url(source: AtlasTextureSource) -> Result<String, TextureSourcePathError> {
  browser_asset_url(source.path)
}

/// A decoded-source dimension mismatch at the browser asset boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextureSourceDimensionsError {
  pub path: &'static str,
  pub expected: (u32, u32),
  pub actual: (u32, u32),
}

impl std::fmt::Display for TextureSourceDimensionsError {
  fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      formatter,
      "texture {} has dimensions {}x{}, expected {}x{}",
      self.path, self.actual.0, self.actual.1, self.expected.0, self.expected.1
    )
  }
}

/// Validates decoded image dimensions against the pinned asset metadata.
pub fn validate_texture_source_dimensions(
  source: AtlasTextureSource,
  actual_width: u32,
  actual_height: u32,
) -> Result<(), TextureSourceDimensionsError> {
  let expected = (source.width, source.height);
  if expected == (actual_width, actual_height) {
    Ok(())
  } else {
    Err(TextureSourceDimensionsError {
      path: source.path,
      expected,
      actual: (actual_width, actual_height),
    })
  }
}

/// A browser-facing simulation session with transactional command handling.
#[derive(Debug, Clone)]
pub struct BrowserSession {
  game: Game,
  last_error: Option<String>,
  commands: Vec<Command>,
  particle_decals: ParticleDecalStore,
  particle_decal_sprites: Vec<ParticleDecalSprite>,
}

impl BrowserSession {
  /// Creates the fixed M4 arena and its representative loot/combat content.
  pub fn new() -> Result<Self, drl_protocol::CommandError> {
    Ok(Self {
      game: Self::fixed_game()?,
      last_error: None,
      commands: Vec::new(),
      particle_decals: ParticleDecalStore::new(256),
      particle_decal_sprites: Vec::new(),
    })
  }

  /// Builds the same fixed content for direct-core parity tests and tools.
  pub fn fixed_game() -> Result<Game, drl_protocol::CommandError> {
    let mut game = Game::new(M4_SEED, M4_WIDTH, M4_HEIGHT, M4_START)?;
    let stairs = Position::new(19, 8);
    game
      .world_mut()
      .map_mut()
      .set_tile(stairs, Tile::StairsDown);

    let loot_position = Position::new(7, 8);
    for kind in [
      drl_protocol::ItemSpawnKind::Shotgun,
      drl_protocol::ItemSpawnKind::GreenArmor,
      drl_protocol::ItemSpawnKind::SmallMedPack,
    ] {
      let id = game.world_mut().allocate_item_id();
      game
        .world_mut()
        .spawn_ground_item(loot_position, Item::from_spawn_kind(id, kind))?;
    }

    let monster_position = Position::new(13, 8);
    let id = game.world_mut().allocate_entity_id();
    let monster = drl_core::Actor::from_monster_kind(id, monster_position, MonsterKind::Imp);
    game.world_mut().actors_mut().insert(id, monster);
    Ok(game)
  }

  /// Returns the current fair player observation.
  #[must_use]
  pub fn observation(&self) -> PlayerObservation {
    self.game.observe_player()
  }

  /// Returns the current render scene derived from the fair observation.
  #[must_use]
  pub fn scene(&self) -> RenderScene {
    RenderScene::from_observation(&self.observation())
  }

  /// Returns retained presentation-only decal requests for the browser pass.
  #[must_use]
  pub fn particle_decal_store(&self) -> &ParticleDecalStore {
    &self.particle_decals
  }

  /// Returns the caller-owned opaque sprite-handle descriptor table.
  #[must_use]
  pub fn particle_decal_sprites(&self) -> &[ParticleDecalSprite] {
    &self.particle_decal_sprites
  }

  /// Retains one presentation-only decal request without touching gameplay.
  pub fn try_insert_particle_decal(
    &mut self,
    insertion: drl_render::ParticleDecalInsertion,
  ) -> Result<(), ParticleDecalStorageError> {
    self.particle_decals.try_insert(insertion)
  }

  /// Replaces the caller-owned descriptor table used by decal rendering.
  pub fn set_particle_decal_sprites(&mut self, sprites: Vec<ParticleDecalSprite>) {
    self.particle_decal_sprites = sprites;
  }

  /// Returns the most recent rejected-command message, if any.
  #[must_use]
  pub fn last_error(&self) -> Option<&str> {
    self.last_error.as_deref()
  }

  /// Returns true after the deterministic session reaches player death.
  #[must_use]
  pub fn is_game_over(&self) -> bool {
    self.game.is_game_over()
  }

  /// Submits one semantic command. Failed commands roll back the session.
  pub fn submit(&mut self, command: Command) -> Result<PresentationStep, String> {
    let before = self.observation();
    let checkpoint = self.game.clone();
    match self.game.step(command) {
      Ok(events) => {
        self.last_error = None;
        self.commands.push(command);
        let after = self.observation();
        let effects = effect_timeline_for_observations(&before, &after, &events);
        Ok(PresentationStep {
          before,
          command,
          events,
          effects,
          after,
        })
      }
      Err(error) => {
        self.game = checkpoint;
        let message = error.to_string();
        self.last_error = Some(message.clone());
        Err(message)
      }
    }
  }

  /// Restarts the deterministic M4 session.
  pub fn restart(&mut self) -> Result<(), drl_protocol::CommandError> {
    *self = Self::new()?;
    Ok(())
  }

  /// Encodes successful fixed-session commands into a versioned save token.
  pub fn snapshot_token(&self) -> Result<String, SnapshotError> {
    persistence::encode_snapshot(&self.commands)
  }

  /// Rebuilds this session from a versioned token without exposing game state.
  pub fn restore_snapshot(&mut self, token: &str) -> Result<(), SnapshotError> {
    let commands = persistence::decode_snapshot(token)?;
    let mut restored =
      Self::new().map_err(|error| SnapshotError::Initialization(error.to_string()))?;
    for command in commands {
      restored
        .submit(command)
        .map_err(SnapshotError::CommandRejected)?;
    }
    *self = restored;
    Ok(())
  }

  /// Returns a replay-schema representation of the fixed browser session.
  ///
  /// The log uses the existing V1 schema; it does not create a browser-specific
  /// wire format or expose authoritative state to JavaScript.
  #[must_use]
  pub fn replay_log(&self) -> ReplayLog {
    let mut replay = ReplayLog::new(M4_SEED, M4_WIDTH, M4_HEIGHT, M4_START);
    replay.record_stairs(Position::new(19, 8));
    replay.record_monster(
      MonsterSpawnSpec::new(
        Position::new(13, 8),
        "Imp",
        MonsterKind::Imp.default_hp(),
        MonsterKind::Imp.default_speed(),
        MonsterKind::Imp.default_melee_damage(),
      )
      .with_ranged_combat((5, 10), 8, 70)
      .with_death_drop(Some(ItemSpawnKind::SmallMedPack)),
    );
    let loot_position = Position::new(7, 8);
    replay.record_item(ItemSpawnSpec::new(loot_position, ItemSpawnKind::Shotgun));
    replay.record_item(ItemSpawnSpec::new(loot_position, ItemSpawnKind::GreenArmor));
    replay.record_item(ItemSpawnSpec::new(
      loot_position,
      ItemSpawnKind::SmallMedPack,
    ));
    for command in &self.commands {
      replay.record_command(*command);
    }
    replay
  }

  /// Maps keyboard names to semantic commands without advancing the game.
  #[must_use]
  pub fn command_for_key(key: &str, observation: &PlayerObservation) -> Option<Command> {
    let direction = match key {
      "ArrowUp" | "w" | "W" | "8" => Some(Direction::North),
      "ArrowRight" | "d" | "D" | "6" => Some(Direction::East),
      "ArrowDown" | "s" | "S" | "2" => Some(Direction::South),
      "ArrowLeft" | "a" | "A" | "4" => Some(Direction::West),
      "7" => Some(Direction::NorthWest),
      "9" => Some(Direction::NorthEast),
      "1" => Some(Direction::SouthWest),
      "3" => Some(Direction::SouthEast),
      _ => None,
    };
    if let Some(direction) = direction {
      return Some(Command::Move(direction));
    }
    match key {
      "." | "5" | "Space" => Some(Command::Wait),
      "g" | "G" => Some(Command::Pickup),
      "r" | "R" => Some(Command::Reload),
      ">" => Some(Command::Descend),
      "f" | "F" => observation
        .visible_actors
        .iter()
        .find(|actor| !actor.is_player)
        .map(|actor| Command::AttackRanged(actor.position)),
      _ => None,
    }
  }

  /// Creates an explicit ranged target command for a DOM/canvas click.
  #[must_use]
  pub const fn target_command(position: Position, confirmed: bool) -> Option<Command> {
    if confirmed {
      Some(Command::AttackRanged(position))
    } else {
      None
    }
  }

  /// Maps an inventory action from a semantic item id.
  #[must_use]
  pub const fn inventory_command(action: InventoryAction, item_id: ItemId) -> Command {
    match action {
      InventoryAction::Equip => Command::Equip(item_id),
      InventoryAction::Use => Command::Use(item_id),
      InventoryAction::Drop => Command::Drop(item_id),
    }
  }
}

/// DOM inventory action supported by the first slice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InventoryAction {
  Equip,
  Use,
  Drop,
}

/// Browser GPU backend state exposed to the DOM error screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuStatus {
  Ready,
  Unsupported,
  Lost,
}

#[cfg(target_arch = "wasm32")]
mod texture;

#[cfg(target_arch = "wasm32")]
pub(crate) mod wasm {
  use super::texture::{BaseTexturePipeline, GpuTextureCache};
  use super::*;
  use drl_render::{AnimationPlayback, PixelViewport, scene_clear_color, shade_color};
  use std::cell::RefCell;
  use wasm_bindgen::prelude::*;
  use wasm_bindgen_futures::JsFuture;
  use web_sys::{HtmlCanvasElement, HtmlImageElement, Storage, Window};
  use wgpu::util::DeviceExt;
  use winit::application::ApplicationHandler;
  use winit::event::{ElementState, WindowEvent};
  use winit::event_loop::{ActiveEventLoop, EventLoop};
  use winit::keyboard::{KeyCode, PhysicalKey};
  use winit::platform::web::{EventLoopExtWebSys, WindowAttributesExtWebSys};
  use winit::window::{Window as WinitWindow, WindowId};

  thread_local! {
    static SESSION: RefCell<Option<BrowserSession>> = const { RefCell::new(None) };
    static RENDERER: RefCell<Option<WebGpuRenderer>> = const { RefCell::new(None) };
    static AUDIO: RefCell<Option<drl_audio::WebAudioMixer>> = const { RefCell::new(None) };
    static TARGET: RefCell<Option<Position>> = const { RefCell::new(None) };
    static ANIMATION_CLOCK: RefCell<AnimationClock> = const { RefCell::new(AnimationClock { start_ms: None }) };
    static ANIMATION_LOOP: RefCell<Option<Closure<dyn FnMut(f64)>>> = const { RefCell::new(None) };
    static VISIBILITY_LISTENER: RefCell<Option<Closure<dyn FnMut()>>> = const { RefCell::new(None) };
  }

  const SAVE_STORAGE_KEY: &str = "drl-rust:m4-session:v1";
  const REJECTED_SAVE_STORAGE_KEY: &str = "drl-rust:m4-session:v1:rejected";

  fn browser_storage() -> Result<Storage, SnapshotError> {
    let window = web_sys::window()
      .ok_or_else(|| SnapshotError::Initialization("window unavailable".to_string()))?;
    window
      .local_storage()
      .map_err(|error| {
        SnapshotError::Initialization(format!("localStorage unavailable: {error:?}"))
      })?
      .ok_or_else(|| SnapshotError::Initialization("localStorage unavailable".to_string()))
  }

  fn persist_session(session: &BrowserSession) -> Result<(), SnapshotError> {
    let token = session.snapshot_token()?;
    browser_storage()?
      .set_item(SAVE_STORAGE_KEY, &token)
      .map_err(|error| SnapshotError::Initialization(format!("save failed: {error:?}")))
  }

  fn remove_persisted_session() -> Result<(), SnapshotError> {
    browser_storage()?
      .remove_item(SAVE_STORAGE_KEY)
      .map_err(|error| SnapshotError::Initialization(format!("clear failed: {error:?}")))
  }

  fn remove_rejected_session() -> Result<(), SnapshotError> {
    browser_storage()?
      .remove_item(REJECTED_SAVE_STORAGE_KEY)
      .map_err(|error| SnapshotError::Initialization(format!("quarantine clear failed: {error:?}")))
  }

  fn quarantine_persisted_session(token: &str, error: &SnapshotError) -> Result<(), SnapshotError> {
    let storage = browser_storage()?;
    let record = persistence::encode_quarantine_record(token, error);
    storage
      .set_item(REJECTED_SAVE_STORAGE_KEY, &record)
      .map_err(|storage_error| {
        SnapshotError::Initialization(format!("quarantine write failed: {storage_error:?}"))
      })?;
    storage
      .remove_item(SAVE_STORAGE_KEY)
      .map_err(|storage_error| {
        SnapshotError::Initialization(format!("active save clear failed: {storage_error:?}"))
      })
  }

  fn rejected_save_message(token: &str, error: &SnapshotError) -> String {
    match quarantine_persisted_session(token, error) {
      Ok(()) => format!(" Saved session ignored ({error}); rejected save quarantined."),
      Err(recovery_error) => {
        format!(" Saved session ignored ({error}); rejected save may remain ({recovery_error}).")
      }
    }
  }

  fn append_persistence_warning(status: String, warning: Option<String>) -> String {
    match warning {
      Some(warning) => format!("{status}{warning}"),
      None => status,
    }
  }

  fn save_after_command(session: &BrowserSession) -> Option<String> {
    persist_session(session).err().map(|error| {
      format!(" Save warning: current session was not persisted ({error}); use Save to retry.")
    })
  }

  fn read_persisted_session() -> Result<Option<String>, SnapshotError> {
    browser_storage()?
      .get_item(SAVE_STORAGE_KEY)
      .map_err(|error| SnapshotError::Initialization(format!("load failed: {error:?}")))
  }

  /// Loads and decodes one same-origin imported atlas layer.
  ///
  /// The returned DOM image is ready for a future WebGPU upload. Dimensions
  /// are checked against the pinned manifest before the image crosses the
  /// renderer boundary.
  pub async fn load_texture_source(
    source: AtlasTextureSource,
  ) -> Result<HtmlImageElement, JsValue> {
    let image = HtmlImageElement::new()?;
    let url = texture_source_url(source).map_err(|error| JsValue::from_str(&error.to_string()))?;
    image.set_src(&url);
    JsFuture::from(image.decode()).await?;
    validate_texture_source_dimensions(source, image.natural_width(), image.natural_height())
      .map_err(|error| JsValue::from_str(&error.to_string()))?;
    // WebGPU's external-image source reports the element's pixel dimensions;
    // pin them to the validated manifest before issuing the copy.
    image.set_width(source.width);
    image.set_height(source.height);
    Ok(image)
  }

  /// Minimal WebGPU renderer that owns no simulation state.
  pub struct WebGpuRenderer {
    _instance: wgpu::Instance,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pipeline: wgpu::RenderPipeline,
    base_texture: BaseTexturePipeline,
    canvas: HtmlCanvasElement,
    textures: Option<GpuTextureCache>,
    texture_upload_error: Option<String>,
  }

  const SCENE_SHADER: &str = r#"
struct VertexInput {
  @location(0) position: vec2<f32>,
  @location(1) color: vec4<f32>,
};

struct VertexOutput {
  @builtin(position) position: vec4<f32>,
  @location(0) color: vec4<f32>,
};

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
  var output: VertexOutput;
  output.position = vec4<f32>(input.position, 0.0, 1.0);
  output.color = input.color;
  return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
  return input.color;
}
"#;

  impl WebGpuRenderer {
    /// Requests the browser WebGPU adapter for a canvas.
    pub async fn new(canvas: HtmlCanvasElement) -> Result<Self, JsValue> {
      let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::BROWSER_WEBGPU,
        flags: wgpu::InstanceFlags::default(),
        memory_budget_thresholds: wgpu::MemoryBudgetThresholds::default(),
        backend_options: wgpu::BackendOptions::default(),
        display: None,
      });
      let surface = instance
        .create_surface(wgpu::SurfaceTarget::Canvas(canvas.clone()))
        .map_err(|error| JsValue::from_str(&format!("surface creation failed: {error}")))?;
      let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
          power_preference: wgpu::PowerPreference::HighPerformance,
          compatible_surface: Some(&surface),
          force_fallback_adapter: false,
          apply_limit_buckets: true,
        })
        .await
        .map_err(|error| JsValue::from_str(&format!("WebGPU unavailable: {error}")))?;
      let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor {
          label: Some("drl-web-device"),
          required_features: wgpu::Features::empty(),
          required_limits: adapter.limits(),
          experimental_features: wgpu::ExperimentalFeatures::disabled(),
          memory_hints: wgpu::MemoryHints::Performance,
          trace: wgpu::Trace::Off,
        })
        .await
        .map_err(|error| JsValue::from_str(&format!("WebGPU device failed: {error}")))?;
      let width = canvas.width().max(1);
      let height = canvas.height().max(1);
      let config = surface
        .get_default_config(&adapter, width, height)
        .ok_or_else(|| JsValue::from_str("WebGPU canvas format unavailable"))?;
      surface.configure(&device, &config);
      let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("drl-web-scene-shader"),
        source: wgpu::ShaderSource::Wgsl(SCENE_SHADER.into()),
      });
      let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("drl-web-scene-pipeline"),
        layout: None,
        vertex: wgpu::VertexState {
          module: &shader,
          entry_point: Some("vs_main"),
          compilation_options: wgpu::PipelineCompilationOptions::default(),
          buffers: &[Some(wgpu::VertexBufferLayout {
            array_stride: 24,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
              wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x2,
                offset: 0,
                shader_location: 0,
              },
              wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                offset: 8,
                shader_location: 1,
              },
            ],
          })],
        },
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        fragment: Some(wgpu::FragmentState {
          module: &shader,
          entry_point: Some("fs_main"),
          compilation_options: wgpu::PipelineCompilationOptions::default(),
          targets: &[Some(wgpu::ColorTargetState {
            format: config.format,
            blend: Some(wgpu::BlendState::ALPHA_BLENDING),
            write_mask: wgpu::ColorWrites::ALL,
          })],
        }),
        multiview_mask: None,
        cache: None,
      });
      let (textures, texture_upload_error) =
        match GpuTextureCache::load(&device, &queue, texture_source_manifest()).await {
          Ok(cache) => (Some(cache), None),
          Err(error) => (
            None,
            Some(
              error
                .as_string()
                .unwrap_or_else(|| "texture upload failed".to_string()),
            ),
          ),
        };
      let base_texture =
        BaseTexturePipeline::new(&device, &queue, config.format, textures.as_ref());
      Ok(Self {
        _instance: instance,
        surface,
        device,
        queue,
        config,
        pipeline,
        base_texture,
        canvas,
        textures,
        texture_upload_error,
      })
    }

    /// Returns the number of unique imported sources uploaded at startup.
    pub fn texture_source_count(&self) -> usize {
      self.textures.as_ref().map_or(0, GpuTextureCache::len)
    }

    /// Reports whether a decoded source has a retained GPU view.
    pub fn has_texture_source(&self, source: AtlasTextureSource) -> bool {
      self
        .textures
        .as_ref()
        .is_some_and(|textures| textures.view(source).is_some())
    }

    /// Returns the non-fatal upload error, if geometry fallback is active.
    pub fn texture_upload_error(&self) -> Option<&str> {
      self.texture_upload_error.as_deref()
    }

    /// Resizes only the presentation surface; it never touches simulation.
    pub fn resize(&mut self, width: u32, height: u32, dpr: f64) {
      let scale = dpr.max(1.0);
      self.config.width = ((width as f64) * scale).round().max(1.0) as u32;
      self.config.height = ((height as f64) * scale).round().max(1.0) as u32;
      self.canvas.set_width(self.config.width);
      self.canvas.set_height(self.config.height);
      self.surface.configure(&self.device, &self.config);
    }

    /// Clears the canvas and presents one deterministic frame.
    pub fn render(&self, scene: &RenderScene) -> Result<(), JsValue> {
      self.render_with_elapsed(scene, None, None)
    }

    /// Presents a frame with caller-owned retained particle decals.
    pub fn render_with_particle_decals(
      &self,
      scene: &RenderScene,
      store: &ParticleDecalStore,
      sprites: &[ParticleDecalSprite],
    ) -> Result<(), JsValue> {
      self.render_with_elapsed(scene, None, Some((store, sprites)))
    }

    /// Presents one frame using caller-supplied elapsed animation time.
    ///
    /// The renderer reads no clock and does not schedule redraws; callers own
    /// elapsed-time and playback policy decisions.
    pub fn render_at_elapsed(
      &self,
      scene: &RenderScene,
      elapsed_ms: u64,
      playback: AnimationPlayback,
    ) -> Result<(), JsValue> {
      self.render_with_elapsed(scene, Some((elapsed_ms, playback)), None)
    }

    /// Presents an elapsed-time frame with caller-owned retained decals.
    pub fn render_at_elapsed_with_particle_decals(
      &self,
      scene: &RenderScene,
      elapsed_ms: u64,
      playback: AnimationPlayback,
      store: &ParticleDecalStore,
      sprites: &[ParticleDecalSprite],
    ) -> Result<(), JsValue> {
      self.render_with_elapsed(scene, Some((elapsed_ms, playback)), Some((store, sprites)))
    }

    fn render_with_elapsed(
      &self,
      scene: &RenderScene,
      elapsed: Option<(u64, AnimationPlayback)>,
      particle_decals: Option<(&ParticleDecalStore, &[ParticleDecalSprite])>,
    ) -> Result<(), JsValue> {
      let frame = match self.surface.get_current_texture() {
        wgpu::CurrentSurfaceTexture::Success(frame)
        | wgpu::CurrentSurfaceTexture::Suboptimal(frame) => frame,
        status => {
          return Err(JsValue::from_str(&format!(
            "GPU frame unavailable: {status:?}"
          )));
        }
      };
      let view = frame
        .texture
        .create_view(&wgpu::TextureViewDescriptor::default());
      let [r, g, b, a] = scene_clear_color(scene.hud.player_hp);
      let clear = wgpu::Color {
        r: f64::from(r),
        g: f64::from(g),
        b: f64::from(b),
        a: f64::from(a),
      };
      let mut encoder = self
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
          label: Some("drl-web-frame"),
        });
      let attachments = [Some(wgpu::RenderPassColorAttachment {
        view: &view,
        depth_slice: None,
        resolve_target: None,
        ops: wgpu::Operations {
          load: wgpu::LoadOp::Clear(clear),
          store: wgpu::StoreOp::Store,
        },
      })];
      {
        let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
          label: Some("drl-web-clear"),
          color_attachments: &attachments,
          depth_stencil_attachment: None,
          timestamp_writes: None,
          occlusion_query_set: None,
          multiview_mask: None,
        });
      }
      let textured_scene = match (elapsed, particle_decals) {
        (None, None) => {
          self
            .base_texture
            .covers_scene(scene, self.config.width, self.config.height)
        }
        (Some((elapsed_ms, playback)), None) => self.base_texture.covers_scene_at_elapsed(
          scene,
          self.config.width,
          self.config.height,
          elapsed_ms,
          playback,
        ),
        (None, Some((store, sprites))) => self.base_texture.covers_scene_with_particle_decals(
          scene,
          self.config.width,
          self.config.height,
          store,
          sprites,
        ),
        (Some((elapsed_ms, playback)), Some((store, sprites))) => {
          self.base_texture.covers_scene_with_selection(
            scene,
            self.config.width,
            self.config.height,
            Some((elapsed_ms, playback)),
            Some((store, sprites)),
          )
        }
      };
      if textured_scene {
        match (elapsed, particle_decals) {
          (None, None) => self.base_texture.draw(
            &self.device,
            &mut encoder,
            &view,
            scene,
            self.config.width,
            self.config.height,
          ),
          (Some((elapsed_ms, playback)), None) => self.base_texture.draw_at_elapsed(
            &self.device,
            &mut encoder,
            &view,
            scene,
            self.config.width,
            self.config.height,
            elapsed_ms,
            playback,
          ),
          (None, Some((store, sprites))) => self.base_texture.draw_with_particle_decals(
            &self.device,
            &mut encoder,
            &view,
            scene,
            self.config.width,
            self.config.height,
            store,
            sprites,
          ),
          (Some((elapsed_ms, playback)), Some((store, sprites))) => {
            self.base_texture.draw_at_elapsed_with_particle_decals(
              &self.device,
              &mut encoder,
              &view,
              scene,
              self.config.width,
              self.config.height,
              elapsed_ms,
              playback,
              store,
              sprites,
            )
          }
        }
      }
      let vertices = if textured_scene {
        target_vertices(scene, self.config.width, self.config.height)
      } else {
        scene_vertices(scene, self.config.width, self.config.height)
      };
      if !vertices.is_empty() {
        let vertex_buffer = self
          .device
          .create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("drl-web-scene-vertices"),
            contents: &vertices,
            usage: wgpu::BufferUsages::VERTEX,
          });
        let vertex_count = (vertices.len() / 24) as u32;
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
          label: Some("drl-web-scene"),
          color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: &view,
            depth_slice: None,
            resolve_target: None,
            ops: wgpu::Operations {
              load: wgpu::LoadOp::Load,
              store: wgpu::StoreOp::Store,
            },
          })],
          depth_stencil_attachment: None,
          timestamp_writes: None,
          occlusion_query_set: None,
          multiview_mask: None,
        });
        pass.set_pipeline(&self.pipeline);
        pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        pass.draw(0..vertex_count, 0..1);
      }
      self.queue.submit([encoder.finish()]);
      self.queue.present(frame);
      Ok(())
    }
  }

  fn push_vertex(vertices: &mut Vec<u8>, x: f32, y: f32, color: [f32; 4]) {
    vertices.extend_from_slice(&x.to_ne_bytes());
    vertices.extend_from_slice(&y.to_ne_bytes());
    for component in color {
      vertices.extend_from_slice(&component.to_ne_bytes());
    }
  }

  fn push_quad(
    vertices: &mut Vec<u8>,
    left: f32,
    bottom: f32,
    right: f32,
    top: f32,
    color: [f32; 4],
  ) {
    push_vertex(vertices, left, bottom, color);
    push_vertex(vertices, right, bottom, color);
    push_vertex(vertices, right, top, color);
    push_vertex(vertices, left, bottom, color);
    push_vertex(vertices, right, top, color);
    push_vertex(vertices, left, top, color);
  }

  fn scene_position(viewport: &PixelViewport, x: i32, y: i32) -> Option<(f32, f32, f32, f32)> {
    let rect = viewport.tile_rect(drl_protocol::Position::new(x, y))?;
    let width = viewport.canvas_width.max(1) as f32;
    let height = viewport.canvas_height.max(1) as f32;
    let left = -1.0 + 2.0 * rect.x as f32 / width;
    let right = -1.0 + 2.0 * rect.x.saturating_add(rect.width) as f32 / width;
    let top = 1.0 - 2.0 * rect.y as f32 / height;
    let bottom = 1.0 - 2.0 * rect.y.saturating_add(rect.height) as f32 / height;
    Some((left, bottom, right, top))
  }

  fn scene_vertices(scene: &RenderScene, canvas_width: u32, canvas_height: u32) -> Vec<u8> {
    let viewport = PixelViewport::fit(
      scene.map_width,
      scene.map_height,
      canvas_width,
      canvas_height,
    );
    let mut vertices = Vec::new();
    for tile in &scene.tiles {
      let color = match tile.kind {
        drl_protocol::TileKind::Wall => [0.08, 0.09, 0.12, 1.0],
        drl_protocol::TileKind::DoorClosed => [0.24, 0.16, 0.09, 1.0],
        drl_protocol::TileKind::DoorOpen => [0.18, 0.20, 0.18, 1.0],
        drl_protocol::TileKind::StairsDown => [0.28, 0.24, 0.08, 1.0],
        drl_protocol::TileKind::Floor => [0.16, 0.18, 0.22, 1.0],
      };
      let color = shade_color(color, tile.lighting_band());
      if let Some((left, bottom, right, top)) =
        scene_position(&viewport, tile.position.x, tile.position.y)
      {
        push_quad(&mut vertices, left, bottom, right, top, color);
      }
    }
    for item in &scene.items {
      if let Some((left, bottom, right, top)) =
        scene_position(&viewport, item.position.x, item.position.y)
      {
        let inset_x = (right - left) * 0.28;
        let inset_y = (top - bottom) * 0.28;
        push_quad(
          &mut vertices,
          left + inset_x,
          bottom + inset_y,
          right - inset_x,
          top - inset_y,
          [0.22, 0.75, 0.35, 1.0],
        );
      }
    }
    for actor in &scene.actors {
      if let Some((left, bottom, right, top)) =
        scene_position(&viewport, actor.position.x, actor.position.y)
      {
        let inset_x = (right - left) * 0.18;
        let inset_y = (top - bottom) * 0.18;
        let color = if actor.is_player {
          [0.25, 0.75, 0.95, 1.0]
        } else {
          [0.85, 0.25, 0.24, 1.0]
        };
        push_quad(
          &mut vertices,
          left + inset_x,
          bottom + inset_y,
          right - inset_x,
          top - inset_y,
          color,
        );
      }
    }
    for target in &scene.target_positions {
      if let Some((left, bottom, right, top)) = scene_position(&viewport, target.x, target.y) {
        let inset_x = (right - left) * 0.08;
        let inset_y = (top - bottom) * 0.08;
        push_quad(
          &mut vertices,
          left + inset_x,
          bottom + inset_y,
          right - inset_x,
          top - inset_y,
          [1.0, 0.82, 0.18, 0.35],
        );
      }
    }
    vertices
  }

  fn target_vertices(scene: &RenderScene, canvas_width: u32, canvas_height: u32) -> Vec<u8> {
    let viewport = PixelViewport::fit(
      scene.map_width,
      scene.map_height,
      canvas_width,
      canvas_height,
    );
    let mut vertices = Vec::new();
    for target in &scene.target_positions {
      if let Some((left, bottom, right, top)) = scene_position(&viewport, target.x, target.y) {
        let inset_x = (right - left) * 0.08;
        let inset_y = (top - bottom) * 0.08;
        push_quad(
          &mut vertices,
          left + inset_x,
          bottom + inset_y,
          right - inset_x,
          top - inset_y,
          [1.0, 0.82, 0.18, 0.35],
        );
      }
    }
    vertices
  }

  struct WinitInputApp {
    canvas: Option<HtmlCanvasElement>,
    window: Option<WinitWindow>,
  }

  impl WinitInputApp {
    fn new(canvas: HtmlCanvasElement) -> Self {
      Self {
        canvas: Some(canvas),
        window: None,
      }
    }
  }

  fn key_name(code: KeyCode) -> Option<&'static str> {
    Some(match code {
      KeyCode::ArrowUp => "ArrowUp",
      KeyCode::ArrowRight => "ArrowRight",
      KeyCode::ArrowDown => "ArrowDown",
      KeyCode::ArrowLeft => "ArrowLeft",
      KeyCode::KeyW => "w",
      KeyCode::KeyA => "a",
      KeyCode::KeyS => "s",
      KeyCode::KeyD => "d",
      KeyCode::Numpad8 => "8",
      KeyCode::Numpad6 => "6",
      KeyCode::Numpad2 => "2",
      KeyCode::Numpad4 => "4",
      KeyCode::Numpad7 => "7",
      KeyCode::Numpad9 => "9",
      KeyCode::Numpad1 => "1",
      KeyCode::Numpad3 => "3",
      KeyCode::Numpad5 => "5",
      KeyCode::NumpadDecimal => ".",
      KeyCode::Period => ".",
      KeyCode::Space => "Space",
      KeyCode::Enter | KeyCode::NumpadEnter => "Enter",
      KeyCode::Escape => "Escape",
      KeyCode::KeyG => "g",
      KeyCode::KeyR => "r",
      KeyCode::KeyF => "f",
      KeyCode::BracketRight => ">",
      _ => return None,
    })
  }

  impl ApplicationHandler for WinitInputApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
      if self.window.is_some() {
        return;
      }
      let Some(canvas) = self.canvas.take() else {
        return;
      };
      let attributes = WinitWindow::default_attributes()
        .with_canvas(Some(canvas))
        .with_focusable(true)
        .with_prevent_default(true);
      match event_loop.create_window(attributes) {
        Ok(window) => self.window = Some(window),
        Err(error) => {
          if let Some(document) = web_sys::window().and_then(|window| window.document()) {
            set_status(&document, &format!("Browser input unavailable: {error}"));
          }
        }
      }
    }

    fn window_event(
      &mut self,
      _event_loop: &ActiveEventLoop,
      _window_id: WindowId,
      event: WindowEvent,
    ) {
      match event {
        WindowEvent::KeyboardInput { event, .. }
          if event.state == ElementState::Pressed && !event.repeat =>
        {
          let PhysicalKey::Code(code) = event.physical_key else {
            return;
          };
          if let Some(key) = key_name(code) {
            let message = dispatch_key(key);
            if let Some(document) = web_sys::window().and_then(|window| window.document()) {
              set_status(&document, &message);
            }
          }
        }
        WindowEvent::Resized(size) => resize(size.width, size.height, 1.0),
        WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
          if let Some(window) = self.window.as_ref() {
            let size = window.inner_size();
            // `inner_size` is already physical pixels here. Applying the
            // scale factor again would double-count Retina/zoom changes.
            let _ = scale_factor;
            resize(size.width, size.height, 1.0);
          }
        }
        _ => {}
      }
    }
  }

  fn update_dom(document: &web_sys::Document, observation: &PlayerObservation) {
    if let Some(hp) = document.get_element_by_id("game-hp") {
      let value = observation.player_hp.map_or_else(
        || "HP: —".to_string(),
        |hp| format!("HP: {}/{}", hp.current, hp.max),
      );
      hp.set_text_content(Some(&value));
    }
    if let Some(turn) = document.get_element_by_id("game-turn") {
      turn.set_text_content(Some(&format!("Turn: {}", observation.turn.count)));
    }
    if let Some(weapon) = document.get_element_by_id("game-weapon") {
      let value = observation.equipped_weapon.as_ref().map_or_else(
        || "Weapon: —".to_string(),
        |item| format!("Weapon: {}", item.name),
      );
      weapon.set_text_content(Some(&value));
    }
    if let Some(targets) = document.get_element_by_id("target-indicator") {
      let count = observation
        .visible_actors
        .iter()
        .filter(|actor| !actor.is_player)
        .count();
      let value = if count == 0 {
        "Targets: none visible".to_string()
      } else {
        format!("Targets: {count} visible (F selects nearest)")
      };
      targets.set_text_content(Some(&value));
    }
    if let Some(inventory) = document.get_element_by_id("inventory") {
      let controls = observation
        .inventory
        .iter()
        .map(|item| {
          format!(
            "<p>{}</p><button type=\"button\" data-action=\"equip\" data-item-id=\"{}\">Equip</button><button type=\"button\" data-action=\"use\" data-item-id=\"{}\">Use</button><button type=\"button\" data-action=\"drop\" data-item-id=\"{}\">Drop</button>",
            item.name,
            item.id.as_u64(),
            item.id.as_u64(),
            item.id.as_u64()
          )
        })
        .collect::<Vec<_>>()
        .join("");
      inventory.set_inner_html(&controls);
    }
  }

  fn update_target_status(document: &web_sys::Document, message: &str) {
    if let Some(targets) = document.get_element_by_id("target-indicator") {
      targets.set_text_content(Some(message));
    }
  }

  fn set_status(document: &web_sys::Document, message: &str) {
    if let Some(status) = document.get_element_by_id("game-status") {
      status.set_text_content(Some(message));
    }
    if let Some(log) = document.get_element_by_id("game-log") {
      log.set_text_content(Some(message));
    }
  }

  fn set_diagnostic(document: &web_sys::Document, title: &str, detail: &str, action: &str) {
    if let Some(panel) = document.get_element_by_id("game-diagnostics") {
      let _ = panel.remove_attribute("hidden");
    }
    if let Some(title_node) = document.get_element_by_id("diagnostics-title") {
      title_node.set_text_content(Some(title));
    }
    if let Some(detail_node) = document.get_element_by_id("diagnostics-detail") {
      detail_node.set_text_content(Some(detail));
    }
    if let Some(action_node) = document.get_element_by_id("diagnostics-action") {
      action_node.set_text_content(Some(action));
    }
  }

  fn render_scene(
    scene: &RenderScene,
    store: &ParticleDecalStore,
    sprites: &[ParticleDecalSprite],
  ) {
    let result = RENDERER.with(|renderer_slot| {
      renderer_slot.borrow().as_ref().map_or(Ok(()), |renderer| {
        renderer.render_with_particle_decals(scene, store, sprites)
      })
    });
    if let Err(error) = result
      && let Some(document) = web_sys::window().and_then(|window| window.document())
    {
      set_status(
        &document,
        &format!("WebGPU presentation unavailable; gameplay is unchanged: {error:?}"),
      );
      set_diagnostic(
        &document,
        "WebGPU presentation unavailable",
        &format!("The renderer reported a local presentation error ({error:?})."),
        "Gameplay is unchanged; retry after checking the desktop Chromium WebGPU environment.",
      );
    }
  }

  fn render_animation_frame(timestamp_ms: f64) {
    let Some(window) = web_sys::window() else {
      return;
    };
    let Some(document) = window.document() else {
      return;
    };
    let Some(elapsed_ms) = ANIMATION_CLOCK.with(|clock| {
      clock
        .borrow_mut()
        .elapsed_ms(document.hidden(), timestamp_ms)
    }) else {
      return;
    };
    let result = SESSION.with(|session_slot| {
      let session_ref = session_slot.borrow();
      let Some(session) = session_ref.as_ref() else {
        return Ok(());
      };
      let scene = session.scene();
      RENDERER.with(|renderer_slot| {
        renderer_slot.borrow().as_ref().map_or(Ok(()), |renderer| {
          renderer.render_at_elapsed_with_particle_decals(
            &scene,
            elapsed_ms,
            AnimationPlayback::Loop,
            session.particle_decal_store(),
            session.particle_decal_sprites(),
          )
        })
      })
    });
    if let Err(error) = result {
      set_status(
        &document,
        &format!("WebGPU animation frame unavailable; gameplay is unchanged: {error:?}"),
      );
      set_diagnostic(
        &document,
        "WebGPU animation unavailable",
        &format!("A local animation frame could not be presented ({error:?})."),
        "Gameplay is unchanged; continue without animation or reload the page.",
      );
    }
  }

  fn request_next_animation_frame() -> Result<(), JsValue> {
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("window unavailable"))?;
    let callback = Closure::wrap(Box::new(|timestamp_ms: f64| {
      render_animation_frame(timestamp_ms);
      if let Err(error) = request_next_animation_frame()
        && let Some(document) = web_sys::window().and_then(|window| window.document())
      {
        set_status(
          &document,
          &format!("Browser animation scheduling unavailable: {error:?}"),
        );
        set_diagnostic(
          &document,
          "Browser animation scheduling unavailable",
          &format!("The browser rejected a local animation-frame request ({error:?})."),
          "Gameplay state is not advanced by the failed request; reload to retry presentation.",
        );
        ANIMATION_LOOP.with(|slot| *slot.borrow_mut() = None);
      }
    }) as Box<dyn FnMut(f64)>);
    window.request_animation_frame(callback.as_ref().unchecked_ref())?;
    ANIMATION_LOOP.with(|slot| *slot.borrow_mut() = Some(callback));
    Ok(())
  }

  fn install_visibility_listener() -> Result<(), JsValue> {
    if VISIBILITY_LISTENER.with(|slot| slot.borrow().is_some()) {
      return Ok(());
    }
    let document = web_sys::window()
      .ok_or_else(|| JsValue::from_str("window unavailable"))?
      .document()
      .ok_or_else(|| JsValue::from_str("document unavailable"))?;
    let callback = Closure::wrap(Box::new(|| {
      ANIMATION_CLOCK.with(|clock| clock.borrow_mut().visibility_changed());
    }) as Box<dyn FnMut()>);
    document
      .add_event_listener_with_callback("visibilitychange", callback.as_ref().unchecked_ref())?;
    VISIBILITY_LISTENER.with(|slot| *slot.borrow_mut() = Some(callback));
    Ok(())
  }

  fn start_animation_loop() -> Result<(), JsValue> {
    if ANIMATION_LOOP.with(|slot| slot.borrow().is_some()) {
      return Ok(());
    }
    if let Err(error) = install_visibility_listener()
      && let Some(document) = web_sys::window().and_then(|window| window.document())
    {
      set_status(
        &document,
        &format!("Browser visibility lifecycle unavailable; animation continues: {error:?}"),
      );
      set_diagnostic(
        &document,
        "Browser visibility lifecycle unavailable",
        &format!("The page could not install its local visibility listener ({error:?})."),
        "Gameplay can continue; reload to retry presentation lifecycle handling.",
      );
    }
    ANIMATION_CLOCK.with(|clock| clock.borrow_mut().reset());
    request_next_animation_frame()
  }

  /// Starts the browser shell after the HTML start button has granted audio.
  #[wasm_bindgen]
  pub async fn boot() -> Result<JsValue, JsValue> {
    let window: Window =
      web_sys::window().ok_or_else(|| JsValue::from_str("window unavailable"))?;
    let document = window
      .document()
      .ok_or_else(|| JsValue::from_str("document unavailable"))?;
    let canvas = document
      .get_element_by_id("game-canvas")
      .ok_or_else(|| JsValue::from_str("#game-canvas is missing"))?
      .dyn_into::<HtmlCanvasElement>()?;
    canvas.set_width(768);
    canvas.set_height(512);
    let mut session =
      BrowserSession::new().map_err(|error| JsValue::from_str(&error.to_string()))?;
    let restore_message = match read_persisted_session() {
      Ok(Some(token)) => match session.restore_snapshot(&token) {
        Ok(()) => " Restored the saved session.".to_string(),
        Err(error) => rejected_save_message(&token, &error),
      },
      Ok(None) => String::new(),
      Err(error) => format!(" Saved session unavailable ({error})."),
    };
    let turn = session.observation().turn.count;
    let renderer = WebGpuRenderer::new(canvas.clone()).await?;
    renderer.render(&session.scene())?;
    let texture_count = renderer.texture_source_count();
    let texture_upload_error = renderer.texture_upload_error().map(str::to_owned);
    // Audio is an optional presentation effect. Browser policy, an unavailable
    // AudioContext, or a suspended context must never prevent the simulation
    // session from starting or accepting commands.
    let mut mixer = drl_audio::WebAudioMixer::new().ok();
    let audio_unlocked = if let Some(mixer) = mixer.as_mut() {
      mixer.unlock().await.is_ok()
    } else {
      false
    };
    let audio_available = mixer.is_some();
    SESSION.with(|slot| *slot.borrow_mut() = Some(session));
    RENDERER.with(|slot| *slot.borrow_mut() = Some(renderer));
    AUDIO.with(|slot| *slot.borrow_mut() = mixer);
    TARGET.with(|slot| *slot.borrow_mut() = None);
    let event_loop = EventLoop::new()
      .map_err(|error| JsValue::from_str(&format!("input loop unavailable: {error}")))?;
    event_loop.spawn_app(WinitInputApp::new(canvas));
    let status = document
      .get_element_by_id("game-status")
      .ok_or_else(|| JsValue::from_str("#game-status is missing"))?;
    let audio_message = match (audio_available, audio_unlocked) {
      (true, true) => "Ready — use arrows/WASD or numpad. Audio is gesture-gated.",
      (true, false) => "Ready — use arrows/WASD or numpad. Audio is suspended; gameplay continues.",
      (false, _) => "Ready — use arrows/WASD or numpad. Audio is unavailable; gameplay continues.",
    };
    let message = match texture_upload_error {
      Some(error) => {
        format!(
          "{audio_message}{restore_message} Texture upload unavailable; geometry fallback active ({error})."
        )
      }
      None => format!("{audio_message}{restore_message} Textures uploaded: {texture_count}."),
    };
    status.set_text_content(Some(&message));
    if let Err(error) = start_animation_loop() {
      set_status(
        &document,
        &format!("Browser animation scheduling unavailable; gameplay continues: {error:?}"),
      );
    }
    SESSION.with(|slot| {
      if let Some(session) = slot.borrow().as_ref() {
        update_dom(&document, &session.observation());
      }
    });
    Ok(JsValue::from_str(&format!("turn={turn}")))
  }

  /// A small exported key contract used by the HTML shell and WASM tests.
  #[wasm_bindgen]
  pub fn key_command(key: &str) -> String {
    let observation = BrowserSession::new().expect("fixed session").observation();
    BrowserSession::command_for_key(key, &observation)
      .map_or_else(|| "none".to_string(), |command| format!("{command:?}"))
  }

  /// Submits one focused keyboard command and redraws without exposing game
  /// state to JavaScript.
  #[wasm_bindgen]
  pub fn dispatch_key(key: &str) -> String {
    SESSION.with(|session_slot| {
      let mut session_ref = session_slot.borrow_mut();
      let Some(session) = session_ref.as_mut() else {
        return "Press Start first.".to_string();
      };
      let observation = session.observation();
      if key == "Escape" {
        TARGET.with(|target_slot| *target_slot.borrow_mut() = None);
        if let Some(document) = web_sys::window().and_then(|window| window.document()) {
          update_target_status(&document, "Targets: selection cancelled");
        }
        return "Targeting cancelled.".to_string();
      }
      if key == "f" || key == "F" {
        let target = observation
          .visible_actors
          .iter()
          .find(|actor| !actor.is_player)
          .map(|actor| actor.position);
        TARGET.with(|target_slot| *target_slot.borrow_mut() = target);
        let Some(target) = target else {
          return "No visible target.".to_string();
        };
        if let Some(document) = web_sys::window().and_then(|window| window.document()) {
          update_target_status(
            &document,
            &format!(
              "Target selected: ({}, {}). Press Enter to fire or Escape to cancel",
              target.x, target.y
            ),
          );
        }
        return format!("Target selected at ({}, {}).", target.x, target.y);
      }
      let command = if key == "Enter" {
        let Some(target) = TARGET.with(|target_slot| *target_slot.borrow()) else {
          return "No target selected.".to_string();
        };
        Command::AttackRanged(target)
      } else {
        let Some(command) = BrowserSession::command_for_key(key, &observation) else {
          return format!("Unbound key: {key}");
        };
        command
      };
      if matches!(command, Command::AttackRanged(_)) {
        TARGET.with(|target_slot| *target_slot.borrow_mut() = None);
      }
      match session.submit(command) {
        Ok(step) => {
          let persistence_warning = save_after_command(session);
          if let Some(document) = web_sys::window().and_then(|window| window.document()) {
            update_dom(&document, &step.after);
            if key == "Enter" {
              update_target_status(&document, "Targets: fired");
            }
          }
          AUDIO.with(|audio_slot| {
            if let Some(mixer) = audio_slot.borrow().as_ref() {
              for cue in drl_audio::cues_for_events(&step.events) {
                let _ = mixer.play(cue);
              }
            }
          });
          render_scene(
            &RenderScene::from_observation(&step.after),
            session.particle_decal_store(),
            session.particle_decal_sprites(),
          );
          let status = if session.is_game_over() {
            "Game over — press Restart to try again.".to_string()
          } else {
            format!("Turn {}: {:?}", step.after.turn.count, command)
          };
          if let Some(warning) = persistence_warning.as_deref()
            && let Some(document) = web_sys::window().and_then(|window| window.document())
          {
            set_status(&document, warning);
          }
          append_persistence_warning(status, persistence_warning)
        }
        Err(error) => format!("Command rejected: {error}"),
      }
    })
  }

  /// Executes an inventory action from a semantic DOM control.
  #[wasm_bindgen]
  pub fn dispatch_inventory(action: &str, item_id: u64) -> String {
    SESSION.with(|session_slot| {
      let mut session_ref = session_slot.borrow_mut();
      let Some(session) = session_ref.as_mut() else {
        return "Press Start first.".to_string();
      };
      let Some(action) = (match action {
        "equip" => Some(InventoryAction::Equip),
        "use" => Some(InventoryAction::Use),
        "drop" => Some(InventoryAction::Drop),
        _ => None,
      }) else {
        return format!("Unbound inventory action: {action}");
      };
      let command = BrowserSession::inventory_command(action, ItemId::new(item_id));
      match session.submit(command) {
        Ok(step) => {
          let persistence_warning = save_after_command(session);
          if let Some(document) = web_sys::window().and_then(|window| window.document()) {
            update_dom(&document, &step.after);
          }
          AUDIO.with(|audio_slot| {
            if let Some(mixer) = audio_slot.borrow().as_ref() {
              for cue in drl_audio::cues_for_events(&step.events) {
                let _ = mixer.play(cue);
              }
            }
          });
          render_scene(
            &RenderScene::from_observation(&step.after),
            session.particle_decal_store(),
            session.particle_decal_sprites(),
          );
          let status = if session.is_game_over() {
            "Game over — press Restart to try again.".to_string()
          } else {
            format!("Turn {}: {:?}", step.after.turn.count, command)
          };
          if let Some(warning) = persistence_warning.as_deref()
            && let Some(document) = web_sys::window().and_then(|window| window.document())
          {
            set_status(&document, warning);
          }
          append_persistence_warning(status, persistence_warning)
        }
        Err(error) => format!("Inventory action rejected: {error}"),
      }
    })
  }

  /// Resizes only the canvas surface. Visibility and DPR are presentation
  /// concerns and never submit a simulation command.
  #[wasm_bindgen]
  pub fn resize(width: u32, height: u32, dpr: f64) {
    RENDERER.with(|renderer_slot| {
      if let Some(renderer) = renderer_slot.borrow_mut().as_mut() {
        renderer.resize(width, height, dpr);
      }
    });
  }

  /// Restarts the fixed session and redraws the initial observation.
  #[wasm_bindgen]
  pub fn restart() -> String {
    SESSION.with(|session_slot| {
      let mut session_ref = session_slot.borrow_mut();
      let Some(session) = session_ref.as_mut() else {
        return "Press Start first.".to_string();
      };
      match session.restart() {
        Ok(()) => {
          let clear_warning = remove_persisted_session().err().map(|error| {
            format!(
              " Save clear warning: the previous save may remain ({error}); use Clear Save to retry."
            )
          });
          let quarantine_warning = remove_rejected_session().err().map(|error| {
            format!(" Rejected-save quarantine clear warning: {error}; use Clear Save to retry.")
          });
          ANIMATION_CLOCK.with(|clock| clock.borrow_mut().reset());
          let observation = session.observation();
          if let Some(document) = web_sys::window().and_then(|window| window.document()) {
            update_dom(&document, &observation);
          }
          render_scene(
            &RenderScene::from_observation(&observation),
            session.particle_decal_store(),
            session.particle_decal_sprites(),
          );
          let status = "Restarted deterministic M4 session.".to_string();
          let clear_warning = clear_warning.or(quarantine_warning);
          if let Some(warning) = clear_warning.as_deref()
            && let Some(document) = web_sys::window().and_then(|window| window.document())
          {
            set_status(&document, warning);
          }
          append_persistence_warning(status, clear_warning)
        }
        Err(error) => format!("Restart failed: {error}"),
      }
    })
  }

  /// Saves the successful fixed-session command history to versioned localStorage.
  #[wasm_bindgen]
  pub fn save() -> String {
    let result = SESSION.with(|session_slot| {
      let session_ref = session_slot.borrow();
      let session = session_ref
        .as_ref()
        .ok_or_else(|| SnapshotError::Initialization("Press Start first.".to_string()))?;
      persist_session(session)
    });
    match result {
      Ok(()) => "Session saved on this device.".to_string(),
      Err(error) => error.to_string(),
    }
  }

  /// Loads and transactionally restores the versioned localStorage snapshot.
  #[wasm_bindgen]
  pub fn load() -> String {
    let token = match read_persisted_session() {
      Ok(Some(token)) => token,
      Ok(None) => return "No saved session found.".to_string(),
      Err(error) => return error.to_string(),
    };
    let result = SESSION.with(|session_slot| {
      let mut session_ref = session_slot.borrow_mut();
      let session = session_ref
        .as_mut()
        .ok_or_else(|| SnapshotError::Initialization("Press Start first.".to_string()))?;
      session.restore_snapshot(&token)
    });
    match result {
      Ok(()) => {
        ANIMATION_CLOCK.with(|clock| clock.borrow_mut().reset());
        TARGET.with(|target_slot| *target_slot.borrow_mut() = None);
        if let Some(document) = web_sys::window().and_then(|window| window.document()) {
          SESSION.with(|session_slot| {
            if let Some(session) = session_slot.borrow().as_ref() {
              update_dom(&document, &session.observation());
              render_scene(
                &RenderScene::from_observation(&session.observation()),
                session.particle_decal_store(),
                session.particle_decal_sprites(),
              );
            }
          });
        }
        "Session loaded from this device.".to_string()
      }
      Err(error) => rejected_save_message(&token, &error),
    }
  }

  /// Removes the local save without changing the active simulation.
  #[wasm_bindgen]
  pub fn clear_save() -> String {
    let active_error = remove_persisted_session().err();
    let quarantine_error = remove_rejected_session().err();
    match (active_error, quarantine_error) {
      (None, None) => "Saved session cleared.".to_string(),
      (Some(error), None) | (None, Some(error)) => error.to_string(),
      (Some(active), Some(quarantine)) => {
        format!("Save clear failed: {active}; {quarantine}")
      }
    }
  }

  /// Changes the user-visible mute state without affecting gameplay.
  #[wasm_bindgen]
  pub fn set_muted(muted: bool) -> String {
    AUDIO.with(|audio_slot| {
      let mut audio_ref = audio_slot.borrow_mut();
      let Some(mixer) = audio_ref.as_mut() else {
        return "Audio unavailable; gameplay continues.".to_string();
      };
      let settings = mixer.settings();
      mixer.set_settings(muted, settings.volume);
      if muted {
        "Audio muted."
      } else {
        "Audio enabled."
      }
      .to_string()
    })
  }

  /// Changes the user-visible volume without affecting gameplay.
  #[wasm_bindgen]
  pub fn set_volume(volume: f32) -> String {
    AUDIO.with(|audio_slot| {
      let mut audio_ref = audio_slot.borrow_mut();
      let Some(mixer) = audio_ref.as_mut() else {
        return "Audio unavailable; gameplay continues.".to_string();
      };
      let settings = mixer.settings();
      mixer.set_settings(settings.muted, volume);
      format!("Audio volume: {:.0}%.", mixer.settings().volume * 100.0)
    })
  }

  /// Retries a suspended Web Audio context from a later trusted gesture.
  #[wasm_bindgen]
  pub async fn unlock_audio() -> String {
    let mixer = AUDIO.with(|audio_slot| audio_slot.borrow_mut().take());
    let Some(mut mixer) = mixer else {
      return "Audio unavailable; gameplay continues.".to_string();
    };
    let result = mixer.unlock().await;
    let unlocked = result.is_ok();
    AUDIO.with(|audio_slot| *audio_slot.borrow_mut() = Some(mixer));
    if unlocked {
      "Audio unlocked.".to_string()
    } else {
      "Audio remains suspended; gameplay continues.".to_string()
    }
  }
}

#[cfg(target_arch = "wasm32")]
pub use wasm::{
  WebGpuRenderer, boot, clear_save, dispatch_inventory, dispatch_key, key_command, load,
  load_texture_source, resize, restart, save, set_muted, set_volume, unlock_audio,
};

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn texture_source_urls_are_same_origin_and_dimensions_are_checked() {
    let source = drl_assets::AtlasId::Enemies.texture_source(drl_assets::SpriteLayer::Base);
    assert_eq!(
      texture_source_url(source).expect("manifest path"),
      "assets/legacy/drl/graphics/enemies.png"
    );
    assert_eq!(
      browser_asset_url("dguy.png").expect("safe path"),
      "assets/legacy/drl/graphics/dguy.png"
    );
    for path in [
      "/dguy.png",
      "../dguy.png",
      "foo/../bar.png",
      "dguy.png?x=1",
      "dguy.png#x",
      r"..\dguy.png",
    ] {
      assert!(browser_asset_url(path).is_err(), "{path}");
    }
    assert!(validate_texture_source_dimensions(source, 512, 192).is_ok());
    let error = validate_texture_source_dimensions(source, 256, 192).unwrap_err();
    assert_eq!(error.path, "enemies.png");
    assert_eq!(error.expected, (512, 192));
    assert_eq!(error.actual, (256, 192));
    assert!(error.to_string().contains("expected 512x192"));
  }

  #[test]
  fn texture_source_manifest_is_stable_and_deduplicated() {
    let sources = texture_source_manifest();
    assert_eq!(sources.len(), 24);
    assert_eq!(sources.first().expect("base source").path, "dguy.png");
    assert_eq!(sources.last().expect("last source").path, "fx_emissive.png");
    assert!(sources.windows(2).all(|window| window[0] != window[1]));
    assert_eq!(
      sources
        .iter()
        .filter(|source| source.path == "levels.png")
        .count(),
      1
    );
  }

  #[test]
  fn base_texture_uvs_preserve_top_left_orientation() {
    let uv = SpriteUv {
      u_min: 0.1,
      v_min: 0.2,
      u_max: 0.3,
      v_max: 0.4,
    };
    assert_eq!(
      base_texture_uvs(uv),
      [
        [0.1, 0.4],
        [0.3, 0.4],
        [0.3, 0.2],
        [0.1, 0.4],
        [0.3, 0.2],
        [0.1, 0.2],
      ]
    );
  }

  #[test]
  fn base_texture_lighting_factor_matches_fair_bands() {
    assert_eq!(base_texture_lighting_factor(LightingBand::Visible), 1.0);
    assert_eq!(base_texture_lighting_factor(LightingBand::Explored), 0.45);
  }

  #[test]
  fn emissive_role_raises_but_never_reduces_fair_light() {
    assert_eq!(emissive_lighting_floor(0.45, 0.8), 0.8);
    assert_eq!(emissive_lighting_floor(1.0, 0.8), 1.0);
  }

  #[test]
  fn emissive_role_pairing_uses_registered_atlas_source() {
    let base = AtlasId::Enemies.texture_source(drl_assets::SpriteLayer::Base);
    let emissive = AtlasId::Enemies.texture_source(drl_assets::SpriteLayer::Emissive);
    assert_eq!(base.path, "enemies.png");
    assert_eq!(emissive.path, "enemies_emissive.png");
    assert_eq!((base.width, base.height), (emissive.width, emissive.height));
  }

  #[test]
  fn outline_role_registration_preserves_optional_atlas_boundary() {
    assert!(
      AtlasId::Enemies
        .layers()
        .contains(&drl_assets::SpriteLayer::Shadow)
    );
    assert!(
      !AtlasId::Levels
        .layers()
        .contains(&drl_assets::SpriteLayer::Shadow)
    );
    let source = AtlasId::Enemies.texture_source(drl_assets::SpriteLayer::Shadow);
    assert_eq!(source.path, "enemies_shadow.png");
    assert_eq!((source.width, source.height), (512, 192));
  }

  #[test]
  fn textured_alpha_cutoff_matches_legacy_boundary() {
    assert!(!retains_textured_fragment(0.0));
    assert!(!retains_textured_fragment(0.099));
    assert!(retains_textured_fragment(0.1));
    assert!(retains_textured_fragment(1.0));
  }

  #[test]
  fn textured_shader_contract_keeps_verified_compositing_terms() {
    assert!(BASE_TEXTURE_SHADER.contains("textureSample(base_texture"));
    assert!(BASE_TEXTURE_SHADER.contains("textureSample(emissive_texture"));
    assert!(BASE_TEXTURE_SHADER.contains("textureSample(mask_texture"));
    assert!(BASE_TEXTURE_SHADER.contains("outline_texture: texture_2d<f32>"));
    assert!(BASE_TEXTURE_SHADER.contains("textureSample(outline_texture"));
    assert!(BASE_TEXTURE_SHADER.contains("mask.rgb * input.colorization.rgb"));
    assert!(BASE_TEXTURE_SHADER.contains("output.colorization = input.colorization"));
    assert!(BASE_TEXTURE_SHADER.contains("max(input.lighting.rgb"));
    assert!(BASE_TEXTURE_SHADER.contains("outline.a * (1.0 - sampled.a)"));
    assert!(BASE_TEXTURE_SHADER.contains("colorized * sampled.a + outline.rgb * outline_alpha"));
    assert!(BASE_TEXTURE_SHADER.contains("output_rgb * lighting, output_alpha"));
    assert!(BASE_TEXTURE_SHADER.contains("if (output.a < 0.1)"));
    assert!(BASE_TEXTURE_SHADER.contains("return output;"));
  }

  #[test]
  fn base_texture_ndc_rect_preserves_destination_orientation() {
    let rect = PixelRect {
      x: 10,
      y: 20,
      width: 30,
      height: 40,
    };
    let [left, bottom, right, top] = base_texture_ndc_rect(rect, 100, 100);
    assert!((left + 0.8).abs() < f32::EPSILON);
    assert!((bottom + 0.2).abs() < f32::EPSILON);
    assert!((right + 0.2).abs() < f32::EPSILON);
    assert!((top - 0.6).abs() < f32::EPSILON);
  }

  #[test]
  fn animation_elapsed_ms_is_monotonic_bounded_and_clock_free() {
    assert_eq!(animation_elapsed_ms(100.0, 100.0), Some(0));
    assert_eq!(animation_elapsed_ms(100.0, 100.9), Some(0));
    assert_eq!(animation_elapsed_ms(100.0, 101.1), Some(1));
    assert_eq!(animation_elapsed_ms(100.0, 99.0), None);
    assert_eq!(animation_elapsed_ms(f64::NAN, 100.0), None);
    assert_eq!(animation_elapsed_ms(100.0, f64::INFINITY), None);
    assert_eq!(animation_elapsed_ms(0.0, u64::MAX as f64), Some(u64::MAX));
  }

  #[test]
  fn rejected_commands_do_not_advance_the_session() {
    let mut session = BrowserSession::new().expect("fixed session");
    let before = session.observation();
    let error = session.submit(Command::Descend).unwrap_err();
    assert!(!error.is_empty());
    assert_eq!(session.observation(), before);
  }

  #[test]
  fn snapshot_round_trip_replays_fixed_session_deterministically() {
    let mut session = BrowserSession::new().expect("fixed session");
    for command in [
      Command::Move(Direction::East),
      Command::Move(Direction::East),
      Command::Move(Direction::East),
      Command::Pickup,
    ] {
      session.submit(command).expect("legal command");
    }
    let expected_observation = session.observation();
    let expected_replay = session.replay_log();
    let token = session.snapshot_token().expect("snapshot encoding");
    assert!(token.starts_with("DRL-RUST-BROWSER-SAVE/1:fixed-m4-v1:"));

    let mut restored = BrowserSession::new().expect("fixed session");
    restored.restore_snapshot(&token).expect("snapshot restore");
    assert_eq!(restored.observation(), expected_observation);
    assert_eq!(restored.replay_log(), expected_replay);
    assert_eq!(restored.snapshot_token().expect("re-encode"), token);
  }

  #[test]
  fn snapshot_rejects_corruption_and_unknown_versions() {
    let mut session = BrowserSession::new().expect("fixed session");
    assert_eq!(
      session.restore_snapshot("DRL-RUST-BROWSER-SAVE/2:fixed-m4-v1:w"),
      Err(SnapshotError::UnsupportedVersion("2".to_string()))
    );
    assert_eq!(
      session.restore_snapshot("DRL-RUST-BROWSER-SAVE/1:other:w"),
      Err(SnapshotError::UnsupportedContent("other".to_string()))
    );
    assert_eq!(
      session.restore_snapshot("not-a-snapshot"),
      Err(SnapshotError::Malformed)
    );
    assert_eq!(
      session.restore_snapshot("DRL-RUST-BROWSER-SAVE/1:fixed-m4-v1:w;;p"),
      Err(SnapshotError::Malformed)
    );
    let oversized = format!("DRL-RUST-BROWSER-SAVE/1:fixed-m4-v1:{}", "w;".repeat(8193));
    assert_eq!(
      session.restore_snapshot(&oversized),
      Err(SnapshotError::TooLarge)
    );
  }

  #[test]
  fn rejected_snapshot_keeps_the_active_session_unchanged() {
    let mut session = BrowserSession::new().expect("fixed session");
    session
      .submit(Command::Move(Direction::East))
      .expect("legal command");
    let before_observation = session.observation();
    let before_replay = session.replay_log();
    let before_token = session.snapshot_token().expect("snapshot encoding");

    assert_eq!(
      session.restore_snapshot("DRL-RUST-BROWSER-SAVE/1:fixed-m4-v1:w;;p"),
      Err(SnapshotError::Malformed)
    );
    assert_eq!(session.observation(), before_observation);
    assert_eq!(session.replay_log(), before_replay);
    assert_eq!(
      session.snapshot_token().expect("snapshot encoding"),
      before_token
    );
  }

  #[test]
  fn snapshot_codec_covers_every_command_variant() {
    let commands = [
      Command::Move(Direction::None),
      Command::Move(Direction::NorthWest),
      Command::AttackMelee(Direction::SouthEast),
      Command::AttackRanged(Position::new(-3, 8)),
      Command::Wait,
      Command::Pickup,
      Command::Drop(ItemId::new(4)),
      Command::Equip(ItemId::new(5)),
      Command::Unequip(drl_protocol::EquipmentSlot::Weapon),
      Command::Unequip(drl_protocol::EquipmentSlot::Armor),
      Command::Use(ItemId::new(6)),
      Command::Reload,
      Command::Descend,
    ];
    let token = persistence::encode_snapshot(&commands).expect("codec encoding");
    assert_eq!(
      persistence::decode_snapshot(&token).expect("codec decoding"),
      commands
    );
  }

  #[test]
  fn keyboard_mapping_covers_diagonal_numpad_and_actions() {
    let observation = BrowserSession::new().expect("fixed session").observation();
    assert_eq!(
      BrowserSession::command_for_key("7", &observation),
      Some(Command::Move(Direction::NorthWest))
    );
    assert_eq!(
      BrowserSession::command_for_key("g", &observation),
      Some(Command::Pickup)
    );
    assert_eq!(
      BrowserSession::command_for_key("r", &observation),
      Some(Command::Reload)
    );
  }

  #[test]
  fn browser_decal_requests_are_presentation_only() {
    let mut session = BrowserSession::new().expect("fixed session");
    let before = session.observation();
    session
      .try_insert_particle_decal(drl_render::ParticleDecalInsertion {
        placement: drl_render::ParticleDecalPlacement {
          cell: [1, 1],
          pixel: [32, 32],
        },
        sprite_id: 100_001,
      })
      .expect("retain presentation request");

    assert_eq!(session.observation(), before);
    assert_eq!(session.particle_decal_store().len(), 1);
    assert!(session.particle_decal_sprites().is_empty());
  }

  #[test]
  fn animation_clock_rebases_after_hidden_frames() {
    let mut clock = AnimationClock::default();
    assert_eq!(clock.elapsed_ms(false, 100.0), Some(0));
    assert_eq!(clock.elapsed_ms(false, 101.0), Some(1));
    assert_eq!(clock.elapsed_ms(true, 500.0), None);
    assert_eq!(clock.elapsed_ms(false, 501.0), Some(0));
    assert_eq!(clock.elapsed_ms(false, 502.0), Some(1));
    clock.reset();
    assert_eq!(clock.elapsed_ms(false, 900.0), Some(0));
  }

  #[test]
  fn animation_clock_rebases_on_visibility_lifecycle_change() {
    let mut clock = AnimationClock::default();
    assert_eq!(clock.elapsed_ms(false, 100.0), Some(0));
    assert_eq!(clock.elapsed_ms(false, 101.0), Some(1));
    clock.visibility_changed();
    assert_eq!(clock.elapsed_ms(false, 500.0), Some(0));
    clock.visibility_changed();
    assert_eq!(clock.elapsed_ms(false, 900.0), Some(0));
  }

  #[test]
  fn browser_session_matches_direct_core_for_identical_commands() {
    let mut browser = BrowserSession::new().expect("fixed session");
    let mut direct = BrowserSession::fixed_game().expect("fixed core game");
    let commands = [
      Command::Wait,
      Command::Move(Direction::East),
      Command::Move(Direction::East),
      Command::Move(Direction::East),
      Command::Pickup,
      Command::Pickup,
      Command::Pickup,
    ];
    for command in commands {
      let expected_events = direct.step(command).expect("direct command");
      let step = browser.submit(command).expect("browser command");
      assert_eq!(step.events, expected_events);
      assert_eq!(
        step.effects,
        drl_render::effect_timeline_for_observations(&step.before, &step.after, &expected_events)
      );
      assert_eq!(step.after, direct.observe_player());
    }
    let replay = browser.replay_log();
    let (replayed, _) = drl_core::ReplayEngine::run(&replay).expect("replay browser run");
    let browser_observation = browser.observation();
    let replay_observation = replayed.observe_player();
    assert_eq!(browser_observation, replay_observation);
    assert!(drl_core::ReplayEngine::verify_determinism(&replay).expect("replay determinism"));
  }
}

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
  use wasm_bindgen_test::*;

  wasm_bindgen_test_configure!(run_in_browser);

  #[wasm_bindgen_test]
  fn key_contract_is_stable() {
    assert!(crate::key_command("ArrowUp").contains("Move"));
  }
}
