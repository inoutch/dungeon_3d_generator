use crate::create_start::create_start;
use crate::delaunary_3d::Delaunay3D;
use crate::passage::Passage;
use crate::room::{Room, RoomId};
use crate::room_connection::RoomConnection;
use crate::voxel_map::{VoxelMap, VoxelMapError};
use nalgebra::Vector3;
use pathfinding::prelude::kruskal;
use rand::{Rng, SeedableRng};
use std::cmp::Ordering;
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
    pub room_margin_x: u32,
    pub room_margin_y: u32,
    pub room_margin_z: u32,
    pub passage_height: u32,
    pub margin_for_bounds: u32, // Margin used to specify a range for all elements to fit, such as passages
}

impl Default for Dungeon3DGeneratorConfig {
    fn default() -> Self {
        Dungeon3DGeneratorConfig {
            width: 32,
            height: 10,
            depth: 32,
            seed: None,
            room_hierarchy: 3,
            room_width_range: 5..=10,
            room_height_range: 2..=2,
            room_depth_range: 5..=10,
            room_margin_x: 4,
            room_margin_y: 1,
            room_margin_z: 4,
            passage_height: 2,
            margin_for_bounds: 4,
        }
    }
}

#[derive(Debug)]
pub struct Dungeon3DGeneratorResult {
    pub rooms: BTreeMap<RoomId, Room>,
    pub voxel_map: VoxelMap,
    pub passages: Vec<Passage>,
}

#[derive(Debug)]
pub enum Dungeon3DGeneratorError {
    NarrowWidthOrRoomWidthTooLarge,
    NarrowDepthOrRoomDepthTooLarge,
    NarrowHeightOrRoomHierarchyTooSmall,
    VoxelMapError(VoxelMapError),
}

pub fn generate_dungeon_3d(
    mut config: Dungeon3DGeneratorConfig,
) -> Result<Dungeon3DGeneratorResult, Dungeon3DGeneratorError> {
    config.room_margin_x = config.room_margin_x.max(1);
    config.room_margin_y = config.room_margin_y.max(1);
    config.room_margin_z = config.room_margin_z.max(1);

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
    let mut voxel_map = VoxelMap::new(
        -(config.margin_for_bounds as i32),
        -(config.margin_for_bounds as i32),
        -(config.margin_for_bounds as i32),
        (config.width + config.margin_for_bounds) as i32,
        (config.height + config.margin_for_bounds) as i32,
        (config.depth + config.margin_for_bounds) as i32,
    );
    for (_, room) in rooms.iter() {
        voxel_map
            .add_room(room)
            .map_err(Dungeon3DGeneratorError::VoxelMapError)?;
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

    #[derive(Eq, PartialEq)]
    struct RoomConnectionKey {
        room_0_id: RoomId,
        room_1_id: RoomId,
    }
    impl RoomConnectionKey {
        pub fn new(room_0_id: RoomId, room_1_id: RoomId) -> Self {
            if room_0_id.inner() < room_1_id.inner() {
                return RoomConnectionKey {
                    room_0_id,
                    room_1_id,
                };
            }
            RoomConnectionKey {
                room_0_id: room_1_id,
                room_1_id: room_0_id,
            }
        }
    }
    impl PartialOrd for RoomConnectionKey {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            Some(self.cmp(other))
        }
    }
    impl Ord for RoomConnectionKey {
        fn cmp(&self, other: &Self) -> Ordering {
            if self.room_0_id == other.room_0_id {
                self.room_1_id.cmp(&other.room_1_id)
            } else {
                self.room_0_id.cmp(&other.room_0_id)
            }
        }
    }
    let necessary_room_connections = kruskal(&weighted_edges)
        .map(|(room0_id, room1_id, _)| {
            (
                RoomConnectionKey::new(*room0_id, *room1_id),
                Rc::clone(
                    room_connection_map
                        .get(room0_id)
                        .unwrap()
                        .get(room1_id)
                        .unwrap(),
                ),
            )
        })
        .collect::<BTreeMap<_, _>>();
    // create passages
    let mut passages = Vec::new();
    for (_, room_connection) in necessary_room_connections.iter() {
        let r0 = rooms.get(&room_connection.room0_id).unwrap();
        let r1 = rooms.get(&room_connection.room1_id).unwrap();
        let (start_room_id, end_room_id, start, dirs) = create_start(r0, r1);
        passages.push(Passage {
            cells: Vec::new(),
            start: (start.x, start.y, start.z),
            start_dirs: dirs,
            start_room_id,
            end_room_id,
            height: config.passage_height as i32,
        });
    }
    for passage in passages.iter() {
        voxel_map
            .add_passage(passage, &rooms)
            .map_err(Dungeon3DGeneratorError::VoxelMapError)?;
    }

    let delaunay = Delaunay3D::new(
        rooms
            .values()
            .map(|room| {
                let center = room.center();
                (room.id, Vector3::new(center.0, center.1, center.2))
            })
            .collect(),
    );
    let additional_room_connections = delaunay
        .edges
        .iter()
        .map(|edge| RoomConnection {
            room0_id: *delaunay.id_map.get(&edge.u).unwrap(),
            room1_id: *delaunay.id_map.get(&edge.v).unwrap(),
            squared_length: (edge.u.position - edge.v.position).norm_squared(),
        })
        .collect::<Vec<_>>();

    for room_connection in additional_room_connections {
        if rng.gen_bool(0.3)
            && !necessary_room_connections.contains_key(&RoomConnectionKey::new(
                room_connection.room0_id,
                room_connection.room1_id,
            ))
        {
            let r0 = rooms.get(&room_connection.room0_id).unwrap();
            let r1 = rooms.get(&room_connection.room1_id).unwrap();
            let (start_room_id, end_room_id, start, dirs) = create_start(r0, r1);
            let passage = Passage {
                cells: Vec::new(),
                start: (start.x, start.y, start.z),
                start_dirs: dirs,
                start_room_id,
                end_room_id,
                height: config.passage_height as i32,
            };
            if voxel_map.add_passage(&passage, &rooms).is_ok() {
                passages.push(passage);
            }
        }
    }

    Ok(Dungeon3DGeneratorResult {
        rooms,
        voxel_map,
        passages,
    })
}

#[cfg(test)]
mod tests {
    use crate::generate_drd::{generate_dungeon_3d, Dungeon3DGeneratorConfig};

    #[test]
    fn test_default_generate() {
        let result = generate_dungeon_3d(Dungeon3DGeneratorConfig {
            seed: Some(0),
            ..Default::default()
        })
        .unwrap();
        insta::assert_debug_snapshot!(result.passages);
        insta::assert_debug_snapshot!(result.rooms);
    }
}
