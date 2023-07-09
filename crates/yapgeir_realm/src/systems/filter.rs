use crate::{IntoSystem, Res, Resources, System};

pub struct FilteredSystem<T, P> {
    system: T,
    predicate: P,
}

impl<T: System<()>, P: System<bool>> System<()> for FilteredSystem<T, P> {
    fn run(&mut self, resources: &'_ mut Resources) -> () {
        if self.predicate.run(resources) {
            self.system.run(resources);
        }
    }
}

pub trait IntoFilteredSystem<Args, R>: IntoSystem<Args, R> {
    fn filter<PredicateArgs, P: System<bool> + 'static>(
        self,
        predicate: impl IntoSystem<PredicateArgs, bool, System = P>,
    ) -> FilteredSystem<Self::System, P>
    where
        Self: Sized,
        <Self as IntoSystem<Args, R>>::System: System<()>,
    {
        let predicate = predicate.system();
        let system = self.system();
        FilteredSystem { system, predicate }
    }
}

impl<Args, R, T> IntoFilteredSystem<Args, R> for T where T: IntoSystem<Args, R> + Sized {}

pub fn resource_exists<R: 'static>() -> impl System<bool> {
    (|r: Option<Res<R>>| r.is_some()).system()
}

#[cfg(test)]
mod tests {
    use super::super::*;
    #[derive(Default)]
    struct MyRes(String);

    fn mut_system(mut s: ResMut<MyRes>) {
        s.0 = format!("Hello, world!{}", &s.0);
    }

    fn filter_system(s: Res<MyRes>) -> bool {
        s.0.is_empty()
    }

    #[test]
    fn test_systems() {
        let mut resources = Resources::default();
        resources.insert(MyRes::default());

        let mut system_runner = SystemRunner::default();
        system_runner.push(mut_system.filter(filter_system));

        // System will run for the first time
        system_runner.run(&mut resources);
        {
            let print = resources.get::<MyRes>().unwrap();
            let message = print.0.as_str();
            assert_eq!(message, "Hello, world!");
        }

        // System will not run the second time
        system_runner.run(&mut resources);
        {
            let print = resources.get::<MyRes>().unwrap();
            let message = print.0.as_str();
            assert_eq!(message, "Hello, world!");
        }
    }
}
