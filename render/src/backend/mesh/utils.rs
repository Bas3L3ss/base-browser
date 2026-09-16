use layout::Rect;
use winit::dpi::PhysicalSize;

pub fn pixel_to_ndc(x: f32, y: f32, viewport: PhysicalSize<u32>) -> (f32, f32) {
    let w = viewport.width as f32;
    let h = viewport.height as f32;
    let x_ndc = (2.0 * x) / w - 1.0;
    let y_ndc = 1.0 - (2.0 * y) / h;
    (x_ndc, y_ndc)
}

pub fn physical_rect_to_ndc(rect: &Rect, viewport: PhysicalSize<u32>) -> (f32, f32, f32, f32) {
    let (x_ndc, y_ndc) = pixel_to_ndc(rect.x, rect.y, viewport);
    let w = viewport.width as f32;
    let h = viewport.height as f32;
    let width_ndc = (2.0 * rect.width) / w;
    let height_ndc = (-2.0 * rect.height) / h;
    (x_ndc, y_ndc, width_ndc, height_ndc)
}

pub fn physical_triangle_to_ndc(
    points: &[(f32, f32); 3],
    viewport: PhysicalSize<u32>
) -> [(f32, f32); 3] {
    [
        pixel_to_ndc(points[0].0, points[0].1, viewport),
        pixel_to_ndc(points[1].0, points[1].1, viewport),
        pixel_to_ndc(points[2].0, points[2].1, viewport),
    ]
}
