use std::marker::PhantomData;

use bitvec::prelude::BitArray;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ButtonAction {
    Up,
    Down,
}

/// Calculate how many u32 values are needed to store N bits;
pub const fn u32_blocks(bits: usize) -> usize {
    (bits + 32 - 1) / 32
}

pub struct Buttons<const N: usize, B> {
    pub pressed: BitArray<[u32; N]>,
    pub current_state: BitArray<[u32; N]>,
    pub previous_state: BitArray<[u32; N]>,

    _b: PhantomData<B>,
}

impl<const N: usize, B> Default for Buttons<N, B> {
    fn default() -> Self {
        Self {
            pressed: Default::default(),
            current_state: Default::default(),
            previous_state: Default::default(),
            _b: PhantomData,
        }
    }
}

pub trait CastToUsize {
    fn as_usize(self) -> usize;
}

impl<const N: usize, B: CastToUsize> Buttons<N, B> {
    #[inline]
    pub(crate) fn flush(&mut self) {
        self.pressed = Default::default();
        self.previous_state = self.current_state;
    }

    #[inline]
    pub fn down(&self, code: B) -> bool {
        let code = code.as_usize();
        self.current_state[code]
    }

    #[inline]
    pub fn up(&self, code: B) -> bool {
        let code = code.as_usize();
        !self.current_state[code]
    }

    #[inline]
    pub fn just_pressed(&self, code: B) -> bool {
        let code = code.as_usize();
        self.pressed[code]
    }
}
