use bevy_reflect::{GetTypeRegistration, TypeRegistry};
use yapgeir_realm::{Realm, Resources};

pub use bevy_reflect;

pub trait RealmExtensions {
    fn register_type<T: GetTypeRegistration>(&mut self) -> &mut Self;
}

impl RealmExtensions for Realm {
    fn register_type<T: GetTypeRegistration>(&mut self) -> &mut Self {
        self.run_system(|resources: &mut Resources| {
            let mut registry = match resources.get_mut::<TypeRegistry>() {
                Some(registry) => registry,
                None => return,
            };

            registry.add_registration(T::get_type_registration());
        });

        self
    }
}
