use sdl2::event::Event as SdlEvent;
use yapgeir_events::Events;
use yapgeir_realm::{Exit, Realm, Res, ResMut};

fn update(
    mut event_pump: ResMut<sdl2::EventPump>,
    mut events: ResMut<Events<SdlEvent>>,
    mut exit: ResMut<Exit>,
) {
    for event in event_pump.poll_iter() {
        if matches!(event, SdlEvent::Quit { .. }) {
            **&mut *exit = true;
        }
        events.0.push(event);
    }
}

pub fn plugin(realm: &mut Realm) {
    realm
        .add_plugin(yapgeir_events::plugin::<SdlEvent>)
        .initialize_resource_with(|sdl: Res<sdl2::Sdl>| {
            sdl.event_pump().expect("Unable to get event pump")
        })
        .add_system(update);
}
