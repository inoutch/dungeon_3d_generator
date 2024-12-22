use crate::room::RoomId;
use nalgebra::Vector3;
use std::sync::LazyLock;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Direction4 {
    Left,
    Right,
    Far,
    Near,
}

impl Direction4 {
    pub fn to_vec3(&self) -> Vector3<i32> {
        match self {
            Direction4::Left => Vector3::new(-1, 0, 0),
            Direction4::Right => Vector3::new(1, 0, 0),
            Direction4::Far => Vector3::new(0, 0, -1),
            Direction4::Near => Vector3::new(0, 0, 1),
        }
    }

    pub fn is_opposite(&self, other: &Self) -> bool {
        match self {
            Direction4::Left => other == &Direction4::Right,
            Direction4::Right => other == &Direction4::Left,
            Direction4::Far => other == &Direction4::Near,
            Direction4::Near => other == &Direction4::Right,
        }
    }
}

pub static DIRECTIONS: LazyLock<[Direction4; 4]> = LazyLock::new(|| {
    [
        Direction4::Left,
        Direction4::Right,
        Direction4::Far,
        Direction4::Near,
    ]
});

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum VoxelType {
    RoomSpace(RoomId),       // 部屋の空間
    RoomFloor(RoomId),       // 部屋の床
    RoomBottomSpace(RoomId), // 部屋の移動可能な空間
    RoomWall(RoomId),        // 部屋の壁
    Wall,
    PassageStair(Direction4),
    PassageSpace,
    PassageFloor,
}
