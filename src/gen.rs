use crate::delaunary_3d::{Delaunay3D, Edge};
use nalgebra::Vector3;
use pathfinding::prelude::kruskal;
use rand::{Rng, SeedableRng};
use std::collections::BTreeMap;
use std::ops::RangeInclusive;
use std::rc::Rc;

pub struct Dungeon3DGeneratorConfig {
    pub width: u32,        // Width of entire dungeon (x-axis)
    pub height: u32,       // Height of entire dungeon (y-axis)
    pub depth: u32,        // Depth of entire dungeon (z-axis)
    pub seed: Option<u64>, // Seed value for random dungeon generation
    pub room_hierarchy: u32,
    pub room_width_range: RangeInclusive<u32>,
    pub room_height_range: RangeInclusive<u32>,
    pub room_depth_range: RangeInclusive<u32>,
    pub room_margin: u32,
}

impl Default for Dungeon3DGeneratorConfig {
    fn default() -> Self {
        Dungeon3DGeneratorConfig {
            width: 32,
            height: 16,
            depth: 32,
            seed: None,
            room_hierarchy: 3,
            room_width_range: 5..=10,
            room_height_range: 2..=3,
            room_depth_range: 5..=10,
            room_margin: 2,
        }
    }
}

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
}

#[derive(Debug)]
pub struct RoomConnection {
    pub room0_id: RoomId,
    pub room1_id: RoomId,
    pub squared_length: f32,
}

#[derive(Debug)]
pub struct Dungeon3DGeneratorResult {
    pub rooms: BTreeMap<RoomId, Room>,
    pub room_connections: Vec<Rc<RoomConnection>>,
    pub edges: Vec<Edge>,
}

#[derive(Debug)]
pub enum Dungeon3DGeneratorError {
    NarrowWidthOrRoomWidthTooLarge,
    NarrowDepthOrRoomDepthTooLarge,
    NarrowHeightOrRoomHierarchyTooSmall,
}

