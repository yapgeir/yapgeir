use std::{fmt::Debug, marker::PhantomData};

use hecs::World;
use nalgebra::Vector2;
use yapgeir_core::Delta;
use yapgeir_realm::{Realm, Res, ResMut};

use yapgeir_reflection::bevy_reflect::Reflect;
use yapgeir_reflection::bevy_reflect::{self, FromReflect};

use super::{
    rapier::{Rapier, RigidBody},
    simple::KinematicBody,
};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Reflect, FromReflect)]
pub enum DirectionX {
    Left,
    Right,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Reflect, FromReflect)]
pub enum DirectionY {
    Down,
    Up,
}

pub trait Directed
where
    Self: Sized,
{
    fn direction(d: Option<&Self>) -> f32;
}

impl Directed for DirectionX {
    fn direction(d: Option<&Self>) -> f32 {
        match d {
            Some(DirectionX::Left) => -1.,
            Some(DirectionX::Right) => 1.,
            None => 0.,
        }
    }
}

impl Directed for DirectionY {
    fn direction(d: Option<&Self>) -> f32 {
        match d {
            Some(DirectionY::Down) => -1.,
            Some(DirectionY::Up) => 1.,
            None => 0.,
        }
    }
}

#[derive(Debug, Default, Reflect)]
pub struct X;

#[derive(Debug, Default, Reflect)]
pub struct Y;

pub trait Axis {
    type Direction: Directed + Debug;
    fn coordinate<'a, T>(vec: &'a mut Vector2<T>) -> &'a mut T;
}

impl Axis for X {
    type Direction = DirectionX;
    fn coordinate<'a, T>(vec: &'a mut Vector2<T>) -> &'a mut T {
        &mut vec[0]
    }
}

impl Axis for Y {
    type Direction = DirectionY;
    fn coordinate<'a, T>(vec: &'a mut Vector2<T>) -> &'a mut T {
        &mut vec[1]
    }
}

// Usually velocity changes in games have some tweening in them.
// Axial acceleration is a sensible way of describing them
#[derive(Debug, Default, Reflect)]
pub struct Acceleration<A: Axis + Reflect> {
    pub acceleration: f32,
    pub limit: f32,

    #[reflect(ignore)]
    _axis: PhantomData<A>,
}

impl<A: Axis + Reflect + Debug> Acceleration<A> {
    pub fn set(&mut self, acceleration: f32, limit: f32, direction: Option<A::Direction>) {
        let d = A::Direction::direction(direction.as_ref());

        self.limit = limit * d;
        self.acceleration = acceleration
            * match direction {
                Some(_) => d,
                None => -self.acceleration.signum(),
            };
    }

    fn accelerate(&self, speed: f32, delta: f32) -> f32 {
        // Check if speed and acceleration have the same sign and speed is above limit
        let same_direction = speed.signum() == self.acceleration.signum();
        let above_limit = (speed >= self.limit && self.acceleration > 0.0)
            || (speed <= self.limit && self.acceleration < 0.0);
        if above_limit && same_direction {
            return speed;
        }

        // Calculate the new speed
        let new_speed = speed + self.acceleration * delta;

        // If the original speed was not above the limit, clamp the new speed to the limit
        if (speed <= self.limit && self.limit <= new_speed)
            || (new_speed <= self.limit && self.limit <= speed)
        {
            self.limit
        } else {
            new_speed
        }
    }
}

fn update_axis<A: Axis + Reflect + Debug + Send + Sync + 'static>(
    mut world: ResMut<World>,
    mut rapier: ResMut<Rapier>,
    delta: Res<Delta>,
) {
    // Update for rapier
    for (_, (acc, body)) in world.query_mut::<(&mut Acceleration<A>, &mut RigidBody)>() {
        let body = &mut rapier.rigid_body_set[**body];

        let mut velocity = body.linvel().clone();
        let axis_velocity = A::coordinate(&mut velocity);
        *axis_velocity = acc.accelerate(*axis_velocity, **delta);

        body.set_linvel(velocity, true);
    }

    // Update for simple physics
    for (_, (acc, body)) in world.query_mut::<(&mut Acceleration<A>, &mut KinematicBody)>() {
        let axis_velocity = A::coordinate(&mut body.velocity);
        *axis_velocity = acc.accelerate(*axis_velocity, **delta);
    }
}

pub fn plugin(realm: &mut Realm) {
    realm
        .add_system(update_axis::<X>)
        .add_system(update_axis::<Y>);
}
