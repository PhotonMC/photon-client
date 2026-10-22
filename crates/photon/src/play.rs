//! `photon play`: join a server and walk around in it.

use std::env;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};

use photon_core::math::Vec3;
use photon_net::connection::PlaySession;
use photon_render::VulkanBackend;
use photon_render::block::biome_tint::BiomeTints;
use photon_render::block::world_mesh::BlockRenderer;
use photon_render::window::{DisplayData, WindowSurface};

use crate::assets::FolderAssets;
use crate::bench::{Bench, RunInfo};
use crate::controls::Controls;
use crate::equipment::armor::{Armor, Worn};
use crate::equipment::held_items::held_item_vertices;
use crate::equipment::item_layers::ItemLayerCache;
use crate::frame_limiter::{FrameLimiter, is_input};
use crate::gui::Gui;
use crate::mobs;
use crate::mobs::humanoids::villagers;
use crate::objects::dropped_items::{WorldItemLayers, world_items};
use crate::objects::falling_blocks::falling_blocks;
use crate::objects::fishing_hooks::{HookView, fishing_hooks};
use crate::objects::piston_blocks::piston_blocks;
use crate::overlays::{self, FrameView, cracks, hands, hud_fov, packed_light, tick_hands};
use crate::ppm;
use crate::profile::{self, FrameProfile, Span};
use crate::screens::ScreenAssets;
use crate::script::{HeldKeys, Script};
use crate::sound::SoundSystem;
use crate::view::{CLOUD_RANGE, CameraType, camera_position, eye_position, player_camera};
use crate::world::animated_atlas::AnimatedAtlas;
use crate::world::sections::SectionMeshes;
use crate::world::sky::{Sky, SkyCamera};
use photon_game::input::pick::pick;
use photon_game::options::UNLIMITED_FRAMERATE_CUTOFF;
use photon_world::local_player::LocalPlayer;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// The client ticks twenty times a second.
const TICK_LENGTH: Duration = Duration::from_millis(50);

/// Join a server and walk around in it.
///
/// The player is the ported `LocalPlayer`: its tick runs twenty times a
/// second against the world the server streams, and what it sends is what
/// the reference client would send. WASD moves, space jumps, left shift
/// sneaks, left control sprints, the mouse looks, and escape frees the
/// cursor (a second escape closes the window).
pub fn run_play(mut args: impl Iterator<Item = String>) -> Result<()> {
    let server: SocketAddr = args
        .next()
        .unwrap_or_else(|| DEFAULT_SERVER.to_owned())
        .parse()?;
    let width: i32 = args
        .next()
        .map(|value| value.parse())
        .transpose()?
        .unwrap_or(1280);
    let height: i32 = args
        .next()
        .map(|value| value.parse())
        .transpose()?
        .unwrap_or(720);
    let session = join(server, &Status::default())?;
    let client = Client::open((width, height))?;
    Play::start(client, session)?.run()?;
    Ok(())
}

/// Where the client joins until the server list chooses (`photon play`'s default too).
pub(crate) const DEFAULT_SERVER: &str = "127.0.0.1:25565";

/// The window, its renderer and the block atlas: made once, kept between the menu and the games.
pub(crate) struct Client {
    // Fields drop in order: the swapchain goes before the window it presents to.
    pub backend: VulkanBackend,
    pub surface: WindowSurface,
    /// The framebuffer size the backend was made for.
    pub size: (u32, u32),
    pub block_renderer: Arc<BlockRenderer>,
    pub animated_atlas: AnimatedAtlas,
    /// `Minecraft.tutorial`: made with the game, so it goes on across the games played.
    pub tutorial: photon_gui::tutorial::Tutorial,
    /// The sound engine and music manager, passed between the menu and the game.
    pub audio: Option<crate::music::Carried>,
}

