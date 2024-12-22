use crate::btree_key_values::BTreeKeyValues;
use crate::constants::{Direction4, VoxelType, DIRECTIONS};
use crate::passage::Passage;
use crate::room::{Room, RoomId};
use nalgebra::Vector3;
use std::collections::{BTreeMap, BTreeSet, HashMap};

#[derive(Debug)]
pub enum VoxelMapError {
    Conflict,
    NoRoom(RoomId),
    Unreachable,
}

#[derive(Clone, Debug)]
pub struct VoxelMap {
    pub map: HashMap<Vector3<i32>, VoxelType>,
    start: Vector3<i32>,
    end: Vector3<i32>,
}

impl VoxelMap {
    pub fn new(x: i32, y: i32, z: i32, width: i32, height: i32, depth: i32) -> Self {
        Self {
            map: Default::default(),
            start: Vector3::new(x, y, z),
            end: Vector3::new(x + width, y + height, z + depth),
        }
    }

    pub fn get(&self, point: &Vector3<i32>) -> VoxelType {
        self.map.get(point).copied().unwrap_or(VoxelType::Wall)
    }

    pub fn add_room(&mut self, room: &Room) -> Result<(), VoxelMapError> {
        for y in -1..room.height as i32 {
            for z in 0..room.depth as i32 {
                for x in 0..room.width as i32 {
                    let p = Vector3::new(
                        x + room.origin.0 as i32,
                        y + room.origin.1 as i32,
                        z + room.origin.2 as i32,
                    );
                    if self.map.contains_key(&p) {
                        return Err(VoxelMapError::Conflict);
                    }
                    if y == -1 {
                        self.map.insert(p, VoxelType::RoomFloor(room.id));
                    } else if y == 0 {
                        self.map.insert(p, VoxelType::RoomBottomSpace(room.id));
                    } else {
                        self.map.insert(p, VoxelType::RoomSpace(room.id));
                    }
                }
            }
        }
        Ok(())
    }

    pub fn add_passage(
        &mut self,
        passage: &Passage,
        rooms: &BTreeMap<RoomId, Room>,
    ) -> Result<(), VoxelMapError> {
        // key = ParallelShiftAll > ParallelShift > Stair
        #[derive(Eq, PartialEq, Hash, Clone, Debug)]
        enum RouteKey {
            ParallelShift { movable_dirs: BTreeSet<Direction4> },
            Stair(Direction4),
        }
        impl RouteKey {
            // 同じ移動先を持って省略可能か
            fn contains(&self, other: &Self) -> bool {
                match other {
                    RouteKey::ParallelShift { movable_dirs } => match self {
                        RouteKey::ParallelShift {
                            movable_dirs: self_movable_dirs,
                        } => movable_dirs
                            .iter()
                            .all(|dir| self_movable_dirs.contains(dir)),
                        RouteKey::Stair(_) => false,
                    },
                    RouteKey::Stair(_) => self == other,
                }
            }
        }
        #[derive(Debug)]
        struct Route {
            key: RouteKey,
            point: Vector3<i32>,
            cost: i32,
            map: HashMap<Vector3<i32>, VoxelType>,
        }

        let start = Vector3::new(passage.start.0, passage.start.1, passage.start.2);
        let end_room = rooms
            .get(&passage.end_room_id)
            .ok_or(VoxelMapError::NoRoom(passage.end_room_id))?;

        let mut queue: BTreeKeyValues<i32, Route> = BTreeKeyValues::default(); // score, route
        let mut route_map: HashMap<Vector3<i32>, Vec<(RouteKey, i32)>> = HashMap::new(); // point, route_key, cost

        for start_dir in passage.start_dirs.iter() {
            let next_point = start + start_dir.to_vec3();
            let next_score = calc_score(end_room, &next_point, 0);
            queue.push_back(
                next_score,
                Route {
                    key: RouteKey::ParallelShift {
                        movable_dirs: DIRECTIONS
                            .iter()
                            .filter(|d| !start_dir.is_opposite(d))
                            .copied()
                            .collect(),
                    },
                    point: next_point,
                    cost: 0,
                    map: Default::default(),
                },
            );
            queue.push_back(
                next_score,
                Route {
                    key: RouteKey::Stair(*start_dir),
                    point: next_point,
                    cost: 0,
                    map: Default::default(),
                },
            );
        }

        while let Some(mut route) = queue.pop_first_back() {
            if route.point.x < self.start.x
                || route.point.y < self.start.y
                || route.point.z < self.start.z
                || self.end.x <= route.point.x
                || self.end.y <= route.point.y
                || self.end.z <= route.point.z
            {
                continue;
            }

            if self.map.get(&route.point) == Some(&VoxelType::RoomBottomSpace(end_room.id)) {
                for (key, value) in route.map.into_iter() {
                    self.map.insert(key, value);
                }
                return Ok(());
            }

            // 既に登録されているルートよりも最短距離があればそちらを利用し処理を省略
            if let Some(exist_routes) = route_map.get_mut(&route.point) {
                if exist_routes.len() > 10 {
                    continue;
                }
                let mut omit = false;
                let mut replace_index: Option<usize> = None;
                for (index, (exist_route_key, exist_cost)) in exist_routes.iter().enumerate() {
                    if exist_route_key.contains(&route.key) && *exist_cost <= route.cost {
                        // 既により良い探索経路が登録されていた場合
                        omit = true;
                        break;
                    }
                    if route.key.contains(exist_route_key) && route.cost < *exist_cost {
                        // 今回のものがより良い探索経路の場合
                        replace_index = Some(index);
                        break;
                    }
                }
                if omit {
                    continue;
                }
                if let Some(index) = replace_index {
                    exist_routes[index].0 = route.key.clone();
                    exist_routes[index].1 = route.cost;
                } else {
                    route_map
                        .entry(route.point.clone_owned())
                        .or_default()
                        .push((route.key.clone(), route.cost));
                }
            } else {
                route_map
                    .entry(route.point.clone_owned())
                    .or_default()
                    .push((route.key.clone(), route.cost));
            }

            match &route.key {
                RouteKey::ParallelShift { movable_dirs } => {
                    // コンフリクトしていないか確認
                    // 通路として塞がれていないか確認
                    if !add_passage(&route.point, passage.height, &self.map, &mut route.map) {
                        continue;
                    }

                    for movable_dir in movable_dirs {
                        // 平行移動の探索を予約
                        let next_point = route.point + movable_dir.to_vec3();
                        let next_const = calc_score(end_room, &next_point, route.cost + 1);
                        queue.push_back(
                            next_const,
                            Route {
                                key: RouteKey::ParallelShift {
                                    movable_dirs: DIRECTIONS
                                        .iter()
                                        .filter(|d| !movable_dir.is_opposite(d))
                                        .copied()
                                        .collect(),
                                },
                                point: next_point,
                                cost: next_const,
                                map: route.map.clone(),
                            },
                        );
                        // 階段の探索を予約
                        queue.push_back(
                            next_const,
                            Route {
                                key: RouteKey::Stair(*movable_dir),
                                point: next_point,
                                cost: next_const,
                                map: route.map.clone(),
                            },
                        );
                    }
                }
                RouteKey::Stair(direction) => {
                    // コンフリクトしていないか確認
                    // 階段として塞がれていないか確認
                    if !add_stair(
                        &route.point,
                        passage.height,
                        direction,
                        &self.map,
                        &mut route.map,
                    ) {
                        continue;
                    }

                    // 平行移動の探索を予約
                    let next_point = route.point + direction.to_vec3() + Vector3::new(0, 1, 0);
                    let next_const = calc_score(end_room, &next_point, route.cost + 1);
                    queue.push_back(
                        next_const,
                        Route {
                            key: RouteKey::ParallelShift {
                                movable_dirs: DIRECTIONS
                                    .iter()
                                    .filter(|d| !direction.is_opposite(d))
                                    .copied()
                                    .collect(),
                            },
                            point: next_point,
                            cost: next_const,
                            map: route.map.clone(),
                        },
                    );
                    // 階段の探索を予約
                    queue.push_back(
                        next_const,
                        Route {
                            key: RouteKey::Stair(*direction),
                            point: next_point,
                            cost: next_const,
                            map: route.map.clone(),
                        },
                    );
                }
            };
        }

        Err(VoxelMapError::Unreachable)
    }
}

