use derive_more::{Constructor, Deref, DerefMut};
use hecs::{Entity, World};
use nalgebra::Vector2;
use rapier2d::prelude::{
    BroadPhase, CCDSolver, ColliderHandle, ColliderSet, DebugRenderBackend, DebugRenderObject,
    DebugRenderPipeline, ImpulseJointSet, IntegrationParameters, IslandManager, MultibodyJointSet,
    NarrowPhase, PhysicsPipeline, Point, Real, RigidBodyHandle, RigidBodySet,
};
use yapgeir_core::Delta;
use yapgeir_realm::{Plugin, Realm, Res, ResMut};
use yapgeir_world_2d::Transform;

use yapgeir_reflection::bevy_reflect::Reflect;
use yapgeir_reflection::{bevy_reflect, RealmExtensions};

#[derive(Deref, DerefMut, Constructor, Clone, Copy, Reflect)]
pub struct RigidBody(#[reflect(ignore)] RigidBodyHandle);

#[derive(Deref, DerefMut, Constructor, Clone, Copy, Reflect)]
pub struct Collider(#[reflect(ignore)] ColliderHandle);

#[derive(Default)]
pub struct Rapier {
    pub gravity: Vector2<f32>,
    pub integration_parameters: IntegrationParameters,
    pub physics_pipeline: PhysicsPipeline,
    pub island_manager: IslandManager,
    pub broad_phase: BroadPhase,
    pub narrow_phase: NarrowPhase,
    pub rigid_body_set: RigidBodySet,
    pub collider_set: ColliderSet,
    pub impulse_joint_set: ImpulseJointSet,
    pub multibody_joint_set: MultibodyJointSet,
    pub ccd_solver: CCDSolver,

    pub debug_render_pipeline: DebugRenderPipeline,
}

impl Rapier {
    pub fn new(gravity: Vector2<f32>) -> Self {
        Self {
            gravity,
            ..Default::default()
        }
    }

    pub fn debug(&mut self, backend: &mut impl DebugRenderBackend) {
        self.debug_render_pipeline.render(
            backend,
            &self.rigid_body_set,
            &self.collider_set,
            &self.impulse_joint_set,
            &self.multibody_joint_set,
            &self.narrow_phase,
        );
    }
}

pub struct LineRenderer<F>(pub F)
where
    F: FnMut(Point<Real>, Point<Real>, [f32; 4]);

impl<F> DebugRenderBackend for LineRenderer<F>
where
    F: FnMut(Point<Real>, Point<Real>, [f32; 4]),
{
    fn draw_line(&mut self, _: DebugRenderObject, a: Point<Real>, b: Point<Real>, color: [f32; 4]) {
        (&mut self.0)(a, b, color);
    }
}

fn update(mut rapier: ResMut<Rapier>, world: Res<World>, delta: Res<Delta>) {
    let rapier = &mut *rapier;
    rapier.integration_parameters.dt = **delta;

    rapier.physics_pipeline.step(
        &rapier.gravity,
        &rapier.integration_parameters,
        &mut rapier.island_manager,
        &mut rapier.broad_phase,
        &mut rapier.narrow_phase,
        &mut rapier.rigid_body_set,
        &mut rapier.collider_set,
        &mut rapier.impulse_joint_set,
        &mut rapier.multibody_joint_set,
        &mut rapier.ccd_solver,
        None,
        &(),
        &(),
    );

    for rigid_body_handle in rapier.island_manager.active_dynamic_bodies() {
        let rigid_body = &rapier.rigid_body_set[*rigid_body_handle];
        if rigid_body.user_data != 0 {
            let entity = Entity::from_bits(rigid_body.user_data as u64).unwrap();
            if let Ok(mut t) = world.get::<&mut Transform>(entity) {
                t.isometry = rigid_body.position().clone();
            }
        }
    }
}

pub struct PhysicsSettings {
    pub gravity: Vector2<f32>,
}

pub fn plugin(settings: PhysicsSettings) -> impl Plugin {
    move |realm: &mut Realm| {
        realm
            .register_type::<RigidBody>()
            .register_type::<Collider>()
            .add_resource(Rapier::new(settings.gravity))
            .add_system(update);
    }
}
