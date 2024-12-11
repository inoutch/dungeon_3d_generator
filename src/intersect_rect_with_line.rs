use crate::intersect_line_and_line::intersect_line_and_line;
use nalgebra::Vector2;

pub fn intersect_rect_with_line(
    rect: (&Vector2<f32>, &Vector2<f32>),
    p0: &Vector2<f32>,
    p1: &Vector2<f32>,
) -> Vec<Vector2<f32>> {
    let l_b = Vector2::new(rect.0.x, rect.0.y);
    let l_t = Vector2::new(rect.0.x, rect.0.y + rect.1.y);
    let r_b = Vector2::new(rect.0.x + rect.1.x, rect.0.y);
    let r_t = Vector2::new(rect.0.x + rect.1.x, rect.0.y + rect.1.y);
    let mut ret = Vec::new();
    if let Some(p) = intersect_line_and_line(p0, p1, &l_t, &r_t) {
        ret.push(p);
    }
    if let Some(p) = intersect_line_and_line(p0, p1, &l_b, &r_b) {
        ret.push(p);
    }
    if let Some(p) = intersect_line_and_line(p0, p1, &l_b, &l_t) {
        ret.push(p);
    }
    if let Some(p) = intersect_line_and_line(p0, p1, &r_b, &r_t) {
        ret.push(p);
    }
    ret
}
