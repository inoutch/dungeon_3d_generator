use crate::constants::{Direction4, DIRECTIONS};
use crate::room::RoomId;
use nalgebra::Vector3;
use rand::prelude::SliceRandom;
use rand::{Rng, SeedableRng};
use std::collections::{BTreeMap, BTreeSet, HashMap, VecDeque};

pub struct CEDConfig {
    pub room_candidates: Vec<CEDRoomCandidate>,
    pub room_size_max: usize,
    pub seed: Option<u64>, // Seed value for random dungeon generation
}

impl Default for CEDConfig {
    fn default() -> Self {
        let room_candidates = vec![
            // square
            CEDRoomCandidate {
                width: 3,
                height: 1,
                depth: 3,
                exit_and_entrances: vec![
                    ((0, 0, 1), Direction4::Left),
                    ((2, 0, 1), Direction4::Right),
                    ((1, 0, 2), Direction4::Near),
                    ((1, 0, 0), Direction4::Far),
                ],
                can_be_terminal: true,
            },
            // T0
            CEDRoomCandidate {
                width: 3,
                height: 1,
                depth: 2,
                exit_and_entrances: vec![
                    ((0, 0, 1), Direction4::Left),
                    ((2, 0, 1), Direction4::Right),
                    ((1, 0, 1), Direction4::Near),
                ],
                can_be_terminal: true,
            },
            // T1
            CEDRoomCandidate {
                width: 3,
                height: 1,
                depth: 2,
                exit_and_entrances: vec![
                    ((0, 0, 0), Direction4::Left),
                    ((2, 0, 0), Direction4::Right),
                    ((1, 0, 0), Direction4::Far),
                ],
                can_be_terminal: true,
            },
            // T2
            CEDRoomCandidate {
                width: 2,
                height: 1,
                depth: 3,
                exit_and_entrances: vec![
                    ((1, 0, 2), Direction4::Near),
                    ((1, 0, 0), Direction4::Far),
                    ((0, 0, 1), Direction4::Left),
                ],
                can_be_terminal: true,
            },
            // T3
            CEDRoomCandidate {
                width: 2,
                height: 1,
                depth: 3,
                exit_and_entrances: vec![
                    ((0, 0, 2), Direction4::Near),
                    ((0, 0, 0), Direction4::Far),
                    ((1, 0, 1), Direction4::Right),
                ],
                can_be_terminal: true,
            },
            // Stair left-right
            CEDRoomCandidate {
                width: 1,
                height: 2,
                depth: 1,
                exit_and_entrances: vec![
                    ((0, 0, 0), Direction4::Left),
                    ((0, 1, 0), Direction4::Right),
                ],
                can_be_terminal: false,
            },
            // Stair right-left
            CEDRoomCandidate {
                width: 1,
                height: 2,
                depth: 1,
                exit_and_entrances: vec![
                    ((0, 1, 0), Direction4::Left),
                    ((0, 0, 0), Direction4::Right),
                ],
                can_be_terminal: false,
            },
            // Stair far-near
            CEDRoomCandidate {
                width: 1,
                height: 2,
                depth: 1,
                exit_and_entrances: vec![
                    ((0, 0, 0), Direction4::Near),
                    ((0, 1, 0), Direction4::Far),
                ],
                can_be_terminal: false,
            },
            // Stair far-near
            CEDRoomCandidate {
                width: 1,
                height: 2,
                depth: 1,
                exit_and_entrances: vec![
                    ((0, 1, 0), Direction4::Near),
                    ((0, 0, 0), Direction4::Far),
                ],
                can_be_terminal: false,
            },
        ];
        CEDConfig {
            room_candidates,
            room_size_max: 20,
            seed: None,
        }
    }
}

#[derive(Debug)]
pub struct CEDRoomCandidate {
    pub width: u32,
    pub height: u32,
    pub depth: u32,
    pub exit_and_entrances: Vec<((i32, i32, i32), Direction4)>, // x, y, z
    pub can_be_terminal: bool,
}

impl Default for CEDRoomCandidate {
    fn default() -> Self {
        CEDRoomCandidate {
            width: 3,
            height: 1,
            depth: 3,
            exit_and_entrances: vec![],
            can_be_terminal: true,
        }
    }
}

pub struct RoomCandidateEntity {
    pub index: usize,
    pub origin: (i32, i32, i32),
}

