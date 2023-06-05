use core::mem::MaybeUninit;
use core::option::Option;

pub struct CircularBuffer<T, const N: usize> {
    arr: [MaybeUninit<T>; N],
    r_index: usize,
    w_index: usize,
}

impl<T, const N: usize> CircularBuffer<T, N> {
    pub const fn new() -> Self {
        Self {
            arr: unsafe { MaybeUninit::uninit().assume_init() },
            r_index: 0,
            w_index: 0,
        }
    }
    pub fn is_empty(&self) -> bool {
        self.r_index == self.w_index
    }
    pub fn is_full(&self) -> bool {
        self.r_index + N == self.w_index
    }
    pub fn read(&mut self) -> Option<T> {
        if self.is_empty() {
            None
        } else {
            let value = unsafe { self.arr[self.r_index % N].assume_init_read() };
            self.r_index += 1;
            Some(value)
        }
    }
    pub fn write(&mut self, value: T) -> Option<()> {
        if self.is_full() {
            None
        } else {
            self.arr[self.w_index % N].write(value);
            self.w_index += 1;
            Some(())
        }
    }
}
