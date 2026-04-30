pub struct EventQueue<T: Copy, const N: usize> {
    slots: [Option<T>; N],
    head: usize,
    len: usize,
    dropped: u32,
}

impl<T: Copy, const N: usize> EventQueue<T, N> {
    pub const fn new() -> Self {
        Self {
            slots: [None; N],
            head: 0,
            len: 0,
            dropped: 0,
        }
    }

    pub const fn len(&self) -> usize {
        self.len
    }

    pub const fn capacity(&self) -> usize {
        N
    }

    pub const fn dropped(&self) -> u32 {
        self.dropped
    }

    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub const fn is_full(&self) -> bool {
        self.len >= N
    }

    pub fn clear(&mut self) {
        while self.pop().is_some() {}
    }

    pub fn reset_dropped(&mut self) {
        self.dropped = 0;
    }

    pub fn push(&mut self, event: T) -> bool {
        if N == 0 || self.len >= N {
            self.dropped = self.dropped.wrapping_add(1);
            return false;
        }

        let tail = (self.head + self.len) % N;
        self.slots[tail] = Some(event);
        self.len += 1;
        true
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 || N == 0 {
            return None;
        }

        let event = self.slots[self.head];
        self.slots[self.head] = None;
        self.head = (self.head + 1) % N;
        self.len -= 1;
        event
    }

    pub fn peek(&self) -> Option<T> {
        if N == 0 || self.len == 0 {
            None
        } else {
            self.slots[self.head]
        }
    }
}
