use yapgeir_realm::{Realm, Res, ResMut};
use yapgeir_reflection::bevy_reflect::Reflect;
use yapgeir_reflection::{bevy_reflect, RealmExtensions};

use crate::{Delta, Frame};

#[derive(Default, Reflect)]
pub struct FrameStats {
    pub frames: u64,
    pub average_fps: f32,
    fps_cache: u64,
    fps_time: f64,
}

fn update(mut frame: ResMut<FrameStats>, delta: Res<Delta>) {
    frame.fps_cache += 1;
    frame.fps_time += **delta as f64;
    if frame.fps_time >= 1f64 {
        frame.average_fps = (frame.fps_cache as f64 / frame.fps_time) as f32;

        println!(
            "FPS: {}, frames: {}, time: {}, lastDelta: {}",
            frame.average_fps, frame.fps_cache, frame.fps_time, **delta
        );

        frame.fps_cache = 0;
        frame.fps_time = 0f64;
    }

    frame.frames += 1;
}

pub fn plugin(realm: &mut Realm) {
    realm
        .register_type::<Delta>()
        .register_type::<Frame>()
        .register_type::<FrameStats>()
        .initialize_resource::<Delta>()
        .initialize_resource::<Frame>()
        .initialize_resource::<FrameStats>()
        .add_system(update);
}
