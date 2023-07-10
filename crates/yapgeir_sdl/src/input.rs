use std::{collections::HashMap, rc::Rc};

use sdl2::{controller::Axis, event::WindowEvent};
use sdl2::{controller::GameController, event::Event as SdlEvent};
use yapgeir_core::Ppt;
use yapgeir_events::Events;
use yapgeir_input::{
    controller::{GamepadButton, GamepadId},
    Input,
};
use yapgeir_realm::{Realm, Res, ResMut};

pub struct SdlControllers {
    subsystem: sdl2::GameControllerSubsystem,
    controllers: HashMap<u32, GameController>,
}

impl SdlControllers {
    pub fn new(subsystem: sdl2::GameControllerSubsystem) -> Self {
        Self {
            subsystem,
            controllers: Default::default(),
        }
    }
}

fn gamepad_button(button: &sdl2::controller::Button) -> GamepadButton {
    match button {
        sdl2::controller::Button::A => GamepadButton::A,
        sdl2::controller::Button::B => GamepadButton::B,
        sdl2::controller::Button::X => GamepadButton::X,
        sdl2::controller::Button::Y => GamepadButton::Y,
        sdl2::controller::Button::Back => GamepadButton::Back,
        sdl2::controller::Button::Guide => GamepadButton::Guide,
        sdl2::controller::Button::Start => GamepadButton::Start,
        sdl2::controller::Button::LeftStick => GamepadButton::LeftStick,
        sdl2::controller::Button::RightStick => GamepadButton::RightStick,
        sdl2::controller::Button::LeftShoulder => GamepadButton::LeftShoulder,
        sdl2::controller::Button::RightShoulder => GamepadButton::RightShoulder,
        sdl2::controller::Button::DPadUp => GamepadButton::DPadUp,
        sdl2::controller::Button::DPadDown => GamepadButton::DPadDown,
        sdl2::controller::Button::DPadLeft => GamepadButton::DPadLeft,
        sdl2::controller::Button::DPadRight => GamepadButton::DPadRight,
        sdl2::controller::Button::Misc1 => GamepadButton::Misc1,
        sdl2::controller::Button::Paddle1 => GamepadButton::Paddle1,
        sdl2::controller::Button::Paddle2 => GamepadButton::Paddle2,
        sdl2::controller::Button::Paddle3 => GamepadButton::Paddle3,
        sdl2::controller::Button::Paddle4 => GamepadButton::Paddle4,
        sdl2::controller::Button::Touchpad => GamepadButton::TouchPad,
    }
}

fn update(
    mut input: ResMut<Input>,
    mut controllers: ResMut<SdlControllers>,
    mut ppt: ResMut<Ppt>,
    events: Res<Events<SdlEvent>>,
    window: Res<Rc<sdl2::video::Window>>,
) {
    for e in &**events {
        match e {
            SdlEvent::KeyDown {
                scancode: Some(scancode),
                ..
            } => input.keyboard.state.set(*scancode as usize, true),
            SdlEvent::KeyUp {
                scancode: Some(scancode),
                ..
            } => input.keyboard.state.set(*scancode as usize, false),
            SdlEvent::ControllerAxisMotion {
                which, axis, value, ..
            } => {
                let gamepad = input
                    .gamepads
                    .get_mut(&GamepadId::new(*which))
                    .expect("gamepad not found");
                match axis {
                    Axis::LeftX => gamepad.left_stick.0 = *value,
                    Axis::LeftY => gamepad.left_stick.1 = *value,
                    Axis::RightX => gamepad.right_stick.0 = *value,
                    Axis::RightY => gamepad.right_stick.1 = *value,
                    Axis::TriggerLeft => gamepad.left_trigger = *value,
                    Axis::TriggerRight => gamepad.right_trigger = *value,
                }
            }
            SdlEvent::ControllerButtonDown { button, which, .. } => input
                .gamepads
                .get_mut(&GamepadId::new(*which))
                .expect("gamepad not found")
                .buttons
                .state
                .set(gamepad_button(button) as usize, true),
            SdlEvent::ControllerButtonUp { button, which, .. } => input
                .gamepads
                .get_mut(&GamepadId::new(*which))
                .expect("gamepad not found")
                .buttons
                .state
                .set(gamepad_button(button) as usize, false),
            SdlEvent::ControllerDeviceAdded { which, .. } => {
                let controller = controllers
                    .subsystem
                    .open(*which)
                    .expect("Unable to open controller");

                controllers.controllers.insert(*which, controller);
                input.gamepads.insert(GamepadId(*which), Default::default());
            }
            SdlEvent::ControllerDeviceRemoved { which, .. } => {
                controllers.controllers.remove(which);
                input.gamepads.remove(&GamepadId(*which));
            }
            SdlEvent::Window {
                win_event: WindowEvent::Moved(_, _),
                ..
            } => {
                ppt.0 = window.drawable_size().0 as f32 / window.size().0 as f32;
            }
            _ => {}
        }
    }
}

pub fn plugin(realm: &mut Realm) {
    realm
        .add_plugin(yapgeir_input::plugin)
        .initialize_resource_with(|sdl: Res<sdl2::Sdl>| {
            let subsystem = sdl
                .game_controller()
                .expect("Unable to initialize game controller");

            SdlControllers::new(subsystem)
        })
        .add_system(update);
}