pub fn generate_dungeon_3d(
    config: Dungeon3DGeneratorConfig,
) -> Result<Dungeon3DGeneratorResult, Dungeon3DGeneratorError> {
    // validate
    let w_divisions_max = config.width / (config.room_width_range.start() + config.room_margin);
    let w_divisions_min = config.width / (config.room_width_range.end() + config.room_margin);
    if w_divisions_min == 0 {
        return Err(Dungeon3DGeneratorError::NarrowWidthOrRoomWidthTooLarge);
    }
    let d_divisions_max = config.width / (config.room_depth_range.start() + config.room_margin);
    let d_divisions_min = config.width / (config.room_depth_range.end() + config.room_margin);
    if d_divisions_min == 0 {
        return Err(Dungeon3DGeneratorError::NarrowDepthOrRoomDepthTooLarge);
    }
    if config.room_hierarchy * (config.room_height_range.start() + config.room_margin)
        > config.height
    {
        return Err(Dungeon3DGeneratorError::NarrowHeightOrRoomHierarchyTooSmall);
    }

    let mut rng: rand::rngs::StdRng = config
        .seed
        .map(SeedableRng::seed_from_u64)
        .unwrap_or_else(rand::rngs::StdRng::from_entropy);

    let mut room_id = RoomId::first();
    let mut rooms = BTreeMap::new();
    let mut room_ids = Vec::new();
    let h_block_size = config.height / config.room_hierarchy;
    for ry in 0..config.room_hierarchy {
        let w_divisions = rng.gen_range(1..=w_divisions_max);
        let w_block_size = config.width / w_divisions;
        for rx in 0..w_divisions {
            let d_divisions = rng.gen_range(1..=d_divisions_max);
            let d_block_size = config.depth / d_divisions;
            for rz in 0..d_divisions {
                let room_width = rng.gen_range(
                    *config.room_width_range.start()
                        ..=(w_block_size - config.room_margin).min(*config.room_width_range.end()),
                );
                let room_height = rng.gen_range(
                    *config.room_height_range.start()
                        ..=(h_block_size - config.room_margin).min(*config.room_height_range.end()),
                );
                let room_depth = rng.gen_range(
                    *config.room_depth_range.start()
                        ..=(d_block_size - config.room_margin).min(*config.room_depth_range.end()),
                );
                let (origin_x, origin_y, origin_z) =
                    (rx * w_block_size, ry * h_block_size, rz * d_block_size);
                let room_origin = (
                    origin_x + rng.gen_range(0..=(w_block_size - room_width - config.room_margin)),
                    origin_y + rng.gen_range(0..=(h_block_size - room_height - config.room_margin)),
                    origin_z + rng.gen_range(0..=(d_block_size - room_depth - config.room_margin)),
                );
                let new_room_id = room_id.gen_id();
                room_ids.push(new_room_id);
                rooms.insert(
                    new_room_id,
                    Room::new(
                        new_room_id,
                        room_width,
                        room_height,
                        room_depth,
                        room_origin,
                    ),
                );
            }
        }
    }

    let center = (
        config.width as f32 / 2.0f32,
        config.height as f32 / 2.0,
        config.depth as f32 / 2.0f32,
    );
    let mut sorted_rooms = rooms
        .values()
        .map(|room| {
            let room_center = room.center();
            let diff = (
                room_center.0 - center.0,
                room_center.1 - center.1,
                room_center.2 - center.2,
            );
            let squared_length = diff.0 * diff.0 + diff.1 * diff.1 + diff.2 * diff.2;
            (room.id, squared_length)
        })
        .collect::<Vec<_>>();
    sorted_rooms.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

    let mut room_connections = Vec::new();
    let mut room_connection_map: BTreeMap<RoomId, BTreeMap<RoomId, Rc<RoomConnection>>> =
        BTreeMap::new();
    for room_index in 0..room_ids.len() {
        let current_room_id = room_ids[room_index];
        let current_room = rooms.get(&current_room_id).unwrap();
        let current_room_center = current_room.center();
        for target_room_id in &room_ids[(room_index + 1)..rooms.len()] {
            let target_room = rooms.get(target_room_id).unwrap();
            let target_room_center = target_room.center();
            let diff = (
                current_room_center.0 - target_room_center.0,
                current_room_center.1 - target_room_center.1,
                current_room_center.2 - target_room_center.2,
            );
            let squared_length = diff.0 * diff.0 + diff.1 * diff.1 + diff.2 * diff.2;
            let room_connection = Rc::new(RoomConnection {
                room0_id: current_room.id,
                room1_id: target_room.id,
                squared_length,
            });
            room_connections.push(room_connection.clone());
            room_connection_map
                .entry(target_room.id)
                .or_default()
                .insert(current_room.id, room_connection.clone());
            room_connection_map
                .entry(current_room.id)
                .or_default()
                .insert(target_room.id, room_connection.clone());
        }
    }

    let delaunay = Delaunay3D::new(
        rooms
            .values()
            .map(|room| {
                let center = room.center();
                Vector3::new(center.0, center.1, center.2)
            })
            .collect(),
    );

    for (_, room) in rooms.iter_mut() {
        let mut room_origin = room.origin;
    }

    // Create mst of room neighbors
    let weighted_edges = room_connections
        .iter()
        .map(|room_connection| {
            (
                room_connection.room0_id,
                room_connection.room1_id,
                room_connection.squared_length as u64,
            )
        })
        .collect::<Vec<_>>();
    let room_connections = kruskal(&weighted_edges)
        .map(|(room0_id, room1_id, _)| {
            Rc::clone(
                room_connection_map
                    .get(room0_id)
                    .unwrap()
                    .get(room1_id)
                    .unwrap(),
            )
        })
        .collect::<Vec<_>>();

    Ok(Dungeon3DGeneratorResult {
        rooms,
        room_connections,
        edges: delaunay.edges,
    })
}

#[cfg(test)]
mod tests {
    use crate::gen::{generate_dungeon_3d, Dungeon3DGeneratorConfig};

    #[test]
    fn test_default_generate() {
        let result = generate_dungeon_3d(Dungeon3DGeneratorConfig {
            seed: Some(0),
            ..Default::default()
        })
        .unwrap();
        insta::assert_debug_snapshot!(result);
    }
}
