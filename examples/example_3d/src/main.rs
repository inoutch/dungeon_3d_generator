use dungeon_3d_generator::gen::{generate_dungeon_3d, Dungeon3DGeneratorConfig};
use kiss3d::light::Light;
use kiss3d::nalgebra::{Point3, Translation3};
use kiss3d::window::Window;

fn main() {
    let mut window = Window::new("Dungeon 3D Generator");

    window.set_light(Light::StickToCamera);

    let dungeon = generate_dungeon_3d(Dungeon3DGeneratorConfig {
        seed: Some(0),
        ..Default::default()
    })
    .unwrap();

    for (_, room) in dungeon.rooms.iter() {
        let mut c = window.add_cube(room.width as f32, room.height as f32, room.depth as f32);
        c.set_color(1.0, 0.0, 0.0);
        c.set_local_translation(Translation3::new(
            room.origin.0 as f32 + room.center_offset.0,
            room.origin.1 as f32 + room.center_offset.1,
            room.origin.2 as f32 + room.center_offset.2,
        ));
    }

    while window.render() {
        for room_connection in dungeon.room_connections.iter() {
            let room0 = dungeon.rooms.get(&room_connection.room0_id).unwrap();
            let room1 = dungeon.rooms.get(&room_connection.room1_id).unwrap();
            let room0_center = room0.center();
            let room1_center = room1.center();
            window.draw_line(
                &Point3::new(room0_center.0, room0_center.1, room0_center.2),
                &Point3::new(room1_center.0, room1_center.1, room1_center.2),
                &Point3::new(0.0, 1.0, 0.0),
            );
        }
    }
}
