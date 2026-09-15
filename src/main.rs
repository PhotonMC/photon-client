use std::env;
use std::net::SocketAddr;
use std::time::Duration;

use voxel_client::connection::connect_and_probe;
use voxel_client::render::VulkanBackend;
use voxel_client::window::{DisplayData, WindowSurface};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args().skip(1);
    if env::args().nth(1).as_deref() == Some("window") {
        return run_window(env::args().skip(2));
    }
    let address: SocketAddr = args
        .next()
        .unwrap_or_else(|| "127.0.0.1:25566".to_owned())
        .parse()?;
    let username = args.next().unwrap_or_else(|| "VoxelClient".to_owned());
    let ticks: u32 = args
        .next()
        .map(|value| value.parse())
        .transpose()?
        .unwrap_or(20);
    let timeout_seconds: u64 = args
        .next()
        .map(|value| value.parse())
        .transpose()?
        .unwrap_or(20);
    if timeout_seconds == 0 {
        return Err("timeout_seconds must be greater than zero".into());
    }

    let mut trace = connect_and_probe(address, &username, Duration::from_secs(timeout_seconds))?;
    for _ in 0..ticks {
        trace.world.tick();
    }

    println!("client_connection=ok");
    println!("play_login={}", trace.play_login);
    println!("world_constructed=true");
    println!("world_chunk_count={}", trace.world.loaded_chunk_count());
    println!("world_section_count={}", trace.world.loaded_section_count());
    println!(
        "update_tags_packet_count={}",
        trace.update_tags_packet_count
    );
    println!(
        "block_tag_vine_climbable={}",
        trace.world.block_state_registry().is_climbable_state(8389)
    );
    println!(
        "block_tag_vine_glide_through={}",
        trace
            .world
            .block_state_registry()
            .is_glide_through_state(8389)
    );
    println!("ticks_run={ticks}");
    println!("world_game_time={}", trace.world.game_time());
    let local_player_previous_position = trace
        .world
        .local_player()
        .map(|player| player.previous_position())
        .unwrap_or((0.0, 0.0, 0.0));
    let local_player_interpolated_position = trace
        .world
        .local_player()
        .map(|player| player.interpolated_position(0.5))
        .unwrap_or((0.0, 0.0, 0.0));
    println!("local_player_previous_position={local_player_previous_position:?}");
    println!("local_player_interpolated_position_half={local_player_interpolated_position:?}");
    println!(
        "local_player_tick_count={}",
        trace
            .world
            .local_player()
            .map(|player| player.tick_count())
            .unwrap_or(0)
    );
    Ok(())
}

/// Open the client window and report the sizing the reference rules produce.
///
/// This is the surface only: nothing is drawn yet, so the window stays blank
/// until the Vulkan backend lands. `frames` bounds the loop so the probe can be
/// run non-interactively; `0` keeps it open until the window is closed.
fn run_window(mut args: impl Iterator<Item = String>) -> Result<(), Box<dyn std::error::Error>> {
    let width: i32 = args.next().map(|value| value.parse()).transpose()?.unwrap_or(854);
    let height: i32 = args.next().map(|value| value.parse()).transpose()?.unwrap_or(480);
    let frames: u64 = args.next().map(|value| value.parse()).transpose()?.unwrap_or(0);

    let display = DisplayData {
        width,
        height,
        fullscreen: false,
    };
    let mut surface = WindowSurface::create(display, "Minecraft")?;

    let scale = surface.scale();
    println!("window_created=true");
    println!("requested_size={width}x{height}");
    println!(
        "framebuffer_size={}x{}",
        scale.framebuffer_width(),
        scale.framebuffer_height()
    );
    let (window_x, window_y) = surface.position();
    println!("window_position={window_x},{window_y}");
    println!("gui_scale={}", scale.gui_scale());
    println!(
        "gui_scaled_size={}x{}",
        scale.gui_scaled_width(),
        scale.gui_scaled_height()
    );
    // Bring the Vulkan backend up against the live surface. Until a frame is
    // presented a Wayland compositor never maps the window, so this is also
    // what makes it appear at all.
    let (display_handle, window_handle) = surface
        .raw_handles()
        .map_err(|error| format!("window handles unavailable: {error}"))?;
    let mut backend = unsafe {
        VulkanBackend::create(
            display_handle,
            window_handle,
            scale.framebuffer_width() as u32,
            scale.framebuffer_height() as u32,
        )
    }?;
    let described = backend.description().clone();
    println!("vulkan_device={}", described.device_name);
    println!("vulkan_discrete={}", described.discrete);
    println!(
        "vulkan_api={}.{}.{}",
        vk_version_major(described.api_version),
        vk_version_minor(described.api_version),
        vk_version_patch(described.api_version)
    );
    println!("swapchain_format={:?}", described.surface_format);
    println!("swapchain_images={}", described.image_count);
    println!(
        "swapchain_extent={}x{}",
        described.extent.width, described.extent.height
    );

    // Placeholder colour: no reference pixel has been compared yet.
    let captured = backend.capture_frame(CLEAR_COLOUR)?;
    let mut rendered = 1_u64;
    if let Ok(path) = env::var("VOXEL_CAPTURE_PPM") {
        write_ppm(&path, &captured)?;
        println!("captured_frame={path}");
        println!(
            "captured_size={}x{}",
            captured.width, captured.height
        );
    }

    let mut seen = 0_u64;
    while !surface.should_close() {
        for event in surface.wait() {
            if matches!(
                event,
                glfw::WindowEvent::Key(glfw::Key::Escape, _, glfw::Action::Press, _)
            ) {
                surface.request_close();
            }
            if let glfw::WindowEvent::FramebufferSize(_, _) = event {
                let scale = surface.scale();
                println!(
                    "resized framebuffer={}x{} gui_scale={} gui_scaled={}x{}",
                    scale.framebuffer_width(),
                    scale.framebuffer_height(),
                    scale.gui_scale(),
                    scale.gui_scaled_width(),
                    scale.gui_scaled_height()
                );
            }
        }
        backend.draw_clear_frame(CLEAR_COLOUR)?;
        rendered += 1;
        seen += 1;
        if frames != 0 && seen >= frames {
            break;
        }
    }
    println!("rendered_frames={rendered}");
    println!("window_closed=true");
    Ok(())
}

/// Placeholder clear colour. This is not a reference value: no Java frame has
/// been captured or compared yet, so nothing about these pixels is parity.
const CLEAR_COLOUR: [f32; 4] = [0.1, 0.12, 0.16, 1.0];

fn vk_version_major(version: u32) -> u32 {
    version >> 22
}

fn vk_version_minor(version: u32) -> u32 {
    (version >> 12) & 0x3ff
}

fn vk_version_patch(version: u32) -> u32 {
    version & 0xfff
}

/// Write a captured frame as binary PPM.
///
/// PPM keeps the readback dependency-free while the frame comparison format is
/// still undecided; P15.a will pin what captures are actually stored as.
fn write_ppm(
    path: &str,
    frame: &voxel_client::render::CapturedFrame,
) -> Result<(), Box<dyn std::error::Error>> {
    use std::io::Write as _;
    let mut out = std::io::BufWriter::new(std::fs::File::create(path)?);
    write!(out, "P6\n{} {}\n255\n", frame.width, frame.height)?;
    for pixel in frame.rgba.as_chunks::<4>().0 {
        out.write_all(&pixel[..3])?;
    }
    out.flush()?;
    Ok(())
}