// 部屋までの距離コスト計算
fn calc_score(room: &Room, start: &Vector3<i32>, cost: i32) -> i32 {
    let center = room.center();
    let d = (Vector3::new(center.0 as i32, room.origin.1 as i32, center.2 as i32) - *start).abs();
    (d.x + d.y + d.z) * 10 + cost
}

#[inline]
fn add_passage(
    point: &Vector3<i32>,
    height: i32,
    readonly_map: &HashMap<Vector3<i32>, VoxelType>,
    writable_map: &mut HashMap<Vector3<i32>, VoxelType>,
) -> bool {
    let ground_point = point + Vector3::new(0, -1, 0);
    let ground = readonly_map
        .get(&ground_point)
        .or_else(|| writable_map.get(&ground_point));
    if ground.is_some() && ground != Some(&VoxelType::PassageFloor) {
        return false;
    }
    writable_map.insert(ground_point, VoxelType::PassageFloor);

    for y in 0..height {
        let space_point = point + Vector3::new(0, y, 0);
        let space = readonly_map
            .get(&space_point)
            .or_else(|| writable_map.get(&space_point));
        if space.is_some() && space != Some(&VoxelType::PassageSpace) {
            return false;
        }

        writable_map.insert(space_point, VoxelType::PassageSpace);
    }
    true
}

#[inline]
fn add_stair(
    point: &Vector3<i32>,
    height: i32,
    direction: &Direction4,
    readonly_map: &HashMap<Vector3<i32>, VoxelType>,
    writable_map: &mut HashMap<Vector3<i32>, VoxelType>,
) -> bool {
    let ground = readonly_map.get(point).or_else(|| writable_map.get(point));
    if ground.is_some() {
        return false;
    }
    writable_map.insert(point.clone_owned(), VoxelType::PassageStair(*direction));

    for y in 0..height {
        let space_point = point + Vector3::new(0, y + 1, 0);
        let space = readonly_map
            .get(&space_point)
            .or_else(|| writable_map.get(&space_point));
        if space.is_some() && space != Some(&VoxelType::PassageSpace) {
            return false;
        }

        writable_map.insert(space_point, VoxelType::PassageSpace);
    }
    true
}
