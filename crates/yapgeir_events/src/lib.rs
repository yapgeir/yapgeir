use derive_more::{Deref, DerefMut};
use yapgeir_realm::Realm;
use yapgeir_realm::ResMut;

/// A generic resource for any events used for cross-system communication.
/// Events are cleared at the beginning of every frame.
#[derive(Deref, DerefMut)]
pub struct Events<E: 'static>(Vec<E>);

fn clear_events<E: 'static>(mut e: ResMut<Events<E>>) {
    e.0.clear();
}

/// Registers a plugin for events of a specific type.
/// This ensures that Event<E> is an available resource, and
/// at the beginning of each frame events are cleared.
pub fn plugin<E: 'static>(realm: &mut Realm) {
    realm
        .add_resource(Events::<E>(Default::default()))
        .add_system(clear_events::<E>);
}
