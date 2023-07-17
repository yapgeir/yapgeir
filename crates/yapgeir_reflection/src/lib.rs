use std::{any::TypeId, collections::HashMap, marker::PhantomData};

use bevy_reflect::{std_traits::ReflectDefault, GetTypeRegistration, Reflect, TypeRegistry};
use derive_more::Deref;
use hecs::{Component, EntityRef};
use yapgeir_realm::{resource_exists, IntoFilteredSystem, Realm, ResMut};

pub use bevy_reflect;

#[derive(Default)]
pub struct Reflection {
    pub type_registry: TypeRegistry,
    pub component_visitors: ComponentVisitors,
}

pub trait ComponentVisitor {
    fn visit<'a>(&self, entity: EntityRef, visitor: Box<dyn FnMut(&mut dyn Reflect) + 'a>);
}

struct TypedComponentVisitor<T>(PhantomData<T>);

impl<T: Reflect> ComponentVisitor for TypedComponentVisitor<T> {
    fn visit<'a>(&self, entity: EntityRef, mut visitor: Box<dyn FnMut(&mut dyn Reflect) + 'a>) {
        let mut component = entity.get::<&mut T>().unwrap();
        visitor(component.as_reflect_mut());
    }
}

#[derive(Default, Deref)]
pub struct ComponentVisitors(HashMap<TypeId, Box<dyn ComponentVisitor>>);

fn register_non_default<'a, T: GetTypeRegistration + Reflect + Component>(
    mut reflection: ResMut<Reflection>,
) {
    reflection.type_registry.register::<T>();
    reflection.component_visitors.0.insert(
        TypeId::of::<T>(),
        Box::new(TypedComponentVisitor::<T>(Default::default())),
    );
}

fn register_default<'a, T: GetTypeRegistration + Reflect + Default + Component>(
    mut reflection: ResMut<Reflection>,
) {
    reflection
        .type_registry
        .register_type_data::<T, ReflectDefault>();
}

pub trait RealmExtensions {
    fn register_type<T>(&mut self) -> &mut Self
    where
        T: GetTypeRegistration + Reflect + Default + 'static;

    fn register_non_default_type<T>(&mut self) -> &mut Self
    where
        T: GetTypeRegistration + Reflect + Component + 'static;
}

impl RealmExtensions for Realm {
    fn register_type<T>(&mut self) -> &mut Self
    where
        T: GetTypeRegistration + Reflect + Default + 'static,
    {
        self.register_non_default_type::<T>()
            .run_system(register_default::<T>.filter(resource_exists::<Reflection>()))
    }

    fn register_non_default_type<T>(&mut self) -> &mut Self
    where
        T: GetTypeRegistration + Reflect + Component + 'static,
    {
        self.run_system(register_non_default::<T>.filter(resource_exists::<Reflection>()))
    }
}

pub fn plugin(realm: &mut Realm) {
    realm.initialize_resource::<Reflection>();
}
