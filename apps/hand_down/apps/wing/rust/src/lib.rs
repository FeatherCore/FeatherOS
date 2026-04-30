#![no_std]
#![no_main]

use core::ffi::{c_int, c_void};
use core::panic::PanicInfo;

extern "C" {
    fn open(path: *const u8, flags: c_int, ...) -> c_int;
    fn close(fd: c_int) -> c_int;
    fn ioctl(fd: c_int, request: c_int, ...) -> c_int;
    fn usleep(usec: u32) -> c_int;
    fn sim_x11events();
}

const O_RDWR: c_int = 3;
const FBIOGET_VIDEOINFO: c_int = 0x2801;
const FBIOGET_PLANEINFO: c_int = 0x2802;
const FB_FMT_RGB16_565: u8 = 11;

const FRAME_US: u32 = 16_666;

#[repr(C)]
struct VideoInfo {
    fmt: u8,
    xres: u16,
    yres: u16,
    nplanes: u8,
}

#[repr(C)]
struct PlaneInfo {
    fbmem: *mut u8,
    fblen: usize,
    stride: u16,
    display: u8,
    bpp: u8,
    xres_virtual: u32,
    yres_virtual: u32,
    xoffset: u32,
    yoffset: u32,
}

#[derive(Clone, Copy)]
struct Color {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

impl Color {
    const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    fn mix(self, other: Self, t: u8) -> Self {
        let t = t as u16;
        let inv = 255u16.saturating_sub(t);
        Self::rgb(
            (((self.r as u16 * inv) + (other.r as u16 * t)) / 255) as u8,
            (((self.g as u16 * inv) + (other.g as u16 * t)) / 255) as u8,
            (((self.b as u16 * inv) + (other.b as u16 * t)) / 255) as u8,
        )
    }
}

#[derive(Clone, Copy)]
struct Rect {
    x: i32,
    y: i32,
    w: u16,
    h: u16,
}

impl Rect {
    const fn new(x: i32, y: i32, w: u16, h: u16) -> Self {
        Self { x, y, w, h }
    }
}

struct Framebuffer {
    fd: c_int,
    ptr: *mut u8,
    width: u16,
    height: u16,
    stride: usize,
    bpp: u8,
    fmt: u8,
}

impl Framebuffer {
    fn open() -> Option<Self> {
        unsafe {
            let fd = open(b"/dev/fb0\0".as_ptr(), O_RDWR);
            if fd < 0 {
                return None;
            }

            let mut vinfo: VideoInfo = core::mem::zeroed();
            if ioctl(fd, FBIOGET_VIDEOINFO, &mut vinfo as *mut VideoInfo) < 0 {
                close(fd);
                return None;
            }

            let mut pinfo: PlaneInfo = core::mem::zeroed();
            if ioctl(fd, FBIOGET_PLANEINFO, &mut pinfo as *mut PlaneInfo) < 0 {
                close(fd);
                return None;
            }

            if pinfo.fbmem.is_null() || pinfo.stride == 0 {
                close(fd);
                return None;
            }

            Some(Self {
                fd,
                ptr: pinfo.fbmem,
                width: vinfo.xres,
                height: vinfo.yres,
                stride: pinfo.stride as usize,
                bpp: pinfo.bpp,
                fmt: vinfo.fmt,
            })
        }
    }

