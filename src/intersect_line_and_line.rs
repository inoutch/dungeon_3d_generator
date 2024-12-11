use nalgebra::Vector2;

pub fn intersect_line_and_line(
    p00: &Vector2<f32>,
    p01: &Vector2<f32>,
    p10: &Vector2<f32>,
    p11: &Vector2<f32>,
) -> Option<Vector2<f32>> {
    let d = (p01.x - p00.x) * (p11.y - p10.y) - (p01.y - p00.y) * (p11.x - p10.x);
    if d == 0.0 {
        return None;
    }

    let v = *p10 - *p00;
    let d_r = ((p11.y - p10.y) * v.x - (p11.x - p10.x) * v.y) / d;
    let d_s = ((p01.y - p00.y) * v.x - (p01.x - p00.x) * v.y) / d;

    if (0.0..=1.0).contains(&d_r) && (0.0..1.0).contains(&d_s) {
        Some(*p00 + d_r * (*p01 - *p00))
    } else {
        None
    }
}
