use crate::constants::{Direction4, VoxelType};
use crate::room::RoomId;
use std::collections::BTreeSet;

#[derive(Debug)]
pub struct Passage {
    pub cells: Vec<((i32, i32, i32), VoxelType)>,
    pub start: (i32, i32, i32),
    pub start_dirs: BTreeSet<Direction4>,
    pub start_room_id: RoomId,
    pub end_room_id: RoomId,
    pub height: i32,
}
