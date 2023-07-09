use std::marker::PhantomData;

use bitvec::prelude::BitArray;

/// Calculate how many u32 values are needed to store N bits;
pub const fn u32_blocks(bits: usize) -> usize {
    (bits + 32 - 1) / 32
}

pub struct Buttons<const N: usize, B> {
    pub state: BitArray<[u32; N]>,
    pub previous_state: BitArray<[u32; N]>,

    _b: PhantomData<B>,
}

impl<const N: usize, B> Default for Buttons<N, B> {
    fn default() -> Self {
        Self {
            state: Default::default(),
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
        self.previous_state = self.state;
    }

    #[inline]
    pub fn down(&self, code: B) -> bool {
        let code = code.as_usize();
        self.state[code]
    }

    #[inline]
    pub fn up(&self, code: B) -> bool {
        let code = code.as_usize();
        !self.state[code]
    }

    #[inline]
    pub fn pressed(&self, code: B) -> bool {
        let code = code.as_usize();
        self.state[code] && !self.previous_state[code]
    }
}
