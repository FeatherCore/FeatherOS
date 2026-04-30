use crate::ContentInset;
use fhre::{Color, Rect};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct LumiaMetrics {
    pub(crate) status_h: u16,
    pub(crate) nav_h: u16,
    pub(crate) pad_x: u16,
    pub(crate) pad_top: u16,
    pub(crate) pad_bottom: u16,
    pub(crate) gap: u16,
    pub(crate) tile: u16,
    pub(crate) label_scale: u16,
    pub(crate) small_scale: u16,
}

impl LumiaMetrics {
    pub(crate) fn for_screen(width: u16, height: u16) -> Self {
        let short = width.min(height).max(1);
        let status_h = scale_metric(short, 20).max(18);
        let nav_h = scale_metric(short, 40).max(30);
        let pad_x = scale_metric(short, 5).max(5);
        let pad_top = status_h.saturating_add(scale_metric(short, 5).max(4));
        let pad_bottom = nav_h.saturating_add(scale_metric(short, 5).max(4));
        let gap = scale_metric(short, 5).max(4);
        let content_w = width.saturating_sub(pad_x.saturating_mul(2));
        let content_h = height.saturating_sub(pad_top.saturating_add(pad_bottom));
        let tile_by_w = content_w.saturating_sub(gap.saturating_mul(2)) / 3;
        let tile_by_h = content_h.saturating_sub(gap.saturating_mul(3)) / 4;
        let tile = tile_by_w.min(tile_by_h).max(scale_metric(short, 52).max(44));
        let label_scale = if short >= 700 {
            3
        } else if short >= 430 {
            2
        } else {
            1
        };
        let small_scale = if short >= 760 { 2 } else { 1 };

        Self {
            status_h,
            nav_h,
            pad_x,
            pad_top,
            pad_bottom,
            gap,
            tile,
            label_scale,
            small_scale,
        }
    }

    pub(crate) fn content_rect(self, width: u16, height: u16) -> Rect {
        Rect::new(0, 0, width, height.saturating_sub(self.nav_h))
    }

    pub(crate) fn start_inset(self) -> ContentInset {
        ContentInset::new(self.pad_x, self.pad_top, self.pad_x, self.pad_bottom)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct LumiaTheme {
    pub(crate) bg: Color,
    pub(crate) panel: Color,
    pub(crate) panel_alt: Color,
    pub(crate) accent: Color,
    pub(crate) accent_alt: Color,
    pub(crate) tile: Color,
    pub(crate) text: Color,
    pub(crate) muted: Color,
    pub(crate) nav: Color,
}

impl LumiaTheme {
    pub(crate) const DARK: Self = Self {
        bg: Color::rgba(0, 0, 0, 120),
        panel: Color::rgba(0, 0, 0, 108),
        panel_alt: Color::rgba(10, 18, 24, 192),
        accent: Color::rgb(0, 120, 215),
        accent_alt: Color::rgb(0, 188, 242),
        tile: Color::rgba(0, 120, 215, 214),
        text: Color::rgba(248, 250, 255, 238),
        muted: Color::rgba(214, 226, 238, 176),
        nav: Color::rgba(0, 0, 0, 178),
    };
}

pub(crate) fn scale_metric(short: u16, value: u16) -> u16 {
    let scaled = (short as u32).saturating_mul(value as u32) / 320;
    scaled.min(u16::MAX as u32) as u16
}
