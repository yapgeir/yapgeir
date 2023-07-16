use std::{any::TypeId, collections::HashMap, marker::PhantomData};

use bevy_reflect::{GetTypeRegistration, Reflect, ReflectMut, TypeRegistry};
use derive_more::Deref;
use hecs::{Component, EntityRef};
use yapgeir_realm::{Realm, ResMut};

pub use bevy_reflect;

#[derive(Default)]
pub struct Reflection {
    pub type_registry: TypeRegistry,
    pub component_visitors: ComponentVisitors,
}

pub trait RealmExtensions {
    fn register_type<T: GetTypeRegistration + Reflect + 'static>(&mut self) -> &mut Self;
}

pub trait ComponentVisitor {
    fn visit<'a>(&self, entity: EntityRef, visitor: Box<dyn FnMut(ReflectMut) + 'a>);
}

struct TypedComponentVisitor<T>(PhantomData<T>);

impl<T: Reflect> ComponentVisitor for TypedComponentVisitor<T> {
    fn visit<'a>(&self, entity: EntityRef, mut visitor: Box<dyn FnMut(ReflectMut) + 'a>) {
        let mut component = entity.get::<&mut T>().unwrap();
        let reflect = component.reflect_mut();
        visitor(reflect);
    }
}

#[derive(Default, Deref)]
pub struct ComponentVisitors(HashMap<TypeId, Box<dyn ComponentVisitor>>);

fn register_type<'a, T: GetTypeRegistration + Reflect + Component>(
    reflection: Option<ResMut<Reflection>>,
) {
    let mut reflection = match reflection {
        Some(registry) => registry,
        None => return,
    };

    reflection
        .type_registry
        .add_registration(T::get_type_registration());

    reflection.component_visitors.0.insert(
        TypeId::of::<T>(),
        Box::new(TypedComponentVisitor::<T>(Default::default())),
    );
}

impl RealmExtensions for Realm {
    fn register_type<T: GetTypeRegistration + Reflect + 'static>(&mut self) -> &mut Self {
        self.run_system(register_type::<T>)
    }
}

pub fn plugin(realm: &mut Realm) {
    realm.initialize_resource::<Reflection>();
}
