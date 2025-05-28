use core::sync::atomic::Ordering;
use core::time::Duration;

use bevy_app::{App, First, Plugin};
use bevy_diagnostic::FrameCount;
use bevy_ecs::system::Res;
use bevy_platform::sync::atomic::AtomicU64;

use crate::render::Framebuffer;

pub struct PSXTimePlugin;

impl Plugin for PSXTimePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(First, update_instant);
    }
}

static MILLISECONDS: AtomicU64 = AtomicU64::new(0);

#[unsafe(no_mangle)]
extern "Rust" fn __bevy_platform_time_instant_elapsed() -> Duration {
    Duration::from_millis(MILLISECONDS.load(Ordering::Acquire))
}

fn update_instant(count: Option<Res<FrameCount>>, buffer: Res<Framebuffer>) {
    let deci_frame_rate = match buffer.mode() {
        psx::gpu::VideoMode::NTSC => 6,
        psx::gpu::VideoMode::PAL => 5,
    };

    if let Some(&FrameCount(count)) = count.as_ref().map(Res::as_ref) {
        let milliseconds = (count as u64 * 100) / deci_frame_rate;
        MILLISECONDS.store(milliseconds, Ordering::Release);
    }
}
