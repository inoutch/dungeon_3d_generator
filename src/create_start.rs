use crate::constants::Direction4;
use crate::intersect_rect_with_line::intersect_rect_with_line;
use crate::room::{Room, RoomId};
use nalgebra::{Vector2, Vector3};
use std::collections::BTreeSet;

pub fn create_start(
    room0: &Room,
    room1: &Room,
) -> (RoomId, RoomId, Vector3<i32>, BTreeSet<Direction4>) {
    let (room_start, room_end) = if room0.origin.1 <= room1.origin.1 {
        (room0, room1)
    } else {
        (room1, room0)
    };
    let room_start_center = room_start.center();
    let room_end_center = room_end.center();
    let diff_center = (
        room_end_center.0 - room_start_center.0,
        room_end_center.2 - room_start_center.2,
    );
    let width = room_start.width + room_end.width;
    let depth = room_start.depth + room_end.depth;
    let mut points = intersect_rect_with_line(
        (
            &Vector2::new(room_start.origin.0 as f32, room_start.origin.2 as f32),
            &Vector2::new(room_start.width as f32, room_start.depth as f32),
        ),
        &Vector2::new(room_start_center.0, room_start_center.2),
        &Vector2::new(diff_center.0 * width as f32, diff_center.1 * depth as f32),
    );
    let mut dirs = BTreeSet::new();
    let mut p = points
        .pop()
        .map(|p| Vector3::new(p.x as i32, room_start.origin.1 as i32, p.y as i32))
        .unwrap_or_else(|| {
            Vector3::new(
                room_start.origin.0 as i32,
                room_start.origin.1 as i32,
                room_start.origin.2 as i32,
            )
        });

    if p.x == room_start.origin.0 as i32 {
        dirs.insert(Direction4::Left);
    } else if p.x == (room_start.origin.0 + room_start.width) as i32 {
        p.x -= 1;
        dirs.insert(Direction4::Right);
    }

    if p.z == room_start.origin.2 as i32 {
        dirs.insert(Direction4::Far);
    } else if p.z == (room_start.origin.2 + room_start.depth) as i32 {
        p.z -= 1;
        dirs.insert(Direction4::Near);
    }

    (room_start.id, room_end.id, p, dirs)
}
