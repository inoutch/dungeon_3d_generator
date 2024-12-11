use crate::constants::Cell;
use crate::delaunary_3d::Delaunay3D;
use crate::intersect_rect_with_line::intersect_rect_with_line;
use nalgebra::{Vector2, Vector3};
use pathfinding::prelude::kruskal;
use rand::{Rng, SeedableRng};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::hash::{Hash, Hasher};
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
    pub room_margin_x: u32,
    pub room_margin_y: u32,
    pub room_margin_z: u32,
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
            room_margin_x: 2,
            room_margin_y: 1,
            room_margin_z: 2,
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

impl Eq for RoomConnection {}

impl PartialEq for RoomConnection {
    fn eq(&self, other: &Self) -> bool {
        self.room0_id == other.room0_id && self.room1_id == other.room1_id
    }
}

impl Hash for RoomConnection {
    fn hash<H: Hasher>(&self, state: &mut H) {
        if self.room0_id.0 < self.room1_id.0 {
            (self.room0_id, self.room1_id).hash(state);
        } else {
            (self.room1_id, self.room0_id).hash(state);
        }
    }
}

#[derive(Debug)]
pub struct Passage {
    pub cells: Vec<((i32, i32, i32), Cell)>,
    pub start: (i32, i32, i32),
    pub start_room_id: RoomId,
    pub end_room_id: RoomId,
}

#[derive(Debug)]
pub struct Dungeon3DGeneratorResult {
    pub rooms: BTreeMap<RoomId, Room>,
    pub room_connections: HashSet<Rc<RoomConnection>>,
    pub cell_map: HashMap<Vector3<i32>, Cell>,
    pub passages: Vec<Passage>,
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
    let w_divisions_max = config.width / (config.room_width_range.start() + config.room_margin_x);
    let w_divisions_min = config.width / (config.room_width_range.end() + config.room_margin_x);
    if w_divisions_min == 0 {
        return Err(Dungeon3DGeneratorError::NarrowWidthOrRoomWidthTooLarge);
    }
    let d_divisions_max = config.width / (config.room_depth_range.start() + config.room_margin_z);
    let d_divisions_min = config.width / (config.room_depth_range.end() + config.room_margin_z);
    if d_divisions_min == 0 {
        return Err(Dungeon3DGeneratorError::NarrowDepthOrRoomDepthTooLarge);
    }
    if config.room_hierarchy * (config.room_height_range.start() + config.room_margin_y)
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
                        ..=(w_block_size - config.room_margin_x)
                            .min(*config.room_width_range.end()),
                );
                let room_height = rng.gen_range(
                    *config.room_height_range.start()
                        ..=(h_block_size - config.room_margin_y)
                            .min(*config.room_height_range.end()),
                );
                let room_depth = rng.gen_range(
                    *config.room_depth_range.start()
                        ..=(d_block_size - config.room_margin_z)
                            .min(*config.room_depth_range.end()),
                );
                let (origin_x, origin_y, origin_z) =
                    (rx * w_block_size, ry * h_block_size, rz * d_block_size);
                let room_origin = (
                    origin_x
                        + rng.gen_range(0..=(w_block_size - room_width - config.room_margin_x)),
                    origin_y
                        + rng.gen_range(0..=(h_block_size - room_height - config.room_margin_y)),
                    origin_z
                        + rng.gen_range(0..=(d_block_size - room_depth - config.room_margin_z)),
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

    for (_, room) in rooms.iter_mut() {
        let mut room_origin = room.origin;
    }

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

    let mut necessary_room_connections = kruskal(&weighted_edges)
        .map(|(room0_id, room1_id, _)| {
            Rc::clone(
                room_connection_map
                    .get(room0_id)
                    .unwrap()
                    .get(room1_id)
                    .unwrap(),
            )
        })
        .collect::<HashSet<_>>();

    let delaunay = Delaunay3D::new(
        rooms
            .values()
            .map(|room| {
                let center = room.center();
                (room.id, Vector3::new(center.0, center.1, center.2))
            })
            .collect(),
    );
    let room_connections = delaunay
        .edges
        .iter()
        .map(|edge| RoomConnection {
            room0_id: *delaunay.id_map.get(&edge.u).unwrap(),
            room1_id: *delaunay.id_map.get(&edge.v).unwrap(),
            squared_length: (edge.u.position - edge.v.position).norm_squared(),
        })
        .collect::<Vec<_>>();

    for room_connection in room_connections {
        if rng.gen_bool(0.3) {
            necessary_room_connections.insert(Rc::new(room_connection));
        }
    }

    // create passages
    let mut passages = Vec::new();
    for room_connection in necessary_room_connections.iter() {
        let r0 = rooms.get(&room_connection.room0_id).unwrap();
        let r1 = rooms.get(&room_connection.room1_id).unwrap();
        let (start_room_id, end_room_id, start) = create_start(r0, r1);
        passages.push(Passage {
            cells: Vec::new(),
            start: (start.x, start.y, start.z),
            start_room_id,
            end_room_id,
        });
    }

    let mut cell_map: HashMap<Vector3<i32>, Cell> = HashMap::new();
    for (room_id, room) in rooms.iter() {
        for y in -1..room.height as i32 {
            for z in 0..room.depth as i32 {
                for x in 0..room.width as i32 {
                    cell_map.insert(
                        Vector3::new(
                            room.origin.0 as i32 + x,
                            room.origin.1 as i32 + y,
                            room.origin.2 as i32 + z,
                        ),
                        if y == -1 {
                            Cell::RoomFloor(*room_id)
                        } else {
                            Cell::RoomSpace(*room_id)
                        },
                    );
                }
            }
        }
    }

    Ok(Dungeon3DGeneratorResult {
        rooms,
        room_connections: necessary_room_connections,
        cell_map,
        passages,
    })
}

fn create_start(room0: &Room, room1: &Room) -> (RoomId, RoomId, Vector3<i32>) {
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
    if let Some(mut p) = points
        .pop()
        .map(|p| Vector3::new(p.x as i32, room_start.origin.1 as i32, p.y as i32))
    {
        if p.x == (room_start.origin.0 + room_start.width) as i32 {
            p.x -= 1;
        } else if p.z == (room_start.origin.2 + room_start.depth) as i32 {
            p.z -= 1;
        }
        return (room_start.id, room_end.id, p);
    }
    (
        room_start.id,
        room_end.id,
        Vector3::new(
            room_start.origin.0 as i32 - 1,
            room_start.origin.1 as i32,
            room_start.origin.2 as i32,
        ),
    )
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
