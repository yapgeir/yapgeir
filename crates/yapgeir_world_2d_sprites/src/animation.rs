use std::{collections::HashMap, mem, ops::Index};

use derive_more::Constructor;
use hecs::{Entity, Without, World};
use yapgeir_assets::animations::{Animation, AnimationKind, AnimationSequence};
use yapgeir_collections::{PersistentSlotMap, Slot};
use yapgeir_core::Delta;
use yapgeir_realm::{system, Realm, Res, ResMut};
use yapgeir_world_2d::Drawable;

#[cfg(feature = "reflection")]
use yapgeir_reflection::{
    bevy_reflect::{self, Reflect},
    RealmExtensions,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Constructor, Hash)]
#[cfg_attr(feature = "reflection", derive(Reflect))]
pub struct AnimationSequenceKey(u16);

impl From<Slot> for AnimationSequenceKey {
    fn from(slot: Slot) -> Self {
        Self(slot.0 as u16)
    }
}

impl Into<Slot> for AnimationSequenceKey {
    fn into(self) -> Slot {
        Slot(self.0 as usize)
    }
}

// This is they key that should be used to access entities
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "reflection", derive(Reflect))]
pub struct AnimationKey(AnimationSequenceKey, u8);

#[derive(Default, Debug)]
pub struct AnimationStorage(pub PersistentSlotMap<String, AnimationSequence>);

impl AnimationStorage {
    #[inline]
    fn is_last_in_sequence(&self, index: AnimationKey) -> bool {
        let key = index.0;
        self.0[Into::<Slot>::into(key)].len() <= index.1 as usize + 1
    }

    pub fn merge(&mut self, map: HashMap<String, AnimationSequence>) {
        for (key, value) in map {
            self.0.insert(key, value);
        }
    }

    pub fn insert(
        &mut self,
        key: impl Into<String>,
        sequence: AnimationSequence,
    ) -> AnimationSequenceKey {
        let slot = self.0.insert(key.into(), sequence);
        slot.into()
    }

    pub fn find_key(&self, key: &str) -> Option<AnimationSequenceKey> {
        self.0.find_slot_by_key(key).map(|slot| slot.into())
    }
}

impl Index<AnimationKey> for AnimationStorage {
    type Output = Animation;

    fn index(&self, key: AnimationKey) -> &Self::Output {
        &self.0[Into::<Slot>::into(key.0)][key.1 as usize]
    }
}

impl Index<AnimationSequenceKey> for AnimationStorage {
    type Output = AnimationSequence;

    fn index(&self, key: AnimationSequenceKey) -> &Self::Output {
        &self.0[Into::<Slot>::into(key)]
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "reflection", derive(Reflect))]
pub struct Frame {
    pub index: u8,
    reversed: bool,
}

fn next_frame(animation: &Animation, frame: Frame) -> Frame {
    let is_last_frame = animation.is_last_frame(frame.index);

    match animation.kind {
        AnimationKind::Loop => Frame {
            index: match is_last_frame {
                true => 0,
                false => frame.index + 1,
            },
            reversed: false,
        },
        AnimationKind::PingPong => {
            let reversed = is_last_frame || (frame.reversed && frame.index > 0);
            Frame {
                index: match reversed {
                    true => frame.index - 1,
                    false => frame.index + 1,
                },
                reversed,
            }
        }
        AnimationKind::Single => Frame {
            index: frame.index + 1,
            reversed: false,
        },
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "reflection", derive(Reflect))]
pub enum FrameState {
    Started,
    Ended,
    Frame(Frame),
}

/// A component that will drive drawable change on an entity
#[derive(Debug, Clone)]
#[cfg_attr(feature = "reflection", derive(Reflect))]
pub struct Animator {
    animation: AnimationKey,
    next_sequence: Option<AnimationSequenceKey>,
    elapsed: f32,
    frame: FrameState,
}

impl Animator {
    pub fn new(sequence: AnimationSequenceKey) -> Self {
        Self {
            animation: AnimationKey(sequence, 0),
            next_sequence: None,
            elapsed: 0.,
            frame: FrameState::Started,
        }
    }

    pub fn play_deferred(&mut self, sequence: AnimationSequenceKey) {
        if self.animation.0 != sequence {
            self.next_sequence = Some(sequence);
        }
    }

    pub fn play_now(&mut self, sequence: AnimationSequenceKey) {
        if self.animation.0 != sequence {
            self.animation = AnimationKey(sequence, 0);
            self.frame = FrameState::Started;
            self.elapsed = 0.;
        }
    }
}

#[derive(Default)]
struct DrawableAdder(Vec<(Entity, Drawable)>);

#[system]
impl DrawableAdder {
    fn update(&mut self, mut world: ResMut<World>, store: Res<AnimationStorage>) {
        self.0.clear();
        for (e, a) in world.query::<Without<&Animator, &Drawable>>().iter() {
            let drawable = store[a.animation].frames[0].clone();
            self.0.push((e, drawable));
        }

        for (e, drawable) in &self.0 {
            world
                .insert_one(e.clone(), drawable.clone())
                .expect("Unable to insert Drawable for entity");
        }
    }
}

fn update(mut world: ResMut<World>, store: Res<AnimationStorage>, delta: Res<Delta>) {
    for (_, (a, drawable)) in world.query_mut::<(&mut Animator, &mut Drawable)>() {
        let frame = match (a.frame, mem::take(&mut a.next_sequence)) {
            (FrameState::Ended, None) => {
                continue;
            }
            (FrameState::Started, None) => Frame::default(),
            (FrameState::Started, Some(next)) | (FrameState::Ended, Some(next)) => {
                a.play_now(next);
                Frame::default()
            }
            (FrameState::Frame(frame), next) => {
                let animation = &store[a.animation];

                a.elapsed += **delta;
                if a.elapsed < animation.frame_time {
                    // Put it back for now
                    a.next_sequence = next;
                    continue;
                }

                a.elapsed -= animation.frame_time;

                match next {
                    Some(next) => {
                        a.play_now(next);
                        Frame::default()
                    }
                    None if animation.is_end(frame.index) => {
                        match store.is_last_in_sequence(a.animation) {
                            true => {
                                a.frame = FrameState::Ended;
                                continue;
                            }
                            false => {
                                a.frame = FrameState::Started;
                                a.animation.1 += 1;
                                Frame::default()
                            }
                        }
                    }
                    None => next_frame(animation, frame),
                }
            }
        };

        a.frame = FrameState::Frame(frame);

        let animation = &store[a.animation];
        *drawable = animation.frames[frame.index as usize].clone();
    }
}

pub fn plugin(realm: &mut Realm) {
    #[cfg(feature = "reflection")]
    realm
        .register_type::<Frame>()
        .register_non_default_type::<Animator>();

    realm
        .add_resource(AnimationStorage::default())
        .add_system(DrawableAdder::default())
        .add_system(update);
}
