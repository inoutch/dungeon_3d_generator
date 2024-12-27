use crate::constants::{Direction4, DIRECTIONS};
use crate::voxel_map::VoxelMap;
use nalgebra::Vector3;
use rand::prelude::SliceRandom;
use rand::{Rng, SeedableRng};
use std::collections::{BTreeMap, HashMap, VecDeque};

pub struct CEDConfig {
    pub room_candidates: Vec<CEDRoomCandidate>,
    pub room_size: usize,
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
            },
        ];
        CEDConfig {
            room_candidates,
            room_size: 20,
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
}

impl Default for CEDRoomCandidate {
    fn default() -> Self {
        CEDRoomCandidate {
            width: 3,
            height: 1,
            depth: 3,
            exit_and_entrances: vec![],
        }
    }
}

pub struct CEDResult {
    pub room_candidates: Vec<CEDRoomCandidate>,
    pub room_candidate_indices: Vec<(usize, (i32, i32, i32))>,
    pub voxel_map: VoxelMap,
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
    let mut voxel_map = VoxelMap::new(0, 0, 0, 1, 1, 1);
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

    let mut room_candidate_indices = Vec::with_capacity(config.room_size);
    let mut cell_map: HashMap<Vector3<i32>, usize> = HashMap::new();
    let mut queue: VecDeque<(usize, Vector3<i32>)> = VecDeque::new();

    let first_room_candidate_index = rng.gen_range(0..config.room_candidates.len());
    let first_room_candidate = &optimized_room_candidates[first_room_candidate_index];
    queue.push_back((first_room_candidate_index, Vector3::new(0, 0, 0)));
    room_candidate_indices.push((first_room_candidate_index, (0, 0, 0)));
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

    while let Some((room_candidate_index, origin)) = queue.pop_front() {
        if room_candidate_indices.len() >= config.room_size {
            break;
        }

        let room_candidate = &optimized_room_candidates[room_candidate_index];
        let mut dirs = *DIRECTIONS;
        dirs.shuffle(&mut rng);

        // 次のエントランスを探す
        for (dir, (x, y, z)) in dirs.iter().filter_map(|dir| {
            room_candidate
                .exit_and_entrances
                .get(dir)
                .map(|result| (dir, result))
        }) {
            if room_candidate_indices.len() >= config.room_size {
                break;
            }

            let next_candidate_entrance_and_exit =
                origin + Vector3::new(*x, *y, *z) + dir.to_vec3();
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
            room_candidate_indices.push((
                *next_candidate_index,
                (
                    next_candidate_origin.x,
                    next_candidate_origin.y,
                    next_candidate_origin.z,
                ),
            ));
            queue.push_back((*next_candidate_index, next_candidate_origin));
        }
    }

    Ok(CEDResult {
        room_candidates: config.room_candidates,
        room_candidate_indices,
        voxel_map,
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
