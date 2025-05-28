use alloc::string::ToString;

use bevy_app::{App, Plugin, PreUpdate};
use bevy_ecs::{
    component::Component,
    entity::Entity,
    event::EventWriter,
    query::With,
    resource::Resource,
    system::{NonSendMut, Single},
};
use bevy_input::gamepad::{
    GamepadAxis, GamepadConnection, GamepadConnectionEvent, RawGamepadAxisChangedEvent,
    RawGamepadButtonChangedEvent, RawGamepadEvent,
};

/// Integrates [`psx`] with [`bevy_input`] to expose player 1 and player 2 gamepads.
/// These can be found using the [`P1`] and [`P2`] [components](Component).
/// The state of the gamepad(s) is polled during [`PreUpdate`] once per update.
///
/// # Notes
///
/// - Once added, [`wait_vblank`](psx::Framebuffer::wait_vblank) will never return.
///   It is unknown at this time if this is intentional clearing of the VBlank interrupt
///   by the kernel or not.
///
/// - There is no mechanism to easily detect when a controller is dis/connected,
///   so both gamepads will be "connected" at startup and remain so.
pub struct PSXInputPlugin;

impl Plugin for PSXInputPlugin {
    fn build(&self, app: &mut App) {
        app.insert_non_send_resource(GamepadStates::new());
        app.add_systems(PreUpdate, update_gamepads);
    }

    fn finish(&self, app: &mut App) {
        let world = app.world_mut();

        let p1 = world.spawn(P1 {}).id();
        let p2 = world.spawn(P2 {}).id();

        for entity in [p1, p2] {
            let event = GamepadConnectionEvent::new(
                entity,
                GamepadConnection::Connected {
                    name: "Sony PlayStation DualShock".to_string(),
                    vendor_id: Some(0x054C),
                    product_id: None,
                },
            );

            world.send_event::<GamepadConnectionEvent>(event.clone());
            world.send_event::<RawGamepadEvent>(event.into());
        }
    }
}

fn update_gamepads(
    mut state: NonSendMut<GamepadStates>,
    mut events: EventWriter<RawGamepadEvent>,
    mut button_events: EventWriter<RawGamepadButtonChangedEvent>,
    mut axis_events: EventWriter<RawGamepadAxisChangedEvent>,
    p1: Single<Entity, With<P1>>,
    p2: Single<Entity, With<P2>>,
) {
    let p1 = p1.into_inner();
    let p2 = p2.into_inner();

    let mut p1_new = GamepadState {
        buttons: state.manager.poll_p1(),
        left: state.manager.poll_lstick_p1(),
        right: state.manager.poll_rstick_p1(),
    };

    let mut p2_new = GamepadState {
        buttons: state.manager.poll_p2(),
        left: state.manager.poll_lstick_p2(),
        right: state.manager.poll_rstick_p2(),
    };

    core::mem::swap(&mut p1_new, &mut state.p1);
    core::mem::swap(&mut p2_new, &mut state.p2);

    let p1_old = p1_new;
    let p2_old = p2_new;

    for (id, old, new) in [(p1, &p1_old, &state.p1), (p2, &p2_old, &state.p2)] {
        // Left & Right sticks
        for (old, new, axis) in [
            (
                old.left.horizontal(),
                new.left.horizontal(),
                GamepadAxis::LeftStickX,
            ),
            (
                old.left.vertical(),
                new.left.vertical(),
                GamepadAxis::LeftStickY,
            ),
            (
                old.right.horizontal(),
                new.right.horizontal(),
                GamepadAxis::RightStickX,
            ),
            (
                old.right.vertical(),
                new.right.vertical(),
                GamepadAxis::RightStickY,
            ),
        ] {
            if old != new {
                let event = RawGamepadAxisChangedEvent::new(id, axis, new.into());
                axis_events.write(event.clone());
                events.write(event.into());
            }
        }

        // Buttons
        for psx in all_buttons() {
            let bevy = as_bevy_button(psx);
            let value = new.buttons.pressed(psx);
            if old.buttons.pressed(psx) != value {
                let value = if value { 1 } else { 0 } as f32;
                let event = RawGamepadButtonChangedEvent::new(id, bevy, value);
                button_events.write(event.clone());
                events.write(event.into());
            }
        }
    }
}

/// Player 1
#[derive(Component)]
#[component(immutable)]
#[component(storage = "SparseSet")]
#[non_exhaustive]
pub struct P1 {}

/// Player 2
#[derive(Component)]
#[component(immutable)]
#[component(storage = "SparseSet")]
#[non_exhaustive]
pub struct P2 {}

#[derive(Resource)]
struct GamepadState {
    buttons: psx::sys::gamepad::Buttons,
    left: psx::sys::gamepad::JoyStick,
    right: psx::sys::gamepad::JoyStick,
}

struct GamepadStates {
    p1: GamepadState,
    p2: GamepadState,
    manager: psx::sys::gamepad::Gamepad<'static>,
}

impl GamepadStates {
    fn new() -> Self {
        let mut manager = psx::sys::gamepad::Gamepad::new();

        GamepadStates {
            p1: GamepadState {
                buttons: manager.poll_p1(),
                left: manager.poll_lstick_p1(),
                right: manager.poll_rstick_p1(),
            },
            p2: GamepadState {
                buttons: manager.poll_p2(),
                left: manager.poll_lstick_p2(),
                right: manager.poll_rstick_p2(),
            },
            manager,
        }
    }
}

const fn as_bevy_button(button: psx::sys::gamepad::Button) -> bevy_input::gamepad::GamepadButton {
    use bevy_input::gamepad::GamepadButton;
    use psx::sys::gamepad::Button::*;

    match button {
        Select => GamepadButton::Select,
        L3 => GamepadButton::LeftThumb,
        R3 => GamepadButton::RightThumb,
        Start => GamepadButton::Start,
        Up => GamepadButton::DPadUp,
        Right => GamepadButton::DPadRight,
        Down => GamepadButton::DPadDown,
        Left => GamepadButton::DPadLeft,
        L2 => GamepadButton::LeftTrigger2,
        R2 => GamepadButton::RightTrigger2,
        L1 => GamepadButton::LeftTrigger,
        R1 => GamepadButton::RightTrigger,
        Triangle => GamepadButton::North,
        Circle => GamepadButton::East,
        Cross => GamepadButton::South,
        Square => GamepadButton::West,
    }
}

const fn all_buttons() -> [psx::sys::gamepad::Button; 16] {
    use psx::sys::gamepad::Button::*;

    [
        Select, L3, R3, Start, Up, Right, Down, Left, L2, R2, L1, R1, Triangle, Circle, Cross,
        Square,
    ]
}
