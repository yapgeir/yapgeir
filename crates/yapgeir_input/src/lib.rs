use controller::{Gamepad, GamepadId};
use indexmap::IndexMap;
use keyboard::Keyboard;
use yapgeir_realm::{Realm, ResMut};

pub mod buttons;
pub mod controller;
pub mod keyboard;

#[derive(Default)]
pub struct Input {
    pub keyboard: Keyboard,
    pub gamepads: IndexMap<GamepadId, Gamepad>,
}

fn update(mut input: ResMut<Input>) {
    input.keyboard.flush();
    for (_, gamepad) in input.gamepads.iter_mut() {
        gamepad.buttons.flush();
    }
}

pub fn plugin(realm: &mut Realm) {
    realm.initialize_resource::<Input>().add_system(update);
}
