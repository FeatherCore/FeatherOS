use crate::asset::WALLPAPER_5;
use fhre::{
    Camera, Color, DrawCommand, DrawList, ImageFit, Rect, RenderNode, Size, Surface, Transform3D,
};

pub(crate) fn draw_wallpaper(surface: &mut Surface, _frame: u32) {
    let w = surface.width();
    let h = surface.height();
    let mut list: DrawList<48> = DrawList::new();
    let camera = Camera::screen_canvas(w, h);
    let bg = RenderNode::new(Transform3D::screen(0, 0, -32), Size::new(w, h)).project(&camera);
    list.push(DrawCommand::FillGradient {
        rect: bg.rect,
        depth: bg.depth,
        top: Color::rgb(5, 12, 28),
        bottom: Color::rgb(52, 28, 76),
    });
    list.push(DrawCommand::DrawImageFit {
        rect: bg.rect,
        depth: bg.depth + 1,
        image: WALLPAPER_5.image(),
        opacity: 255,
        fit: ImageFit::Cover,
    });

    let horizon = h as i32 - h as i32 / 4;
    let water = RenderNode::new(
        Transform3D::screen(0, horizon, -24),
        Size::new(w, h.saturating_sub(horizon as u16)),
    )
    .project(&camera);
    list.push(DrawCommand::FillGradient {
        rect: water.rect,
        depth: water.depth,
        top: Color::rgba(40, 92, 118, 140),
        bottom: Color::rgba(9, 20, 36, 230),
    });

    let moon_x = w as i32 - 92;
    let moon_y = 74;
    list.push(DrawCommand::FillCircle {
        center: fhre::Point::new(moon_x, moon_y),
        depth: -20,
        radius: 34,
        color: Color::rgba(222, 232, 255, 86),
    });
    list.push(DrawCommand::FillCircle {
        center: fhre::Point::new(moon_x + 14, moon_y - 4),
        depth: -19,
        radius: 38,
        color: Color::rgba(5, 12, 28, 228),
    });

    for i in 0..36 {
        let x = ((i * 47 + 21) % w.max(1) as usize) as i32;
        let y = ((i * 29 + 17) % (h.max(2) as usize / 2)) as i32;
        list.push(DrawCommand::FillRect {
            rect: Rect::new(x, y, 1, 1),
            depth: -18,
            color: Color::rgba(236, 244, 255, 150),
        });
    }

    list.sort_by_depth();
    list.execute(surface);
}
