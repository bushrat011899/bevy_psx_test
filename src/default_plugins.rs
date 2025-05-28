use bevy_app::plugin_group;

plugin_group! {
    /// This plugin group will add all the default plugins for a *Bevy* application:
    pub struct DefaultPlugins {
        // bevy_app:::TaskPoolPlugin,
        bevy_diagnostic:::FrameCountPlugin,
        bevy_time:::TimePlugin,
        bevy_transform:::TransformPlugin,
        bevy_diagnostic:::DiagnosticsPlugin,
        bevy_input:::InputPlugin,
        bevy_state::app:::StatesPlugin,
    }
}
