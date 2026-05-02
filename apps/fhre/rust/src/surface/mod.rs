use crate::{
    backend::{BackendCapabilities, RenderBackend},
    raster::clamp_i32,
    Color, DrawCommand, GlyphIdResolver, GlyphResolver, GlyphRunResolver, ImageResolver,
    KerningResolver, LayerBudget, LayerSpec, MaskSpec, PixelFormat, Point, Rect, RenderStats,
    SvgResolver,
};

const SURFACE_MASK_STACK: usize = 4;

#[derive(Clone, Copy, Debug)]
pub struct Surface {
    pub(crate) pixels: *mut u8,
    pub(crate) width: u16,
    pub(crate) height: u16,
    pub(crate) stride: usize,
    pub(crate) bpp: u8,
    pub(crate) format: PixelFormat,
    pub(crate) clip: Option<Rect>,
    pub(crate) image_resolver: Option<ImageResolver>,
    pub(crate) glyph_resolver: Option<GlyphResolver>,
    pub(crate) glyph_id_resolver: Option<GlyphIdResolver>,
    pub(crate) glyph_run_resolver: Option<GlyphRunResolver>,
    pub(crate) kerning_resolver: Option<KerningResolver>,
    pub(crate) svg_resolver: Option<SvgResolver>,
    pub(crate) layer_budget: LayerBudget,
    pub(crate) mask_stack: [Option<MaskSpec>; SURFACE_MASK_STACK],
    pub(crate) mask_len: usize,
    pub(crate) mask_overflowed: bool,
}

impl Surface {
    pub const unsafe fn from_raw(
        pixels: *mut u8,
        width: u16,
        height: u16,
        stride: usize,
        bpp: u8,
        nuttx_format: u8,
    ) -> Self {
        Self {
            pixels,
            width,
            height,
            stride,
            bpp,
            format: PixelFormat::from_nuttx(nuttx_format, bpp),
            clip: None,
            image_resolver: None,
            glyph_resolver: None,
            glyph_id_resolver: None,
            glyph_run_resolver: None,
            kerning_resolver: None,
            svg_resolver: None,
            layer_budget: LayerBudget::DEFAULT,
            mask_stack: [None; SURFACE_MASK_STACK],
            mask_len: 0,
            mask_overflowed: false,
        }
    }

    pub const fn width(&self) -> u16 {
        self.width
    }

    pub const fn height(&self) -> u16 {
        self.height
    }

    pub const fn bpp(&self) -> u8 {
        self.bpp
    }

    pub const fn is_valid(&self) -> bool {
        !self.pixels.is_null() && self.width > 0 && self.height > 0 && self.stride > 0
    }

    pub const fn clip(&self) -> Option<Rect> {
        self.clip
    }

    pub fn set_image_resolver(&mut self, resolver: Option<ImageResolver>) {
        self.image_resolver = resolver;
    }

    pub fn set_glyph_resolver(&mut self, resolver: Option<GlyphResolver>) {
        self.glyph_resolver = resolver;
    }

    pub fn set_glyph_id_resolver(&mut self, resolver: Option<GlyphIdResolver>) {
        self.glyph_id_resolver = resolver;
    }

    pub fn set_glyph_run_resolver(&mut self, resolver: Option<GlyphRunResolver>) {
        self.glyph_run_resolver = resolver;
    }

    pub fn set_kerning_resolver(&mut self, resolver: Option<KerningResolver>) {
        self.kerning_resolver = resolver;
    }

    pub fn set_svg_resolver(&mut self, resolver: Option<SvgResolver>) {
        self.svg_resolver = resolver;
    }

    pub fn set_layer_budget(&mut self, budget: LayerBudget) {
        self.layer_budget = budget;
    }

    pub const fn layer_budget(&self) -> LayerBudget {
        self.layer_budget
    }

    pub fn set_clip(&mut self, clip: Option<Rect>) {
        let screen = Rect::new(0, 0, self.width, self.height);
        self.clip = match clip {
            Some(rect) => Some(rect.clipped_to(screen)),
            None => None,
        };
    }

    pub fn clear_clip(&mut self) {
        self.clip = None;
    }

    pub(crate) const fn has_masks(&self) -> bool {
        self.mask_len != 0
    }

    pub fn push_mask(&mut self, spec: MaskSpec) -> bool {
        if self.mask_len >= SURFACE_MASK_STACK {
            self.mask_overflowed = true;
            return false;
        }
        self.mask_stack[self.mask_len] = Some(spec);
        self.mask_len += 1;
        true
    }

