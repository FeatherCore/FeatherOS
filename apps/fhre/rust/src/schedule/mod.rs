pub type SystemFn<C> = fn(&mut C);

pub struct Schedule<C, const N: usize> {
    systems: [Option<SystemFn<C>>; N],
    len: usize,
    overflowed: bool,
}

impl<C, const N: usize> Schedule<C, N> {
    pub const fn new() -> Self {
        Self {
            systems: [None; N],
            len: 0,
            overflowed: false,
        }
    }

    pub const fn len(&self) -> usize {
        self.len
    }

    pub const fn capacity(&self) -> usize {
        N
    }

    pub const fn overflowed(&self) -> bool {
        self.overflowed
    }

    pub fn clear(&mut self) {
        let mut index = 0;
        while index < N {
            self.systems[index] = None;
            index += 1;
        }
        self.len = 0;
        self.overflowed = false;
    }

    pub fn add_system(&mut self, system: SystemFn<C>) -> bool {
        if self.len >= N {
            self.overflowed = true;
            return false;
        }

        self.systems[self.len] = Some(system);
        self.len += 1;
        true
    }

    pub fn run(&self, context: &mut C) {
        let mut index = 0;
        while index < self.len {
            if let Some(system) = self.systems[index] {
                system(context);
            }
            index += 1;
        }
    }
}
