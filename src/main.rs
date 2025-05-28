#![no_std]
#![no_main]

mod default_plugins;
mod input;
mod logging;
mod render;
mod runner;
mod time;

// Is this enough? Too much? Who knows!
psx::heap!(1 MB);

use alloc::borrow::ToOwned;
use alloc::string::String;
use core::fmt::Write;
use render::{Framebuffer, Resolution};

use bevy_app::prelude::*;
use bevy_diagnostic::FrameCount;
use bevy_ecs::prelude::*;
use bevy_input::gamepad::Gamepad;
use bevy_time::prelude::*;
use psx::LoadedTIM;
use psx::gpu::VideoMode;

use crate::default_plugins::DefaultPlugins;
use crate::input::{P1, P2, PSXInputPlugin};
use crate::logging::PSXLogPlugin;
use crate::render::PSXRenderPlugin;
use crate::runner::PSXRunnerPlugin;
use crate::time::PSXTimePlugin;

#[unsafe(no_mangle)]
fn main() {
    App::new()
        .add_plugins(PSXLogPlugin)
        .add_plugins(PSXTimePlugin)
        .add_plugins(DefaultPlugins)
        .add_plugins(PSXInputPlugin)
        .add_plugins(PSXRenderPlugin {
            mode: VideoMode::NTSC,
            resolution: Resolution::W320xH240,
        })
        .add_plugins(PSXRunnerPlugin)
        .add_systems(Startup, (create_default_font, spawn_scene).chain())
        .add_systems(Update, update_frame_count_display)
        .add_systems(Update, update_fps_display)
        .add_systems(Update, (update_p1_input_display, update_p2_input_display))
        .add_systems(Last, update_text)
        .run();
}

fn spawn_scene(mut commands: Commands, font: Res<DefaultFont>, buffer: Res<Framebuffer>) {
    let (w, h) = buffer.resolution().into();
    commands.spawn((
        Text("Bevy says Hello World!".to_owned()),
        TextBox(font.new_text_box((160 - 90, 120 - 8), (w, h))),
    ));

    commands.spawn((
        Text("Frame: ?".to_owned()),
        TextBox(font.new_text_box((0, 8), (w, h))),
        FrameCountDisplay,
    ));

    commands.spawn((
        Text("FPS: ?".to_owned()),
        TextBox(font.new_text_box((0, 16), (w, h))),
        FPSDisplay,
    ));

    commands.spawn((
        Text("P1: ?".to_owned()),
        TextBox(font.new_text_box((0, 24), (w, h))),
        P1InputDisplay,
    ));

    commands.spawn((
        Text("P2: ?".to_owned()),
        TextBox(font.new_text_box((0, 32), (w, h))),
        P2InputDisplay,
    ));
}

fn create_default_font(mut commands: Commands, mut fb: ResMut<Framebuffer>) {
    commands.insert_resource(DefaultFont(fb.load_default_font()));
}

fn update_frame_count_display(
    mut query: Query<&mut Text, With<FrameCountDisplay>>,
    count: Res<FrameCount>,
) {
    for mut content in &mut query {
        content.clear();
        let _ = content.write_fmt(format_args!("Frame: {:?}", count.0));
    }
}

fn update_fps_display(mut query: Query<&mut Text, With<FPSDisplay>>, count: Res<Time<Real>>) {
    for mut content in &mut query {
        content.clear();
        let _ = content.write_fmt(format_args!("FPS: {:0.2}", count.delta_secs_f64().recip()));
    }
}

fn update_p1_input_display(
    mut query: Query<&mut Text, With<P1InputDisplay>>,
    player_1: Single<&Gamepad, With<P1>>,
) {
    let player_1 = player_1.into_inner();
    for mut content in &mut query {
        content.clear();
        let _ = content.write_str("P1: ");
        for button in player_1.get_pressed() {
            let _ = content.write_fmt(format_args!("{button:?}, "));
        }
    }
}

fn update_p2_input_display(
    mut query: Query<&mut Text, With<P2InputDisplay>>,
    player_2: Single<&Gamepad, With<P2>>,
) {
    let player_2 = player_2.into_inner();
    for mut content in &mut query {
        content.clear();
        let _ = content.write_str("P2: ");
        for button in player_2.get_pressed() {
            let _ = content.write_fmt(format_args!("{button:?}, "));
        }
    }
}

fn update_text(mut query: Query<(&Text, &mut TextBox)>) {
    for (content, mut output) in &mut query {
        output.reset();
        let _ = output.write_str(content);
        output.newline();
    }
}

#[derive(Resource, derive_more::Deref, derive_more::DerefMut)]
struct DefaultFont(LoadedTIM);

#[derive(Component, derive_more::Deref, derive_more::DerefMut)]
struct Text(String);

#[derive(Component, derive_more::Deref, derive_more::DerefMut)]
struct TextBox(psx::TextBox);

#[derive(Component)]
struct FrameCountDisplay;

#[derive(Component)]
struct FPSDisplay;

#[derive(Component)]
struct P1InputDisplay;

#[derive(Component)]
struct P2InputDisplay;

// Oh this is _so_ unsafe
#[unsafe(no_mangle)]
extern "C" fn __sync_synchronize() {
    core::sync::atomic::fence(core::sync::atomic::Ordering::SeqCst);
}
