use crate::room::RoomId;
use std::hash::{Hash, Hasher};

#[derive(Debug)]
pub struct RoomConnection {
    pub room0_id: RoomId,
    pub room1_id: RoomId,
    pub squared_length: f32,
}

impl Eq for RoomConnection {}

impl PartialEq for RoomConnection {
    fn eq(&self, other: &Self) -> bool {
        self.room0_id == other.room0_id && self.room1_id == other.room1_id
    }
}

impl Hash for RoomConnection {
    fn hash<H: Hasher>(&self, state: &mut H) {
        if self.room0_id.inner() < self.room1_id.inner() {
            (self.room0_id, self.room1_id).hash(state);
        } else {
            (self.room1_id, self.room0_id).hash(state);
        }
    }
}
