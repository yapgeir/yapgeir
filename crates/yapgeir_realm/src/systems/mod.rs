use std::marker::PhantomData;

use derive_more::{Deref, DerefMut};

use crate::Resources;

pub use errors::*;
pub use filter::*;
pub use param::*;

mod errors;
mod filter;
mod param;

#[derive(Deref, DerefMut, Default)]
pub struct Exit(bool);

pub trait System<R = ()> {
    fn run(&mut self, resources: &mut Resources) -> R;
}

#[derive(Default)]
pub struct SystemRunner {
    systems: Vec<Box<dyn System<()>>>,
}

impl SystemRunner {
    #[inline]
    pub fn push<I, S: System<()> + 'static>(
        &mut self,
        system: impl IntoSystem<I, (), System = S>,
    ) -> usize {
        let len = self.systems.len();
        self.systems.push(Box::new(system.system()));
        len
    }

    #[inline]
    pub fn remove(&mut self, index: usize) {
        self.systems.remove(index);
    }

    pub fn run(&mut self, resources: &mut Resources) -> bool {
        for system in &mut self.systems {
            system.run(resources);
            if resources.get::<Exit>().is_some_and(|e| e.0) {
                return false;
            }
        }

        true
    }
}

// SystemRunner can also be used as a system
impl System<()> for SystemRunner {
    #[inline]
    fn run(&mut self, resources: &mut Resources) {
        self.run(resources);
    }
}

impl<R, T> System<R> for T
where
    T: Fn(&mut Resources) -> R,
{
    fn run(&mut self, resources: &mut Resources) -> R {
        self(resources)
    }
}

pub trait IntoSystem<Args, R = ()> {
    type System: System<R>;
    fn system(self) -> Self::System;
}

impl<R, T> IntoSystem<(), R> for T
where
    T: System<R>,
{
    type System = T;
    fn system(self) -> Self::System {
        self
    }
}

// A wrapper for system functions.
pub struct FunctionSystem<F, Args>(F, PhantomData<fn() -> Args>);

macro_rules! impl_system {
    ($($params:ident),*) => {
        #[allow(unused_parens)]
        impl<F, R, $($params: SystemParam),*> System<R> for FunctionSystem<F, ($($params),*)>
        where
            for<'r> F: FnMut($($params),*) -> R + FnMut($(<$params as SystemParam>::Item<'r>),*) -> R,
        {
            fn run(&mut self, resources: &mut Resources) -> R {
                // println!("Running system {}", std::any::type_name::<F>());
                self.0($(match $params::get(resources) {
                    Ok(r) => r,
                    Err(error) => panic!("Unable to inject resource into system {}.\n\t{}", std::any::type_name::<F>(), error),
                }),*)
            }
        }

        #[allow(unused_parens)]
        impl<F, R, $($params: SystemParam),*> IntoSystem<($($params),*), R> for F
        where
            for<'r> F: FnMut($($params),*) -> R + FnMut($(<$params as SystemParam>::Item<'r>),*) -> R,
        {
            type System = FunctionSystem<Self, ($($params),*)>;

            fn system(self) -> Self::System {
                FunctionSystem(self, PhantomData)
            }
        }
    };
}

macro_rules! impl_systems {
    ($first_param:ident) => {
        impl_system!($first_param);
    };

    ($first_param:ident, $($params:ident),*) => {
        impl_system!($first_param, $($params),*);
        impl_systems!($($params),*);
    };
}

impl_systems!(A1, A2, A3, A4, A5, A6, A7, A8, A9, A10, A11, A12, A13, A14, A15, A16);

#[cfg(test)]
mod tests {
    use super::*;

    struct Print(String);

    fn system0(mut s: ResMut<String>) {
        *s = format!("{}, world!", s);
    }

    fn system1(s: Res<String>, n: Res<u32>, mut r: ResMut<Print>) {
        r.0 = format!("{} {} {}", s, r.0, n)
    }

    #[test]
    fn test_systems() {
        let mut resources = Resources::default();
        resources.insert(String::from("Hello"));
        resources.insert(0u32);
        resources.insert(Print("empty".to_string()));

        let mut system_runner = SystemRunner::default();
        system_runner.push(system0);
        system_runner.push(system1);
        system_runner.run(&mut resources);

        let print = resources.get::<String>().unwrap();
        let message = print.as_str();
        assert_eq!(message, "Hello, world!");

        let print = resources.get::<Print>().unwrap();
        let message = print.0.as_str();
        assert_eq!(message, "Hello, world! empty 0");
    }
}
