use bevy_app::{App, Plugin};
use bevy_color::{Color, Srgba};
use bevy_ecs::{
    change_detection::DetectChanges,
    resource::Resource,
    schedule::{IntoScheduleConfigs as _, Schedule, ScheduleLabel},
    system::{Res, ResMut},
};
use psx::gpu::VideoMode;

pub struct PSXRenderPlugin {
    pub mode: VideoMode,
    pub resolution: Resolution,
}

impl Plugin for PSXRenderPlugin {
    fn build(&self, app: &mut App) {
        let world = app.world_mut();

        world.init_resource::<ClearColor>();

        let fb = Framebuffer::new(self.mode, self.resolution);

        world.insert_resource(fb);

        let mut render_schedule = Schedule::new(RenderSchedule);

        render_schedule.add_systems((update_clear_color, draw_and_swap).chain());

        world.add_schedule(render_schedule);
    }
}

#[derive(ScheduleLabel, Hash, Debug, PartialEq, Eq, Clone, Copy)]
pub struct RenderSchedule;

pub fn update_clear_color(mut framebuffer: ResMut<Framebuffer>, clear_color: Res<ClearColor>) {
    if !clear_color.is_changed() && !clear_color.is_added() {
        return;
    }
    let psx_color = as_psx_color(clear_color.0.into());
    log::info!("Updating Background Color to {:?}", psx_color);
    framebuffer.set_bg_color(psx_color);
}

pub fn draw_and_swap(mut framebuffer: ResMut<Framebuffer>) {
    framebuffer.draw_sync();
    // TODO: Enabling the gamepad causes vblank waiting to fail, why?
    // framebuffer.wait_vblank();
    framebuffer.swap();
}

#[derive(Resource, derive_more::Deref, derive_more::DerefMut)]
pub struct Framebuffer {
    #[deref]
    #[deref_mut]
    buffer: psx::Framebuffer,
    mode: VideoMode,
    resolution: Resolution,
}

impl Framebuffer {
    pub fn new(mode: VideoMode, resolution: Resolution) -> Self {
        let (w, h) = resolution.into();

        let buffer = psx::Framebuffer::new((0, 0), (0, h), (w, h), mode, None).unwrap();

        Self {
            buffer,
            mode,
            resolution,
        }
    }

    pub const fn mode(&self) -> VideoMode {
        self.mode
    }

    pub const fn resolution(&self) -> Resolution {
        self.resolution
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Resolution {
    #[default]
    W320xH240,
}

impl From<Resolution> for (i16, i16) {
    fn from(value: Resolution) -> Self {
        match value {
            Resolution::W320xH240 => (320, 240),
        }
    }
}

#[derive(Resource, Clone, Debug, derive_more::Deref, derive_more::DerefMut)]
pub struct ClearColor(pub Color);

impl Default for ClearColor {
    fn default() -> Self {
        Self(Color::srgb_u8(43, 44, 47))
    }
}

const fn as_bevy_color(color: psx::gpu::Color) -> Srgba {
    let psx::gpu::Color { red, green, blue } = color;
    Srgba::rgb(red as f32 / 255., green as f32 / 255., blue as f32 / 255.)
}

const fn as_psx_color(color: Srgba) -> psx::gpu::Color {
    let Srgba {
        red, green, blue, ..
    } = color;
    let red = (red.clamp(0., 1.) * 255.) as u8;
    let green = (green.clamp(0., 1.) * 255.) as u8;
    let blue = (blue.clamp(0., 1.) * 255.) as u8;
    psx::gpu::Color { red, green, blue }
}
