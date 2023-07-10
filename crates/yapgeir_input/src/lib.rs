use controller::{Gamepad, GamepadId};
use derive_more::Constructor;
use indexmap::IndexMap;
use keyboard::Keyboard;
use mouse::{Mouse, MouseButtonEvent};
use yapgeir_realm::{Realm, ResMut};

pub mod buttons;
pub mod controller;
pub mod keyboard;
pub mod mouse;

#[derive(Constructor, Default, Debug, Clone, Copy, PartialEq, Hash)]
pub struct Axial<T> {
    pub x: T,
    pub y: T,
}

#[derive(Default)]
pub struct Input {
    pub mouse: Mouse,
    pub keyboard: Keyboard,
    pub gamepads: IndexMap<GamepadId, Gamepad>,
}

fn update(mut input: ResMut<Input>) {
    input.keyboard.flush();
    input.mouse.buttons.flush();
    for (_, gamepad) in input.gamepads.iter_mut() {
        gamepad.buttons.flush();
    }
}

pub fn plugin(realm: &mut Realm) {
    realm
        .initialize_resource::<Input>()
        .add_plugin(yapgeir_events::plugin::<MouseButtonEvent>)
        .add_system(update);
}
