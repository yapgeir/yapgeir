use by_address::ByAddress;
use indexmap::IndexMap;
use std::time::{Duration, Instant};
use yapgeir_core::Frame;
use yapgeir_realm::{Realm, Res, ResMut};

pub use yapgeir_instrument_macro::instrument;

#[cfg(feature = "allocations")]
mod allocator;

#[derive(Default, Debug)]
pub struct Values {
    pub invocations: u64,
    pub allocations: u64,
    pub duration: Duration,
}

#[derive(Default, Debug)]
pub struct System {
    last_frame: u64,
    pub total: Values,
    pub current_frame: Values,
}

#[derive(Default, Debug)]
pub struct Instrumentation {
    pub frame: u64,
    pub data: IndexMap<ByAddress<&'static str>, System>,
}

pub struct InstrumentationGuard<'a> {
    time: Instant,
    frame: u64,
    system: &'a mut System,
    #[cfg(feature = "allocations")]
    allocations: allocator::Counter,
}

impl<'a> Drop for InstrumentationGuard<'a> {
    fn drop(&mut self) {
        let duration = self.time.elapsed();

        if self.frame != self.system.last_frame {
            self.system.last_frame = self.frame;
            self.system.current_frame = Values::default();
        }

        self.system.current_frame.invocations += 1;
        self.system.current_frame.duration += duration;

        self.system.total.invocations += 1;
        self.system.total.duration += duration;

        #[cfg(feature = "allocations")]
        {
            let allocations = self.allocations.count();
            self.system.current_frame.allocations += allocations;
            self.system.total.allocations += allocations;
        }
    }
}

impl Instrumentation {
    pub fn guard<'a>(&'a mut self, system: &'static str) -> InstrumentationGuard<'a> {
        let system = self.data.entry(ByAddress(system)).or_default();

        InstrumentationGuard {
            time: Instant::now(),
            frame: self.frame,
            system,
            #[cfg(feature = "allocations")]
            allocations: allocator::CountingAllocator::counter(),
        }
    }
}

pub fn update(mut instrumentation: ResMut<Instrumentation>, frame: Res<Frame>) {
    instrumentation.frame = **frame;
}

pub fn plugin(realm: &mut Realm) {
    realm
        .initialize_resource::<Instrumentation>()
        .add_system(update);
}
