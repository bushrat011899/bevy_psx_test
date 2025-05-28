use bevy_app::{App, Plugin, PluginsState};

use crate::render::RenderSchedule;

/// Sets up a [runner](App::set_runner) for the Bevy [application](App) which waits for VBlank
/// between calls to [`update`](App::update).
#[derive(Default)]
pub struct PSXRunnerPlugin;

impl Plugin for PSXRunnerPlugin {
    fn build(&self, app: &mut App) {
        app.set_runner(|mut app| {
            while app.plugins_state() == PluginsState::Adding {}

            app.finish();
            app.cleanup();

            loop {
                app.update();

                if let Some(exit) = app.should_exit() {
                    return exit;
                }

                app.world_mut().run_schedule(RenderSchedule);
            }
        });
    }
}
