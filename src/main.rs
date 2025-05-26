#![no_std]
#![no_main]

// Is this enough? Too much? Who knows!
psx::heap!(1 MB);

use core::fmt::Write;

use alloc::borrow::ToOwned;
use alloc::string::String;
use bevy_app::prelude::*;
use bevy_diagnostic::{DiagnosticsPlugin, FrameCount, FrameCountPlugin};
use bevy_ecs::prelude::*;
use bevy_ecs::system::RunSystemOnce;
use psx::LoadedTIM;
use psx::gpu::VideoMode;

const RES: (i16, i16) = (320, 240);

#[unsafe(no_mangle)]
fn main() {
    App::new()
        .add_plugins((DiagnosticsPlugin, FrameCountPlugin))
        .set_runner(move |mut app| {
            loop {
                app.update();

                if let Some(exit) = app.should_exit() {
                    return exit;
                }

                let _ = app.world_mut().run_system_once(render);
            }
        })
        .add_systems(
            Startup,
            (create_framebuffer, create_default_font, spawn_scene).chain(),
        )
        .add_systems(Update, update_frame_count_display)
        .add_systems(Last, update_text)
        .run();
}

fn create_framebuffer(world: &mut World) {
    let buf0 = (0, 0);
    let buf1 = (0, 240);
    let fb = psx::Framebuffer::new(buf0, buf1, RES, VideoMode::NTSC, None).unwrap();
    world.insert_resource(Framebuffer(fb));
}

fn create_default_font(mut commands: Commands, mut fb: ResMut<Framebuffer>) {
    commands.insert_resource(DefaultFont(fb.load_default_font()));
}

fn spawn_scene(mut commands: Commands, font: Res<DefaultFont>) {
    commands.spawn((
        Text("Bevy says Hello World!".to_owned()),
        TextBox(font.new_text_box((160 - 90, 120 - 8), RES)),
    ));

    commands.spawn((
        Text("Frame: ?".to_owned()),
        TextBox(font.new_text_box((0, 8), RES)),
        FrameCountDisplay,
    ));
}

fn update_frame_count_display(
    mut query: Query<&mut Text, With<FrameCountDisplay>>,
    count: Res<FrameCount>,
) {
    for mut content in &mut query {
        content.clear();
        let _ = content.write_fmt(format_args!("Frame: {}", count.0));
    }
}

fn update_text(mut query: Query<(&Text, &mut TextBox)>) {
    for (content, mut output) in &mut query {
        output.reset();
        let _ = output.write_str(content);
        output.newline();
    }
}

fn render(mut framebuffer: ResMut<Framebuffer>) {
    framebuffer.draw_sync();
    framebuffer.wait_vblank();
    framebuffer.swap();
}

#[derive(Resource, derive_more::Deref, derive_more::DerefMut)]
struct Framebuffer(psx::Framebuffer);

#[derive(Resource, derive_more::Deref, derive_more::DerefMut)]
struct DefaultFont(LoadedTIM);

#[derive(Component, derive_more::Deref, derive_more::DerefMut)]
struct Text(String);

#[derive(Component, derive_more::Deref, derive_more::DerefMut)]
struct TextBox(psx::TextBox);

#[derive(Component)]
struct FrameCountDisplay;

// Oh this is _so_ unsafe
#[unsafe(no_mangle)]
extern "C" fn __sync_synchronize() {
    core::sync::atomic::fence(core::sync::atomic::Ordering::SeqCst);
}
