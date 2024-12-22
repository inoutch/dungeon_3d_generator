#[derive(Debug)]
pub struct Room {
    pub id: RoomId,
    pub width: u32,
    pub height: u32,
    pub depth: u32,
    pub origin: (u32, u32, u32),
    pub center_offset: (f32, f32, f32),
}

impl Room {
    pub fn new(id: RoomId, width: u32, height: u32, depth: u32, origin: (u32, u32, u32)) -> Self {
        Room {
            id,
            width,
            height,
            depth,
            origin,
            center_offset: (width as f32 / 2.0, height as f32 / 2.0, depth as f32 / 2.0),
        }
    }

    pub fn center(&self) -> (f32, f32, f32) {
        (
            self.center_offset.0 + self.origin.0 as f32,
            self.center_offset.1 + self.origin.1 as f32,
            self.center_offset.2 + self.origin.2 as f32,
        )
    }

    pub fn end(&self) -> (u32, u32, u32) {
        (
            self.origin.0 + self.width,
            self.origin.1 + self.height,
            self.origin.2 + self.depth,
        )
    }

    pub fn is_contract(&self, other: &Room, margin: u32) -> bool {
        let self_end = self.end();
        let self_end = (
            self_end.0 + margin,
            self_end.1 + margin,
            self_end.2 + margin,
        );
        let other_end = other.end();
        let other_end = (
            other_end.0 + margin,
            other_end.1 + margin,
            other_end.2 + margin,
        );
        self.origin.0 <= other_end.0
            && other.origin.0 <= self_end.0
            && self.origin.1 <= other_end.1
            && other.origin.1 <= self_end.1
            && self.origin.2 <= other_end.2
            && other.origin.2 <= self_end.2
    }
}

#[derive(Ord, PartialOrd, PartialEq, Eq, Hash, Copy, Clone, Debug)]
pub struct RoomId(u64);

impl RoomId {
    pub fn first() -> Self {
        RoomId(1)
    }

    pub fn gen_id(&mut self) -> Self {
        let ret = *self;
        self.0 += 1;
        ret
    }

    pub fn inner(&self) -> u64 {
        self.0
    }
}
