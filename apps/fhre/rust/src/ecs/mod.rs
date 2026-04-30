#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Entity {
    pub index: u16,
    pub generation: u16,
}

impl Entity {
    pub const INVALID: Self = Self {
        index: u16::MAX,
        generation: u16::MAX,
    };

    pub const fn new(index: u16, generation: u16) -> Self {
        Self { index, generation }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct EntitySlot {
    generation: u16,
    alive: bool,
}

impl EntitySlot {
    const EMPTY: Self = Self {
        generation: 0,
        alive: false,
    };
}

pub struct EntityWorld<const N: usize> {
    slots: [EntitySlot; N],
    alive: usize,
}

impl<const N: usize> EntityWorld<N> {
    pub const fn new() -> Self {
        Self {
            slots: [EntitySlot::EMPTY; N],
            alive: 0,
        }
    }

    pub const fn capacity(&self) -> usize {
        N
    }

    pub const fn len(&self) -> usize {
        self.alive
    }

    pub const fn is_empty(&self) -> bool {
        self.alive == 0
    }

    pub fn spawn(&mut self) -> Option<Entity> {
        let mut index = 0;
        while index < N {
            if !self.slots[index].alive {
                self.slots[index].alive = true;
                self.alive += 1;
                return Some(Entity::new(index as u16, self.slots[index].generation));
            }
            index += 1;
        }
        None
    }

    pub fn despawn(&mut self, entity: Entity) -> bool {
        if !self.is_alive(entity) {
            return false;
        }

        let slot = &mut self.slots[entity.index as usize];
        slot.alive = false;
        slot.generation = slot.generation.wrapping_add(1);
        self.alive = self.alive.saturating_sub(1);
        true
    }

    pub fn is_alive(&self, entity: Entity) -> bool {
        let index = entity.index as usize;
        index < N && self.slots[index].alive && self.slots[index].generation == entity.generation
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ComponentEntry<T: Copy> {
    pub entity: Entity,
    pub value: T,
}

pub struct ComponentStorage<T: Copy, const N: usize> {
    entries: [Option<ComponentEntry<T>>; N],
    len: usize,
    overflowed: bool,
}

impl<T: Copy, const N: usize> ComponentStorage<T, N> {
    pub const fn new() -> Self {
        Self {
            entries: [None; N],
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
            self.entries[index] = None;
            index += 1;
        }
        self.len = 0;
        self.overflowed = false;
    }

    pub fn insert(&mut self, entity: Entity, value: T) -> bool {
        if let Some(index) = self.find_index(entity) {
            if let Some(entry) = self.entries[index].as_mut() {
                entry.value = value;
            }
            return true;
        }

        let mut index = 0;
        while index < N {
            if self.entries[index].is_none() {
                self.entries[index] = Some(ComponentEntry { entity, value });
                self.len += 1;
                return true;
            }
            index += 1;
        }

        self.overflowed = true;
        false
    }

    pub fn remove(&mut self, entity: Entity) -> Option<T> {
        let index = self.find_index(entity)?;
        let entry = self.entries[index].take()?;
        self.len = self.len.saturating_sub(1);
        Some(entry.value)
    }

    pub fn contains(&self, entity: Entity) -> bool {
        self.find_index(entity).is_some()
    }

    pub fn get(&self, entity: Entity) -> Option<&T> {
        let index = self.find_index(entity)?;
        self.entries[index].as_ref().map(|entry| &entry.value)
    }

    pub fn get_mut(&mut self, entity: Entity) -> Option<&mut T> {
        let index = self.find_index(entity)?;
        self.entries[index].as_mut().map(|entry| &mut entry.value)
    }

    pub fn retain_alive<const ENTITIES: usize>(&mut self, entities: &EntityWorld<ENTITIES>) {
        let mut index = 0;
        while index < N {
            let remove = match self.entries[index] {
                Some(entry) => !entities.is_alive(entry.entity),
                None => false,
            };
            if remove {
                self.entries[index] = None;
                self.len = self.len.saturating_sub(1);
            }
            index += 1;
        }
    }

    pub fn for_each(&self, mut f: impl FnMut(Entity, &T)) {
        let mut index = 0;
        while index < N {
            if let Some(entry) = self.entries[index].as_ref() {
                f(entry.entity, &entry.value);
            }
            index += 1;
        }
    }

    pub fn for_each_mut(&mut self, mut f: impl FnMut(Entity, &mut T)) {
        let mut index = 0;
        while index < N {
            if let Some(entry) = self.entries[index].as_mut() {
                f(entry.entity, &mut entry.value);
            }
            index += 1;
        }
    }

    fn find_index(&self, entity: Entity) -> Option<usize> {
        let mut index = 0;
        while index < N {
            if matches!(self.entries[index], Some(entry) if entry.entity == entity) {
                return Some(index);
            }
            index += 1;
        }
        None
    }
}

pub struct ResourceSlot<T: Copy> {
    value: Option<T>,
}

impl<T: Copy> ResourceSlot<T> {
    pub const fn new() -> Self {
        Self { value: None }
    }

    pub const fn is_some(&self) -> bool {
        self.value.is_some()
    }

    pub fn insert(&mut self, value: T) {
        self.value = Some(value);
    }

    pub fn remove(&mut self) -> Option<T> {
        self.value.take()
    }

    pub const fn get(&self) -> Option<&T> {
        self.value.as_ref()
    }

    pub fn get_mut(&mut self) -> Option<&mut T> {
        self.value.as_mut()
    }
}