impl Client {
    /// Open the window and bake the blocks from `PHOTON_ASSETS`.
    pub(crate) fn open((width, height): (i32, i32)) -> Result<Self> {
        let root =
            env::var("PHOTON_ASSETS").map_err(|_| "PHOTON_ASSETS must name the asset folder")?;
        let (surface, mut backend, size) = open_window(width, height)?;
        let (block_renderer, animated_atlas) = bake_blocks(&root, &mut backend)?;
        Ok(Self {
            surface,
            backend,
            size,
            block_renderer,
            animated_atlas,
            tutorial: photon_gui::tutorial::Tutorial::new(
                crate::controls::load_options().tutorial_step,
            ),
            audio: None,
        })
    }
}

/// A resized or fullscreen window gets a swapchain of its new size; whether it did.
fn resize_to_window(
    surface: &WindowSurface,
    backend: &mut VulkanBackend,
    sized_for: &mut (u32, u32),
) -> Result<bool> {
    let size = surface.scale();
    let wanted = (
        size.framebuffer_width() as u32,
        size.framebuffer_height() as u32,
    );
    if (backend.out_of_date() || wanted != *sized_for) && wanted.0 > 0 && wanted.1 > 0 {
        *sized_for = wanted;
        backend.resize(wanted.0, wanted.1)?;
        return Ok(true);
    }
    Ok(false)
}

/// The menu's swapchain fitted to its window.
pub(crate) fn fit_swapchain(client: &mut Client) -> Result<()> {
    resize_to_window(&client.surface, &mut client.backend, &mut client.size).map(drop)
}

/// How a game ended.
pub(crate) enum Exit {
    /// The window closed.
    Quit,
    /// The player left the server: back to the title screen.
    Title,
}

/// Play the joined `session` in `client`'s window until the player leaves or the window closes.
pub(crate) fn play_in(client: Client, session: PlaySession) -> Result<(Client, Exit)> {
    Play::start(client, session)?.run()
}

/// How far a join has come: still connecting, or in the level waiting for it to load.
#[derive(Default)]
pub(crate) struct Status(std::sync::atomic::AtomicBool);

impl Status {
    pub(crate) fn loading(&self) -> bool {
        self.0.load(std::sync::atomic::Ordering::Relaxed)
    }
}

/// A join on its own thread (`ConnectScreen.connect`'s "Server Connector"), so the window keeps
/// drawing; its session, or why it failed.
pub(crate) struct JoinTask {
    pub status: Arc<Status>,
    result: std::sync::mpsc::Receiver<std::result::Result<PlaySession, String>>,
}

impl JoinTask {
    pub(crate) fn start(server: SocketAddr) -> Self {
        let status = Arc::new(Status::default());
        let (send, result) = std::sync::mpsc::channel();
        let shared = Arc::clone(&status);
        std::thread::Builder::new()
            .name("Server Connector".to_owned())
            .spawn(move || {
                let _ = send.send(join(server, &shared).map_err(|error| error.to_string()));
            })
            .expect("a thread to connect on");
        Self { status, result }
    }

    /// The session once the join has finished, or why it failed; none while it goes on.
    pub(crate) fn finished(&self) -> Option<std::result::Result<PlaySession, String>> {
        match self.result.try_recv() {
            Ok(result) => Some(result),
            Err(std::sync::mpsc::TryRecvError::Empty) => None,
            Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                Some(Err("the connection thread stopped".to_owned()))
            }
        }
    }
}

/// What a drawn frame leaves the loop to do.
enum Flow {
    Next,
    Stop,
}

/// Where a frame is seen from.
struct FrameCamera {
    partial: f64,
    /// The player's eyes.
    eye: Vec3,
    /// The camera, which backs away from the eyes when detached.
    camera: Vec3,
    /// The camera relative to the mesh origin.
    relative: [f32; 3],
    /// The camera's yaw and pitch.
    rotation: (f32, f32),
    /// The view and projection at the camera, and at the origin for the sky.
    view_projection: [[f32; 4]; 4],
    sky_view_projection: [[f32; 4]; 4],
    render_distance: i32,
    /// What the camera is in (`Camera.getFluidInCamera`), and what the fog reads of the player.
    fog: photon_render::sky::FogView,
    /// `Camera.getCullFrustum`.
    frustum: photon_render::visibility::Frustum,
    /// What the locator bar reads of the camera.
    locator: photon_gui::hud::locator::LocatorCamera,
}

