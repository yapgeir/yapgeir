use derive_more::{AsRef, Constructor, Deref, From};
use hecs::{Entity, Without, World};
use nalgebra::Vector2;
use yapgeir_assets::atlas::SubTexture;
use yapgeir_realm::{system, Realm, Res, ResMut};
use yapgeir_world_2d::{Flip, Transform, WorldCamera};

use yapgeir_reflection::bevy_reflect::Reflect;
use yapgeir_reflection::{bevy_reflect, RealmExtensions};

/// An uncropped size of the sprite in meters. Used by DebugRenderer.
#[derive(Debug, Clone, Copy, Deref, AsRef, From)]
pub struct DebugSize(pub Vector2<f32>);

#[derive(Constructor, Debug, Default, Reflect)]
pub struct Drawable {
    #[reflect(ignore)]
    pub sub_texture: SubTexture,
}

// Sprite's quad in world space
#[derive(Debug, Clone, Deref, From, Default, Reflect)]
pub struct DrawQuad([[f32; 2]; 4]);

impl DrawQuad {
    fn update_model(
        (_, (transform, drawable, model)): (Entity, (&Transform, &Drawable, &mut DrawQuad)),
    ) {
        let mut mat = transform.isometry.to_homogeneous();
        match transform.flip {
            Some(Flip::X) => mat[0] *= -1.,
            Some(Flip::Y) => mat[4] *= -1.,
            None => {}
        };

        let points = drawable.sub_texture.clip.points();
        *model = points.map(|p| mat.transform_point(&p).into()).into();
    }
}

#[derive(Default)]
struct DrawQuadAdder(Vec<Entity>);

#[system]
impl DrawQuadAdder {
    fn update(&mut self, mut world: ResMut<World>) {
        self.0.clear();
        for (e, _) in world
            .query::<Without<(&Transform, &Drawable), &DrawQuad>>()
            .iter()
        {
            self.0.push(e);
        }

        for e in &self.0 {
            world
                .insert_one(e.clone(), DrawQuad::default())
                .expect("Unable to insert Drawable for entity");
        }
    }
}

fn update_quads(world: Res<World>) {
    world
        .query::<(&Transform, &Drawable, &mut DrawQuad)>()
        .iter()
        .for_each(DrawQuad::update_model)
}

pub fn plugin(realm: &mut Realm) {
    realm
        .register_type::<DrawQuad>()
        .register_type::<Drawable>()
        .initialize_resource::<WorldCamera>()
        .add_system(DrawQuadAdder::default())
        .add_system(update_quads);
}
