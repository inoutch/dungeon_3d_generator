use dungeon_3d_generator::core_expansion_dungeon::{generate_ced, CEDConfig};
use kiss3d::light::Light;
use kiss3d::nalgebra::Translation3;
use kiss3d::window::Window;
use rand::prelude::SliceRandom;
use rand::{Rng, SeedableRng};

fn main() {
    let mut window = Window::new("Core expansion dungeon 3D");
    window.set_light(Light::StickToCamera);

    let dungeon = generate_ced(CEDConfig {
        seed: Some(1),
        ..Default::default()
    })
    .unwrap();

    for (index, (room_candidate_index, origin)) in
        dungeon.room_candidate_indices.into_iter().enumerate()
    {
        let room = &dungeon.room_candidates[room_candidate_index];
        let mut c = window.add_cube(room.width as f32, room.height as f32, room.depth as f32);
        let (r, g, b) = generate_random_color_from_i32(index as i32);
        c.set_color(r, g, b);
        c.set_local_translation(Translation3::new(
            origin.0 as f32 + room.width as f32 / 2.0,
            origin.1 as f32 + room.height as f32 / 2.0,
            origin.2 as f32 + room.depth as f32 / 2.0,
        ));
    }

    while window.render() {}
}

fn generate_random_color_from_i32(value: i32) -> (f32, f32, f32) {
    let mut rng: rand::rngs::StdRng = SeedableRng::seed_from_u64(value as u64);

    let hue_buckets = [
        (255, 0, 0),   // 赤
        (0, 255, 0),   // 緑
        (0, 0, 255),   // 青
        (255, 255, 0), // 黄色
        (255, 0, 255), // マゼンタ
        (0, 255, 255),
    ];
    let base_color = *hue_buckets.choose(&mut rng).unwrap();
    let r = (base_color.0 + rng.gen_range(-30..=30)).clamp(0, 255) as u8;
    let g = (base_color.1 + rng.gen_range(-30..=30)).clamp(0, 255) as u8;
    let b = (base_color.2 + rng.gen_range(-30..=30)).clamp(0, 255) as u8;

    (r as f32 / 256.0, g as f32 / 256.0, b as f32 / 256.0)
}