/// A joined session with its window, renderer and everything the frames
/// draw.
struct Play {
    session: PlaySession,
    // Fields drop in order: the swapchain goes before the window it presents
    // to, or the driver reaches into a destroyed Wayland surface.
    backend: VulkanBackend,
    surface: WindowSurface,
    script: Option<Script>,
    block_renderer: Arc<BlockRenderer>,
    biome_tints: Arc<BiomeTints>,
    animated_atlas: AnimatedAtlas,
    /// Meshes are measured from a fixed point near where the player started.
    origin: [i32; 3],
    sections: SectionMeshes,
    sky: Sky,
    player_model: photon_render::entity::player_model::PlayerModel,
    /// The other players' models, and which default skin the local player wears.
    remote_models: overlays::RemoteModels,
    mob_models: mobs::MobModels,
    item_layers: WorldItemLayers,
    /// The frame's entity and item triangles, kept between frames so their room is reused.
    entity_vertices: Vec<photon_render::scene::Vertex>,
    item_vertices: Vec<photon_render::scene::Vertex>,
    /// What humanoids wear.
    armor: Armor,
    /// The layers of what humanoids hold, in the right and the left hand.
    worn_layers: [ItemLayerCache; 4],
    entity_textures: photon_render::entity::textures::EntityTextures,
    skin_rect: photon_render::entity::textures::TextureRect,
    gui: Gui,
    screen_assets: ScreenAssets,
    /// `ItemInHandRenderer`: the stacks the hands show and how high they are.
    item_in_hand: photon_game::item::in_hand::ItemInHand,
    /// Each item's first-person layers, by item and hand.
    held_item_layers: [ItemLayerCache; 2],
    camera: photon_render::camera::Camera,
    controls: Controls,
    /// The tutorial hints.
    tutorial: photon_gui::tutorial::Tutorial,
    /// `PHOTON_CAPTURE_FRAMES`: how many frames after the script ends are
    /// captured, and how many have been.
    capture_frames: u32,
    captured_frames: u32,
    aspect: f32,
    sized_for: (u32, u32),
    accumulated: Duration,
    last_frame: Instant,
    ticks: u64,
    /// The sound packets that have arrived.
    sounds_heard: usize,
    /// The sound engine, if there is a device to play on.
    sound: crate::sound::SoundSlot,
    /// The music of the dimension and its biomes.
    music: crate::music::GameMusic,
    /// The sounds of the place the player is in.
    ambient: crate::ambient::GameAmbient,
    /// The level's particles.
    particles: crate::particles::Particles,
    frames: u64,
    profile: Option<FrameProfile>,
    /// `PHOTON_BENCH`: the frames kept for a report.
    bench: Option<Bench>,
    /// How long a frame lasts at the least, and what lowers the rate.
    limiter: FrameLimiter,
}

/// Where the player spawned, reported with the chunks loaded and the biome
/// there.
fn report_spawn(session: &PlaySession) -> Result<Vec3> {
    let position = session.player().ok_or("no local player")?.position();
    println!(
        "play_spawn={:.3},{:.3},{:.3}",
        position.x, position.y, position.z
    );
    println!("play_chunks={}", session.world().loaded_chunk_count());
    let spawn_biome = session
        .world()
        .biome_at(
            position.x.floor() as i32,
            position.y.floor() as i32,
            position.z.floor() as i32,
        )
        .and_then(|id| {
            session
                .registry_names("minecraft:worldgen/biome")
                .get(usize::try_from(id).ok()?)
        })
        .cloned()
        .unwrap_or_default();
    println!("play_biome={spawn_biome}");
    Ok(position)
}

