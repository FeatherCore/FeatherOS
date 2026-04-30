use crate::Rect;

pub struct DirtyRegion<const N: usize> {
    rects: [Rect; N],
    len: usize,
    overflowed: bool,
}

impl<const N: usize> DirtyRegion<N> {
    pub const fn new() -> Self {
        Self {
            rects: [Rect::EMPTY; N],
            len: 0,
            overflowed: false,
        }
    }

    pub const fn len(&self) -> usize {
        self.len
    }

    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub const fn overflowed(&self) -> bool {
        self.overflowed
    }

    pub fn clear(&mut self) {
        self.len = 0;
        self.overflowed = false;
    }

    pub fn full_redraw(&mut self, screen: Rect) {
        self.clear();
        if N > 0 {
            self.rects[0] = screen;
            self.len = 1;
        }
        self.overflowed = true;
    }

    pub fn push(&mut self, rect: Rect, screen: Rect) {
        let rect = rect.clipped_to(screen);
        if rect.is_empty() {
            return;
        }

        let mut i = 0;
        while i < self.len {
            if self.rects[i].intersects(rect) {
                self.rects[i] = self.rects[i].union(rect).clipped_to(screen);
                return;
            }
            i += 1;
        }

        if self.len < N {
            self.rects[self.len] = rect;
            self.len += 1;
            return;
        }

        self.overflowed = true;
        if N > 0 {
            let mut merged = rect;
            let mut j = 0;
            while j < self.len {
                merged = merged.union(self.rects[j]);
                j += 1;
            }
            self.rects[0] = merged.clipped_to(screen);
            self.len = 1;
        }
    }

    pub fn touches(&self, rect: Rect) -> bool {
        if self.len == 0 {
            return false;
        }

        let mut i = 0;
        while i < self.len {
            if self.rects[i].intersects(rect) {
                return true;
            }
            i += 1;
        }
        false
    }

    pub fn rect(&self, index: usize) -> Option<Rect> {
        if index < self.len {
            Some(self.rects[index])
        } else {
            None
        }
    }

    pub fn union_rect(&self) -> Option<Rect> {
        if self.len == 0 {
            return None;
        }

        let mut rect = self.rects[0];
        let mut index = 1;
        while index < self.len {
            rect = rect.union(self.rects[index]);
            index += 1;
        }
        Some(rect)
    }

    pub fn copy_into<const M: usize>(&self, out: &mut DirtyRegion<M>, screen: Rect) {
        let mut index = 0;
        while index < self.len {
            out.push(self.rects[index], screen);
            index += 1;
        }
        if self.overflowed {
            out.full_redraw(screen);
        }
    }
}

pub struct DirtyTracker<const N: usize> {
    screen: Rect,
    dirty: DirtyRegion<N>,
    first_frame: bool,
}

impl<const N: usize> DirtyTracker<N> {
    pub const fn new(screen: Rect) -> Self {
        Self {
            screen,
            dirty: DirtyRegion::new(),
            first_frame: true,
        }
    }

    pub fn set_screen(&mut self, screen: Rect) {
        if self.screen != screen {
            self.screen = screen;
            self.first_frame = true;
        }
    }

    pub fn begin_frame(&mut self, screen: Rect) {
        self.set_screen(screen);
        self.dirty.clear();
        if self.first_frame {
            self.dirty.full_redraw(self.screen);
        }
    }

    pub fn begin_frame_current_screen(&mut self) {
        self.begin_frame(self.screen);
    }

    pub fn mark_current(&mut self, current: Rect) {
        self.dirty.push(current, self.screen);
    }

    pub fn mark_transition(&mut self, previous: Option<Rect>, current: Rect) {
        if let Some(previous) = previous {
            self.dirty.push(previous, self.screen);
        }
        self.dirty.push(current, self.screen);
    }

    pub fn force_full(&mut self) {
        self.dirty.full_redraw(self.screen);
    }

    pub fn full_redraw(&mut self) {
        self.force_full();
    }

    pub const fn region(&self) -> &DirtyRegion<N> {
        &self.dirty
    }

    pub const fn dirty(&self) -> &DirtyRegion<N> {
        self.region()
    }

    pub fn finish_frame(&mut self) {
        self.first_frame = false;
    }
}
