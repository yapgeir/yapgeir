use std::borrow::Cow;

use bevy_reflect::{GetTypeRegistration, Reflect, TypeRegistry};
use derive_more::Deref;
use yapgeir_realm::{Realm, Resources};

pub use bevy_reflect;

#[derive(Debug, Clone, PartialEq, Eq, Reflect, Deref)]
#[deref(forward)]
pub struct Named(Cow<'static, str>);

impl<T> From<T> for Named
where
    T: Into<Cow<'static, str>>,
{
    fn from(value: T) -> Self {
        Named(value.into())
    }
}

pub trait RealmExtensions {
    fn register_type<T: GetTypeRegistration>(&mut self) -> &mut Self;
}

impl RealmExtensions for Realm {
    fn register_type<T: GetTypeRegistration>(&mut self) -> &mut Self {
        self.run_system(|resources: &mut Resources| {
            let mut registry = resources
                .get_mut::<TypeRegistry>()
                .expect("TypeRegistry is not registered as a resource!");

            registry.add_registration(T::get_type_registration());
        });

        self
    }
}