/// The game's window and a Vulkan backend on it, with the framebuffer size
/// the backend was made for; `PHOTON_FULLSCREEN` then starts fullscreen, as
/// F11 would switch.
fn open_window(width: i32, height: i32) -> Result<(WindowSurface, VulkanBackend, (u32, u32))> {
    let display = DisplayData {
        width,
        height,
        fullscreen: false,
    };
    let mut surface = WindowSurface::create(display, "Minecraft")?;
    let scale = surface.scale();
    let (display_handle, window_handle) = surface
        .raw_handles()
        .map_err(|error| format!("window handles unavailable: {error}"))?;
    let size = (
        scale.framebuffer_width() as u32,
        scale.framebuffer_height() as u32,
    );
    // SAFETY: the handles are the window's, and the caller holds the surface
    // beside the backend, declared first so it is dropped after it.
    let backend = unsafe { VulkanBackend::create(display_handle, window_handle, size.0, size.1) }?;
    if env::var_os("PHOTON_FULLSCREEN").is_some() {
        surface.toggle_fullscreen();
    }
    Ok((surface, backend, size))
}

/// Every block's models and one atlas for all of them (sent to the GPU),
/// as the reference bakes them at resource load; the world is then meshed a
/// 16-block section at a time as its chunks arrive and change.
fn bake_blocks(
    root: &str,
    backend: &mut VulkanBackend,
) -> Result<(Arc<BlockRenderer>, AnimatedAtlas)> {
    let source = FolderAssets {
        root: std::path::PathBuf::from(root),
    };
    let requested_mips: i32 = env::var("PHOTON_MIP_LEVELS")
        .ok()
        .map(|value| value.parse())
        .transpose()?
        .unwrap_or(4);
    let baking = Instant::now();
    let mut block_renderer = BlockRenderer::new(&source, requested_mips)
        .map_err(|error| format!("block models: {error:?}"))?;
    println!(
        "play_atlas={}x{} models={} skipped={} in {:?}",
        block_renderer.atlas.width,
        block_renderer.atlas.height,
        block_renderer.model_count(),
        block_renderer.skipped.len(),
        baking.elapsed()
    );
    // The pipelines' compiled shaders are kept beside the options between runs.
    let cache_path = crate::controls::game_directory().join("pipeline.cache");
    let cached = backend.use_pipeline_cache(cache_path)?;
    let pipelines = Instant::now();
    backend.load_atlas(&block_renderer.atlas.levels)?;
    backend.save_pipeline_cache();
    log::info!(
        "pipelines built in {:?} from a cache of {cached} bytes",
        pipelines.elapsed()
    );
    // The atlas now lives on the GPU; the renderer only keeps where the
    // sprites are, and the animations take their frames with them.
    let animated = AnimatedAtlas::new(&mut block_renderer.atlas);
    block_renderer.atlas.levels = Vec::new();
    Ok((Arc::new(block_renderer), animated))
}

/// The sun and the moon's phases in the block atlas.
fn sky_sprites(block_renderer: &BlockRenderer) -> photon_render::sky::SkySprites {
    let sprite = |name: &str| {
        block_renderer
            .atlas
            .sprites
            .get(name)
            .copied()
            .unwrap_or([0.0; 4])
    };
    let moon_names: Vec<String> = photon_render::sky::moon_sprites().collect();
    photon_render::sky::SkySprites {
        sun: sprite(photon_render::sky::SUN_SPRITE),
        end_flash: sprite(photon_render::sky::END_FLASH_SPRITE),
        moon: std::array::from_fn(|phase| sprite(&moon_names[phase])),
    }
}

