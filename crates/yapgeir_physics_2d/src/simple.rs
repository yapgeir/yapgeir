use derive_more::Constructor;
use hecs::World;
use nalgebra::Vector2;
use yapgeir_core::Delta;
use yapgeir_realm::{Realm, Res, ResMut};
use yapgeir_world_2d::Transform;

#[derive(Constructor, Default, Clone, Debug)]
pub struct KinematicBody {
    pub velocity: Vector2<f32>,
    pub force: Vector2<f32>,
}

fn update(mut world: ResMut<World>, delta: Res<Delta>) {
    let delta = **delta;

    for (_, (b, t)) in world.query_mut::<(&mut KinematicBody, &mut Transform)>() {
        b.velocity += b.force * delta;
        t.isometry.translation.vector += b.velocity * delta;
    }
}

pub fn plugin(realm: &mut Realm) {
    realm.add_system(update);
}
