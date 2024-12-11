use crate::gen::RoomId;

#[derive(Debug, Copy, Clone)]
pub enum Direction4 {
    Left,
    Right,
    Far,
    Near,
}

#[derive(Debug, Copy, Clone)]
pub enum Cell {
    RoomSpace(RoomId),
    RoomFloor(RoomId),
    Stair(Direction4),
    PassageSpace,
    PassageFloor,
}