/// Join a server and wait until its level has loaded around the player,
/// then a little longer for the chunks near them.
pub(crate) fn join(server: SocketAddr, status: &Status) -> Result<PlaySession> {
    let username = env::var("PHOTON_USER").unwrap_or_else(|_| "Photon".to_owned());
    let auto_jump = env::var("PHOTON_AUTO_JUMP").is_ok_and(|value| value == "1");
    println!("play_connecting={server} user={username}");
    let information = crate::controls::client_information(&crate::controls::load_options());
    let timeout = Duration::from_secs(30);
    let mut session = match crate::profile_login::from_environment(&username)? {
        Some(account) => {
            PlaySession::connect_signed_in(server, account, timeout, auto_jump, information)?
        }
        None => PlaySession::connect(server, &username, timeout, auto_jump, information)?,
    };
    status.0.store(true, std::sync::atomic::Ordering::Relaxed);
    println!("play_joined, waiting for the level");
    let started = std::time::Instant::now();
    while !session.is_loaded() {
        session.wait(Duration::from_millis(100))?;
        if started.elapsed() > Duration::from_secs(30) {
            let world = session.world();
            let cache = world.client_chunk_cache_snapshot();
            return Err(format!(
                "the level did not load within thirty seconds: player at {:?}, {} chunks, cache centre {:?}",
                session.player().map(|player| player.position()),
                world.loaded_chunk_count(),
                cache.map(|cache| (cache.view_center_x, cache.view_center_z, cache.storage_radius)),
            )
            .into());
        }
    }
    // Let the chunks around the player arrive before the first mesh.
    // A wait ends early when a packet arrives, so the ticks keep to their own 50 ms pace.
    let settle = std::time::Instant::now();
    let mut next_tick = settle;
    while settle.elapsed() < Duration::from_millis(2000) {
        session.wait(Duration::from_millis(50))?;
        if std::time::Instant::now() >= next_tick {
            session.tick(photon_world::input::Input::EMPTY)?;
            next_tick = std::time::Instant::now() + Duration::from_millis(50);
        }
    }
    Ok(session)
}

/// A texture of the game's own, `minecraft/textures/<name>.png`, with a
/// label for its errors.
pub fn texture(root: &str, name: &str, label: &str) -> Result<photon_render::DecodedImage> {
    let path = std::path::Path::new(root).join(format!("minecraft/textures/{name}.png"));
    Ok(
        photon_render::texture::png::decode_rgba(&std::fs::read(path)?)
            .map_err(|error| format!("{label}: {error:?}"))?,
    )
}

/// `ModelBakery.DESTROY_STAGES`, stacked into one image.
fn destroy_stages(root: &str) -> Result<photon_render::DecodedImage> {
    let mut cracks = photon_render::DecodedImage {
        width: 0,
        height: 0,
        rgba: Vec::new(),
    };
    for stage in 0..10 {
        let label = format!("destroy_stage_{stage}");
        let image = texture(root, &format!("block/{label}"), &label)?;
        if stage > 0 && (image.width, image.height) != (cracks.width, cracks.height / stage) {
            return Err(format!("{label}: stages differ in size").into());
        }
        cracks.width = image.width;
        cracks.height += image.height;
        cracks.rgba.extend_from_slice(&image.rgba);
    }
    Ok(cracks)
}

/// What a finished script leaves the player as.
fn report_end(session: &PlaySession, ticks: u64, (y_rot, x_rot): (f32, f32)) -> Result<()> {
    let player = session.player().ok_or("no local player")?;
    let position = player.position();
    println!(
        "play_end ticks={ticks} position={:.4},{:.4},{:.4} rotation={y_rot},{x_rot} on_ground={}",
        position.x,
        position.y,
        position.z,
        player.on_ground()
    );
    println!("play_teleports={}", session.teleports());
    println!("play_swings={}", session.swings());
    if let Some(warning) = session.warning() {
        println!("play_warning={warning}");
    }
    Ok(())
}

/// The blocks that fall or are carried (`FallingBlockRenderer`) for the frame.
fn set_moving_blocks(
    backend: &mut photon_render::VulkanBackend,
    session: &PlaySession,
    tints: (&BlockRenderer, &BiomeTints),
    view: overlays::FrameView,
) -> Result<()> {
    let mut moving = falling_blocks(session, tints, view);
    moving.extend(piston_blocks(session, tints, view));
    let moving: Vec<_> = moving
        .iter()
        .map(|(mesh, centre)| (mesh, *centre))
        .collect();
    backend.set_moving_blocks(&moving)?;
    Ok(())
}

mod audio;
mod camera;
mod chat;
mod cull;
mod draw;
mod effects;
mod fitting;
mod gizmo;
mod glints;
mod own_model;
mod pacing;
mod schedule;
mod social_toast;
mod ticks;
mod tutorial;
