use std::marker::PhantomData;

use crate::{IntoSystem, Resources, System};

pub struct ResultSystem<S, R, E, H: Fn(E) -> R> {
    system: S,
    handler: H,

    _e: PhantomData<E>,
}

impl<S, R, E, H> System<R> for ResultSystem<S, R, E, H>
where
    S: System<Result<R, E>>,
    H: Fn(E) -> R,
{
    fn run(&mut self, resources: &'_ mut Resources) -> R {
        let result = self.system.run(resources);

        match result {
            Ok(r) => r,
            Err(e) => (self.handler)(e),
        }
    }
}

pub trait IntoResultSystem<Args, R, E>: IntoSystem<Args, Result<R, E>> {
    fn on_error<H: Fn(E) -> R>(self, handler: H) -> ResultSystem<Self::System, R, E, H>
    where
        Self: Sized,
        <Self as IntoSystem<Args, Result<R, E>>>::System: System<Result<R, E>>,
    {
        let system = self.system();
        ResultSystem {
            system,
            handler,
            _e: PhantomData,
        }
    }
}

impl<Args, R, E, S> IntoResultSystem<Args, R, E> for S where
    S: IntoSystem<Args, Result<R, E>> + Sized
{
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use super::super::*;

    fn error_system(_: &mut Resources) -> Result<(), i32> {
        Err(-1)
    }

    #[test]
    fn test_error_handler() {
        let mut resources = Resources::default();

        let result: Rc<RefCell<i32>> = Default::default();

        let mut system_runner = SystemRunner::default();
        system_runner.push(error_system.on_error({
            let result = result.clone();
            move |e| {
                let mut result = result.borrow_mut();
                *result = e;
            }
        }));
        system_runner.run(&mut resources);

        assert_eq!(*result.borrow(), -1);
    }
}