pub struct CEDResult {
    pub room_candidates: Vec<CEDRoomCandidate>,
    pub room_candidate_entities: BTreeMap<RoomId, RoomCandidateEntity>,
    pub room_candidate_connections: BTreeMap<RoomId, BTreeSet<RoomId>>,
}

#[derive(Debug)]
pub enum CEDError {
    InvalidRoomCandidateExitAndEntrance { index: usize },
}

#[derive(Debug)]
struct OptimizedRoomCandidate {
    pub width: u32,
    pub height: u32,
    pub depth: u32,
    pub exit_and_entrances: BTreeMap<Direction4, (i32, i32, i32)>, // x, y, z
}

pub fn generate_ced(config: CEDConfig) -> Result<CEDResult, CEDError> {
    if let Some((index, _)) =
        config
            .room_candidates
            .iter()
            .enumerate()
            .find(|(_, room_candidate)| {
                room_candidate
                    .exit_and_entrances
                    .iter()
                    .any(|((x, y, z), dir)| {
                        *y < 0
                            || room_candidate.height as i32 <= *y
                            || !validate_dir_of_room_candidate(
                                *x,
                                *z,
                                room_candidate.width,
                                room_candidate.depth,
                                *dir,
                            )
                    })
            })
    {
        return Err(CEDError::InvalidRoomCandidateExitAndEntrance { index });
    }

    let optimized_room_candidates = config
        .room_candidates
        .iter()
        .map(|room_candidate| OptimizedRoomCandidate {
            width: room_candidate.width,
            height: room_candidate.height,
            depth: room_candidate.depth,
            exit_and_entrances: room_candidate
                .exit_and_entrances
                .iter()
                .map(|((x, y, z), dir)| (*dir, (*x, *y, *z)))
                .collect(),
        })
        .collect::<Vec<_>>();

    let mut rng: rand::rngs::StdRng = config
        .seed
        .map(SeedableRng::seed_from_u64)
        .unwrap_or_else(rand::rngs::StdRng::from_entropy);

    let mut room_candidates_by_dir: HashMap<Direction4, Vec<(usize, (i32, i32, i32))>> =
        HashMap::new();
    for (dir, (index, (x, y, z))) in config
        .room_candidates
        .iter()
        .enumerate()
        .flat_map(|(i, room_candidate)| {
            room_candidate
                .exit_and_entrances
                .iter()
                .map(|((x, y, z), dir)| (*dir, (i, (*x, *y, *z))))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>()
    {
        room_candidates_by_dir
            .entry(dir)
            .or_default()
            .push((index, (x, y, z)));
    }

    struct Node {
        room_candidate_index: usize,
        origin: Vector3<i32>,
        from_room_id: Option<RoomId>,
    }

    let mut current_room_id = RoomId::first();
    let mut room_candidate_entities = BTreeMap::new();
    let mut room_candidate_connections: BTreeMap<RoomId, BTreeSet<RoomId>> = BTreeMap::new();
    let mut cell_map: HashMap<Vector3<i32>, usize> = HashMap::new();
    let mut queue: VecDeque<Node> = VecDeque::new();

    let first_room_candidate_index = rng.gen_range(0..config.room_candidates.len());
    let first_room_candidate = &optimized_room_candidates[first_room_candidate_index];
    queue.push_back(Node {
        room_candidate_index: first_room_candidate_index,
        origin: Vector3::new(0, 0, 0),
        from_room_id: None,
    });
    room_candidate_entities.insert(
        current_room_id.gen_id(),
        RoomCandidateEntity {
            index: first_room_candidate_index,
            origin: (0, 0, 0),
        },
    );
    for x in 0..first_room_candidate.width {
        for y in 0..first_room_candidate.height {
            for z in 0..first_room_candidate.depth {
                cell_map.insert(
                    Vector3::new(x as i32, y as i32, z as i32),
                    first_room_candidate_index,
                );
            }
        }
    }

    while let Some(node) = queue.pop_front() {
        if room_candidate_entities.len() >= config.room_size_max {
            break;
        }

        let room_candidate = &optimized_room_candidates[node.room_candidate_index];
        let mut dirs = *DIRECTIONS;
        dirs.shuffle(&mut rng);

        // 次のエントランスを探す
        for (dir, (x, y, z)) in dirs.iter().filter_map(|dir| {
            room_candidate
                .exit_and_entrances
                .get(dir)
                .map(|result| (dir, result))
        }) {
            if room_candidate_entities.len() >= config.room_size_max {
                break;
            }

            let next_candidate_entrance_and_exit =
                node.origin + Vector3::new(*x, *y, *z) + dir.to_vec3();
            let next_candidate_dir = dir.opposite();
            let Some(next_candidates) = room_candidates_by_dir.get_mut(&next_candidate_dir) else {
                continue;
            };
            next_candidates.shuffle(&mut rng);

            let Some((next_candidate_index, next_candidate_entrance_and_exit_offset)) =
                next_candidates.iter().find(|(index, _)| {
                    let room_candidate = &optimized_room_candidates[*index];
                    let entrance_and_exit = room_candidate
                        .exit_and_entrances
                        .get(&next_candidate_dir)
                        .map(|(x, y, z)| Vector3::new(*x, *y, *z))
                        .unwrap();
                    !any_cell(room_candidate, |p| {
                        cell_map.contains_key(
                            &(next_candidate_entrance_and_exit - entrance_and_exit + p),
                        )
                    })
                })
            else {
                continue;
            };

            let next_room_id = current_room_id.gen_id();
            let next_candidate_room = &optimized_room_candidates[*next_candidate_index];
            let next_candidate_origin = next_candidate_entrance_and_exit
                - Vector3::new(
                    next_candidate_entrance_and_exit_offset.0,
                    next_candidate_entrance_and_exit_offset.1,
                    next_candidate_entrance_and_exit_offset.2,
                );
            for x in 0..next_candidate_room.width {
                for y in 0..next_candidate_room.height {
                    for z in 0..next_candidate_room.depth {
                        cell_map.insert(
                            next_candidate_origin + Vector3::new(x as i32, y as i32, z as i32),
                            *next_candidate_index,
                        );
                    }
                }
            }
            if let Some(from_room_id) = node.from_room_id {
                room_candidate_connections
                    .entry(from_room_id)
                    .or_default()
                    .insert(next_room_id);
                room_candidate_connections
                    .entry(next_room_id)
                    .or_default()
                    .insert(from_room_id);
            }
            queue.push_back(Node {
                room_candidate_index: *next_candidate_index,
                origin: next_candidate_origin,
                from_room_id: Some(next_room_id),
            });
            room_candidate_entities.insert(
                next_room_id,
                RoomCandidateEntity {
                    index: *next_candidate_index,
                    origin: (
                        next_candidate_origin.x,
                        next_candidate_origin.y,
                        next_candidate_origin.z,
                    ),
                },
            );
        }
    }

    let mut queue = room_candidate_entities
        .keys()
        .cloned()
        .collect::<VecDeque<_>>();
    while let Some(room_id) = queue.pop_front() {
        let Some(room_ids) = room_candidate_connections.get(&room_id) else {
            continue;
        };
        if room_ids.len() >= 2
            || config.room_candidates[room_candidate_entities.get(&room_id).unwrap().index]
                .can_be_terminal
        {
            continue;
        }
        room_candidate_entities.remove(&room_id);
        for room_id in room_candidate_connections.remove(&room_id).unwrap() {
            queue.push_back(room_id);
        }
        for (_room_id, connections) in room_candidate_connections.iter_mut() {
            connections.retain(|room_id| room_candidate_entities.contains_key(room_id));
        }
    }

    Ok(CEDResult {
        room_candidates: config.room_candidates,
        room_candidate_entities,
        room_candidate_connections,
    })
}

fn any_cell<F>(room_candidate: &OptimizedRoomCandidate, f: F) -> bool
where
    F: Fn(&Vector3<i32>) -> bool,
{
    for x in 0..room_candidate.width {
        for y in 0..room_candidate.height {
            for z in 0..room_candidate.depth {
                if f(&Vector3::new(x as i32, y as i32, z as i32)) {
                    return true;
                }
            }
        }
    }
    false
}

fn validate_dir_of_room_candidate(x: i32, z: i32, width: u32, depth: u32, dir: Direction4) -> bool {
    (x == 0 && dir == Direction4::Left)
        || (x == width as i32 - 1 && dir == Direction4::Right)
        || (z == 0 && dir == Direction4::Far)
        || (z == depth as i32 - 1 && dir == Direction4::Near)
}