    pub fn pop_mask(&mut self) -> bool {
        if self.mask_len == 0 {
            return false;
        }
        self.mask_len -= 1;
        self.mask_stack[self.mask_len] = None;
        true
    }

    pub fn clear_masks(&mut self) {
        self.mask_len = 0;
        self.mask_overflowed = false;
        let mut i = 0;
        while i < SURFACE_MASK_STACK {
            self.mask_stack[i] = None;
            i += 1;
        }
    }

    pub fn clear(&mut self, color: Color) {
        self.fill_rect(Rect::new(0, 0, self.width, self.height), color);
    }

    pub fn put_pixel(&mut self, x: i32, y: i32, color: Color) {
        if !self.is_valid() || x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return;
        }
        if let Some(color) = self.apply_masks_to_color(Point::new(x, y), color) {
            unsafe {
                let offset = y as usize * self.stride + x as usize * (self.bpp as usize / 8);
                let ptr = self.pixels.add(offset);
                let out = color.over(self.read_pixel_ptr(ptr));
                self.write_pixel_ptr(ptr, out);
            }
        }
    }

    pub fn fill_rect(&mut self, rect: Rect, color: Color) {
        if !self.is_valid() || rect.w == 0 || rect.h == 0 {
            return;
        }

        let rect = match self.clip {
            Some(clip) => rect.clipped_to(clip),
            None => rect,
        };

        let x0 = clamp_i32(rect.x, 0, self.width as i32);
        let y0 = clamp_i32(rect.y, 0, self.height as i32);
        let x1 = clamp_i32(rect.right(), 0, self.width as i32);
        let y1 = clamp_i32(rect.bottom(), 0, self.height as i32);

        if x0 >= x1 || y0 >= y1 {
            return;
        }

        if !self.has_masks() {
            if color.a == 255 && self.fill_rect_opaque_fast(x0, y0, x1, y1, color) {
                return;
            }
            if self.fill_rect_alpha_fast(x0, y0, x1, y1, color) {
                return;
            }
        }

        let old_clip = self.clip;
        self.clip = None;
        for y in y0..y1 {
            for x in x0..x1 {
                self.put_pixel(x, y, color);
            }
        }
        self.clip = old_clip;
    }

    pub fn fill_vgradient(&mut self, rect: Rect, top: Color, bottom: Color) {
        if !self.is_valid() || rect.is_empty() {
            return;
        }

        let clipped = match self.clip {
            Some(clip) => rect.clipped_to(clip),
            None => rect,
        };
        let x0 = clamp_i32(clipped.x, 0, self.width as i32);
        let y0 = clamp_i32(clipped.y, 0, self.height as i32);
        let x1 = clamp_i32(clipped.right(), 0, self.width as i32);
        let y1 = clamp_i32(clipped.bottom(), 0, self.height as i32);
        if x0 >= x1 || y0 >= y1 {
            return;
        }

        let h = rect.h.max(1) as i32;
        let has_masks = self.has_masks();
        let old_clip = self.clip;
        self.clip = None;

        let mut y = y0;
        while y < y1 {
            let row = y.saturating_sub(rect.y).clamp(0, h.saturating_sub(1));
            let t = ((row * 255) / h) as u8;
            let color = top.mix(bottom, t);
            if !has_masks && color.a == 255 && self.fill_rect_opaque_fast(x0, y, x1, y + 1, color) {
                y += 1;
                continue;
            }
            if !has_masks && self.fill_rect_alpha_fast(x0, y, x1, y + 1, color) {
                y += 1;
                continue;
            }
            let mut x = x0;
            while x < x1 {
                self.put_pixel(x, y, color);
                x += 1;
            }
            y += 1;
        }

        self.clip = old_clip;
    }
}

impl RenderBackend for Surface {
    fn capabilities(&self) -> BackendCapabilities {
        BackendCapabilities::software(self.format)
    }

    fn set_clip(&mut self, clip: Option<Rect>) {
        Surface::set_clip(self, clip);
    }

    fn draw_command(&mut self, cmd: DrawCommand) {
        cmd.execute(self);
    }

    fn push_mask(&mut self, spec: MaskSpec) -> bool {
        Surface::push_mask(self, spec)
    }

    fn pop_mask(&mut self) -> bool {
        Surface::pop_mask(self)
    }

    fn clear_masks(&mut self) {
        Surface::clear_masks(self);
    }

    fn draw_layer_commands(
        &mut self,
        rect: Rect,
        spec: LayerSpec,
        cmds: &[DrawCommand],
        clips: &[Option<Rect>],
        stats: &mut RenderStats,
    ) -> bool {
        self.draw_layer_commands_impl(rect, spec, cmds, clips, stats)
    }
}