    fn draw_demo(&mut self, tick: u16) {
        draw_wallpaper(self, tick);
        draw_shell(self, tick);
    }
}

impl Drop for Framebuffer {
    fn drop(&mut self) {
        unsafe {
            if self.fd >= 0 {
                let _ = close(self.fd);
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn wing_rust_main(_argc: i32, _argv: *mut *mut u8) -> i32 {
    let Some(mut fb) = Framebuffer::open() else {
        return 1;
    };

    let mut tick = 0u16;
    loop {
        unsafe {
            sim_x11events();
        }
        fb.draw_demo(tick);
        tick = tick.wrapping_add(1);
        unsafe {
            let _ = usleep(FRAME_US);
        }
    }
}

fn draw_wallpaper(fb: &mut Framebuffer, tick: u16) {
    let top = Color::rgb(5, 12, 27);
    let mid = Color::rgb(22, 54, 86);
    let bottom = Color::rgb(58, 24, 75);
    let height = fb.height.max(1) as i32;
    for y in 0..fb.height as i32 {
        let t = ((y * 255) / height) as u8;
        let mut c = if t < 142 {
            top.mix(mid, t.saturating_mul(2))
        } else {
            mid.mix(bottom, t.saturating_sub(142).saturating_mul(2))
        };
        let shimmer = (((tick as i32 + y * 3) & 31) as u8).min(18);
        c = c.mix(Color::rgb(74, 112, 170), shimmer);
        fill_rect(fb, Rect::new(0, y, fb.width, 1), c);
    }

    let moon = Rect::new(fb.width as i32 - 150, 84, 74, 74);
    fill_circle(fb, moon, Color::rgba(210, 220, 245, 54));
    fill_circle(
        fb,
        Rect::new(moon.x + 18, moon.y - 4, 78, 78),
        Color::rgba(6, 12, 28, 220),
    );

    for i in 0..24 {
        let x = ((i * 73 + 19) % fb.width as usize) as i32;
        let y = ((i * 41 + 37) % (fb.height as usize / 2 + 1)) as i32;
        fill_rect(fb, Rect::new(x, y, 1, 1), Color::rgba(230, 238, 255, 130));
    }
}

fn draw_shell(fb: &mut Framebuffer, tick: u16) {
    let w = fb.width as i32;
    let h = fb.height as i32;
    let accent = Color::rgb(72, 132, 248);
    let panel = Rect::new(24, 86, fb.width.saturating_sub(48), fb.height.saturating_sub(136));

    draw_text(fb, 26, 30, "10:08", Color::rgba(240, 246, 255, 210), 2);
    draw_text(
        fb,
        w - 84,
        30,
        "72",
        Color::rgba(240, 246, 255, 180),
        2,
    );
    fill_round_rect(
        fb,
        panel,
        26,
        Color::rgba(22, 20, 39, 168),
    );

    draw_text(fb, panel.x + 24, panel.y + 26, "WING", Color::rgb(255, 255, 255), 3);
    draw_text(
        fb,
        panel.x + 24,
        panel.y + 58,
        "CLEAN SHELL KERNEL",
        Color::rgba(210, 220, 240, 188),
        1,
    );

    let bar = Rect::new(panel.x + 24, panel.y + 86, panel.w.saturating_sub(48), 22);
    fill_round_rect(fb, bar, 11, Color::rgba(244, 246, 252, 210));
    fill_round_rect(
        fb,
        Rect::new(bar.x + 8, bar.y + 7, bar.w.saturating_sub(72), 8),
        4,
        accent,
    );

    let tile_w = (panel.w.saturating_sub(72)) / 3;
    let tile_h = 64;
    let labels = ["DRAW", "DIRTY", "INPUT", "FONT", "SVG", "ECS"];
    for row in 0..2 {
        for col in 0..3 {
            let index = row * 3 + col;
            let x = panel.x + 24 + (tile_w as i32 + 12) * col as i32;
            let y = panel.y + 130 + (tile_h as i32 + 12) * row as i32;
            let rect = Rect::new(x, y, tile_w, tile_h);
            fill_round_rect(fb, rect, 14, Color::rgba(245, 247, 255, 214));
            let pulse = (((tick as usize + index * 17) & 63) as u8).saturating_mul(3);
            fill_circle(
                fb,
                Rect::new(x + tile_w as i32 / 2 - 16, y + 14, 32, 32),
                accent.mix(Color::rgb(110, 222, 238), pulse),
            );
            draw_text(
                fb,
                x + 12,
                y + tile_h as i32 - 18,
                labels[index],
                Color::rgb(35, 45, 66),
                1,
            );
        }
    }

    let card_y = panel.y + 292;
    draw_card(fb, panel.x + 24, card_y, panel.w.saturating_sub(48), "NOTIFY", "LVGL QUALITY PIPELINE");
    draw_card(fb, panel.x + 24, card_y + 76, panel.w.saturating_sub(48), "DEMO", "NO APP LAYERS YET");
    draw_card(fb, panel.x + 24, card_y + 152, panel.w.saturating_sub(48), "NEXT", "DECLARATIVE ECS SPEC");

    fill_round_rect(
        fb,
        Rect::new(w / 2 - 30, h - 34, 60, 6),
        3,
        Color::rgba(250, 250, 255, 220),
    );
}

fn draw_card(fb: &mut Framebuffer, x: i32, y: i32, w: u16, title: &'static str, body: &'static str) {
    let rect = Rect::new(x, y, w, 62);
    fill_round_rect(fb, rect, 18, Color::rgba(250, 250, 255, 224));
    fill_circle(fb, Rect::new(x + 16, y + 16, 30, 30), Color::rgb(72, 132, 248));
    draw_text(fb, x + 58, y + 13, title, Color::rgb(30, 40, 58), 1);
    draw_text(fb, x + 58, y + 34, body, Color::rgba(65, 75, 92, 210), 1);
}

fn fill_rect(fb: &mut Framebuffer, rect: Rect, color: Color) {
    let x0 = rect.x.max(0).min(fb.width as i32) as u16;
    let y0 = rect.y.max(0).min(fb.height as i32) as u16;
    let x1 = (rect.x + rect.w as i32).max(0).min(fb.width as i32) as u16;
    let y1 = (rect.y + rect.h as i32).max(0).min(fb.height as i32) as u16;
    for y in y0..y1 {
        for x in x0..x1 {
            put_pixel(fb, x, y, color);
        }
    }
}

fn fill_circle(fb: &mut Framebuffer, rect: Rect, color: Color) {
    let rx = rect.w as i32 / 2;
    let ry = rect.h as i32 / 2;
    let cx = rect.x + rx;
    let cy = rect.y + ry;
    let r = rx.min(ry).max(1);
    let feather = 2;
    let r2 = r * r;
    let outer = (r + feather) * (r + feather);
    for y in rect.y..rect.y + rect.h as i32 {
        for x in rect.x..rect.x + rect.w as i32 {
            let dx = x - cx;
            let dy = y - cy;
            let d2 = dx * dx + dy * dy;
            if d2 <= r2 {
                put_pixel_checked(fb, x, y, color);
            } else if d2 <= outer {
                put_pixel_checked(fb, x, y, Color { a: color.a / 2, ..color });
            }
        }
    }
}

fn fill_round_rect(fb: &mut Framebuffer, rect: Rect, radius: i32, color: Color) {
    let r = radius.max(1);
    let x1 = rect.x + rect.w as i32 - 1;
    let y1 = rect.y + rect.h as i32 - 1;
    for y in rect.y..=y1 {
        for x in rect.x..=x1 {
            let cx = if x < rect.x + r {
                rect.x + r
            } else if x > x1 - r {
                x1 - r
            } else {
                x
            };
            let cy = if y < rect.y + r {
                rect.y + r
            } else if y > y1 - r {
                y1 - r
            } else {
                y
            };
            let dx = x - cx;
            let dy = y - cy;
            let d2 = dx * dx + dy * dy;
            if d2 <= r * r {
                put_pixel_checked(fb, x, y, color);
            } else if d2 <= (r + 1) * (r + 1) {
                put_pixel_checked(fb, x, y, Color { a: color.a / 2, ..color });
            }
        }
    }
}

fn put_pixel_checked(fb: &mut Framebuffer, x: i32, y: i32, color: Color) {
    if x >= 0 && y >= 0 && x < fb.width as i32 && y < fb.height as i32 {
        put_pixel(fb, x as u16, y as u16, color);
    }
}

fn put_pixel(fb: &mut Framebuffer, x: u16, y: u16, color: Color) {
    unsafe {
        if fb.bpp == 16 || fb.fmt == FB_FMT_RGB16_565 {
            let offset = y as usize * fb.stride + x as usize * 2;
            let ptr = fb.ptr.add(offset) as *mut u16;
            let src = rgb565(color);
            if color.a == 255 {
                ptr.write_volatile(src);
            } else {
                let dst = ptr.read_volatile();
                ptr.write_volatile(blend565(dst, src, color.a));
            }
        } else {
            let offset = y as usize * fb.stride + x as usize * 4;
            let ptr = fb.ptr.add(offset) as *mut u32;
            let src = ((color.r as u32) << 16) | ((color.g as u32) << 8) | color.b as u32;
            if color.a == 255 {
                ptr.write_volatile(src);
            } else {
                let dst = ptr.read_volatile();
                ptr.write_volatile(blend888(dst, src, color.a));
            }
        }
    }
}

fn rgb565(color: Color) -> u16 {
    ((color.r as u16 & 0xf8) << 8) | ((color.g as u16 & 0xfc) << 3) | (color.b as u16 >> 3)
}

fn blend565(dst: u16, src: u16, alpha: u8) -> u16 {
    let dr = ((dst >> 11) & 0x1f) << 3;
    let dg = ((dst >> 5) & 0x3f) << 2;
    let db = (dst & 0x1f) << 3;
    let sr = ((src >> 11) & 0x1f) << 3;
    let sg = ((src >> 5) & 0x3f) << 2;
    let sb = (src & 0x1f) << 3;
    let a = alpha as u16;
    let inv = 255u16.saturating_sub(a);
    rgb565(Color::rgb(
        ((sr * a + dr * inv) / 255) as u8,
        ((sg * a + dg * inv) / 255) as u8,
        ((sb * a + db * inv) / 255) as u8,
    ))
}

fn blend888(dst: u32, src: u32, alpha: u8) -> u32 {
    let a = alpha as u32;
    let inv = 255u32.saturating_sub(a);
    let dr = (dst >> 16) & 0xff;
    let dg = (dst >> 8) & 0xff;
    let db = dst & 0xff;
    let sr = (src >> 16) & 0xff;
    let sg = (src >> 8) & 0xff;
    let sb = src & 0xff;
    (((sr * a + dr * inv) / 255) << 16)
        | (((sg * a + dg * inv) / 255) << 8)
        | ((sb * a + db * inv) / 255)
}

fn draw_text(fb: &mut Framebuffer, x: i32, y: i32, text: &str, color: Color, scale: u8) {
    let mut cursor = x;
    for byte in text.bytes() {
        if byte == b' ' {
            cursor += 4 * scale as i32;
        } else {
            draw_glyph(fb, cursor, y, byte, color, scale);
            cursor += 6 * scale as i32;
        }
    }
}

fn draw_glyph(fb: &mut Framebuffer, x: i32, y: i32, byte: u8, color: Color, scale: u8) {
    let glyph = glyph5x7(byte);
    let scale = scale.max(1) as i32;
    for (row, bits) in glyph.iter().copied().enumerate() {
        for col in 0..5 {
            if (bits & (1 << (4 - col))) != 0 {
                fill_rect(
                    fb,
                    Rect::new(
                        x + col as i32 * scale,
                        y + row as i32 * scale,
                        scale as u16,
                        scale as u16,
                    ),
                    color,
                );
            }
        }
    }
}

fn glyph5x7(byte: u8) -> [u8; 7] {
    match byte {
        b'0' => [0x0e, 0x11, 0x13, 0x15, 0x19, 0x11, 0x0e],
        b'1' => [0x04, 0x0c, 0x04, 0x04, 0x04, 0x04, 0x0e],
        b'2' => [0x0e, 0x11, 0x01, 0x02, 0x04, 0x08, 0x1f],
        b'3' => [0x1e, 0x01, 0x01, 0x0e, 0x01, 0x01, 0x1e],
        b'4' => [0x02, 0x06, 0x0a, 0x12, 0x1f, 0x02, 0x02],
        b'5' => [0x1f, 0x10, 0x10, 0x1e, 0x01, 0x01, 0x1e],
        b'6' => [0x06, 0x08, 0x10, 0x1e, 0x11, 0x11, 0x0e],
        b'7' => [0x1f, 0x01, 0x02, 0x04, 0x08, 0x08, 0x08],
        b'8' => [0x0e, 0x11, 0x11, 0x0e, 0x11, 0x11, 0x0e],
        b'9' => [0x0e, 0x11, 0x11, 0x0f, 0x01, 0x02, 0x0c],
        b':' => [0x00, 0x04, 0x04, 0x00, 0x04, 0x04, 0x00],
        b'/' => [0x01, 0x01, 0x02, 0x04, 0x08, 0x10, 0x10],
        b'+' => [0x00, 0x04, 0x04, 0x1f, 0x04, 0x04, 0x00],
        b'-' => [0x00, 0x00, 0x00, 0x1f, 0x00, 0x00, 0x00],
        b'A' => [0x0e, 0x11, 0x11, 0x1f, 0x11, 0x11, 0x11],
        b'B' => [0x1e, 0x11, 0x11, 0x1e, 0x11, 0x11, 0x1e],
        b'C' => [0x0e, 0x11, 0x10, 0x10, 0x10, 0x11, 0x0e],
        b'D' => [0x1e, 0x11, 0x11, 0x11, 0x11, 0x11, 0x1e],
        b'E' => [0x1f, 0x10, 0x10, 0x1e, 0x10, 0x10, 0x1f],
        b'F' => [0x1f, 0x10, 0x10, 0x1e, 0x10, 0x10, 0x10],
        b'G' => [0x0e, 0x11, 0x10, 0x17, 0x11, 0x11, 0x0f],
        b'H' => [0x11, 0x11, 0x11, 0x1f, 0x11, 0x11, 0x11],
        b'I' => [0x0e, 0x04, 0x04, 0x04, 0x04, 0x04, 0x0e],
        b'K' => [0x11, 0x12, 0x14, 0x18, 0x14, 0x12, 0x11],
        b'L' => [0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x1f],
        b'M' => [0x11, 0x1b, 0x15, 0x15, 0x11, 0x11, 0x11],
        b'N' => [0x11, 0x19, 0x15, 0x13, 0x11, 0x11, 0x11],
        b'O' => [0x0e, 0x11, 0x11, 0x11, 0x11, 0x11, 0x0e],
        b'P' => [0x1e, 0x11, 0x11, 0x1e, 0x10, 0x10, 0x10],
        b'Q' => [0x0e, 0x11, 0x11, 0x11, 0x15, 0x12, 0x0d],
        b'R' => [0x1e, 0x11, 0x11, 0x1e, 0x14, 0x12, 0x11],
        b'S' => [0x0f, 0x10, 0x10, 0x0e, 0x01, 0x01, 0x1e],
        b'T' => [0x1f, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04],
        b'U' => [0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x0e],
        b'V' => [0x11, 0x11, 0x11, 0x11, 0x11, 0x0a, 0x04],
        b'W' => [0x11, 0x11, 0x11, 0x15, 0x15, 0x15, 0x0a],
        b'X' => [0x11, 0x11, 0x0a, 0x04, 0x0a, 0x11, 0x11],
        b'Y' => [0x11, 0x11, 0x0a, 0x04, 0x04, 0x04, 0x04],
        b'Z' => [0x1f, 0x01, 0x02, 0x04, 0x08, 0x10, 0x1f],
        _ => [0x1f, 0x11, 0x05, 0x02, 0x04, 0x00, 0x04],
    }
}

#[panic_handler]
fn panic(_panic: &PanicInfo<'_>) -> ! {
    loop {}
}

#[no_mangle]
extern "C" fn rust_eh_personality(
    _version: i32,
    _actions: i32,
    _exception_class: u32,
    _exception_object: *mut c_void,
    _context: *mut c_void,
) -> i32 {
    loop {}
}
