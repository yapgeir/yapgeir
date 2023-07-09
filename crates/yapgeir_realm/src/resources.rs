use std::{
    any::{Any, TypeId},
    cell::{Ref, RefCell, RefMut},
    collections::HashMap,
};

#[derive(Default)]
pub struct Resources {
    resources: HashMap<TypeId, RefCell<Box<dyn Any>>>,
}

impl Resources {
    pub fn insert<T: 'static>(&mut self, resource: T) {
        self.resources
            .insert(TypeId::of::<T>(), RefCell::new(Box::new(resource)));
    }

    pub fn contains<T: 'static>(&self) -> bool {
        self.resources.contains_key(&TypeId::of::<T>())
    }

    pub fn get<T: 'static>(&self) -> Option<Ref<T>> {
        self.resources
            .get(&TypeId::of::<T>())
            .map(|res| res.borrow())
            .map(|res| Ref::map(res, |r| r.downcast_ref::<T>().expect("Downcast failed")))
    }

    pub fn get_mut<T: 'static>(&self) -> Option<RefMut<T>> {
        self.resources
            .get(&TypeId::of::<T>())
            .map(|res| res.borrow_mut())
            .map(|res| RefMut::map(res, |r| r.downcast_mut::<T>().expect("Downcast failed")))
    }

    pub fn remove<T: 'static>(&mut self) -> Option<T> {
        self.resources
            .remove(&TypeId::of::<T>())
            .map(|res| res.into_inner())
            .map(|res| *res.downcast::<T>().expect("Downcast failed"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Print(String);

    #[test]
    fn test_print_system() {
        let mut resources = Resources::default();
        resources.insert(String::from("Hello, world!"));
        resources.insert(0u32);
        resources.insert(Print("none".to_string()));

        fn run(resources: &Resources) {
            let message = resources.get_mut::<String>().unwrap();
            let counter = resources.get_mut::<u32>().unwrap();
            let mut print = resources.get_mut::<Print>().unwrap();

            print.0 = format!("{} {}", message, counter);
        }

        run(&resources);

        let print = resources.get::<Print>().unwrap();
        let message = print.0.as_str();
        assert_eq!(message, "Hello, world! 0");
    }

    #[test]
    fn test_increment_system() {
        let mut resources = Resources::default();
        resources.insert(0u32);
        resources.insert(15i32);

        fn run(resources: &Resources) {
            let value = resources.get::<i32>().unwrap();
            let mut counter = resources.get_mut::<u32>().unwrap();
            *counter += *value as u32;
        }

        run(&mut resources);

        // Check if the system ran successfully by verifying the counter is incremented
        let number = *resources.get::<u32>().unwrap();
        assert_eq!(number, 15);
    }
}
