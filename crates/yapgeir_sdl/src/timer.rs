use yapgeir_core::{Delta, Frame};
use yapgeir_realm::{Realm, Res, ResMut};

struct Timer {
    timer: sdl2::TimerSubsystem,
    previous_counter: u64,
}

impl Timer {
    pub fn new(timer: sdl2::TimerSubsystem) -> Self {
        Self {
            timer,
            previous_counter: 0,
        }
    }
}

fn update(mut timer: ResMut<Timer>, mut delta: ResMut<Delta>, mut frame: ResMut<Frame>) {
    let counter = timer.timer.performance_counter();
    let freq = timer.timer.performance_frequency();

    if timer.previous_counter == u64::default() {
        timer.previous_counter = counter - 1;
    }

    frame.0 += 1;
    delta.0 = ((counter - timer.previous_counter) as f32 / (freq as f32)).min(1f32);
    timer.previous_counter = counter;
}

pub fn plugin(realm: &mut Realm) {
    realm
        .initialize_resource::<Delta>()
        .initialize_resource::<Frame>()
        .initialize_resource_with(|sdl: Res<sdl2::Sdl>| {
            let timer = sdl.timer().expect("Unable to get sdl timer");
            Timer::new(timer)
        })
        .add_system(update);
}
