use super::*;

pub(super) fn index(x: usize, y: usize, z: usize) -> usize {
    x | y << 8 | z << 4
}

/// `Camera.createProjectionMatrixForCulling`: JOML's `perspective` with a
/// zero-to-one depth range, the vertical field of view the larger of the
/// camera's and the option's.
pub fn culling_projection(fov_degrees: f32, aspect: f32, far: f32) -> [[f32; 4]; 4] {
    let near = 0.05_f32;
    let h = (fov_degrees.to_radians() * 0.5).tan();
    [
        [1.0 / (h * aspect), 0.0, 0.0, 0.0],
        [0.0, 1.0 / h, 0.0, 0.0],
        [0.0, 0.0, far / (near - far), -1.0],
        [0.0, 0.0, far * near / (near - far), 0.0],
    ]
}

pub(super) fn step(key: SectionKey, direction: Direction) -> SectionKey {
    let (dx, dy, dz) = direction.step();
    (key.0 + dx, key.1 + dy, key.2 + dz)
}

/// `ChunkTrackingView.isInViewDistance`.
pub(super) fn in_view_distance(center: (i32, i32), distance: i32, chunk: (i32, i32)) -> bool {
    let delta = |a: i32, b: i32| i64::from(((a - b).abs() - 1).max(0));
    let (dx, dz) = (delta(chunk.0, center.0), delta(chunk.1, center.1));
    dx * dx + dz * dz < i64::from(distance) * i64::from(distance)
}

/// The flood's nodes over the view area (`ViewArea`): one slot per section
/// within the view distance of the camera's column, so a node is found by
/// arithmetic rather than by hashing its key.
pub(super) struct Nodes {
    /// The view area's lowest corner.
    low: SectionKey,
    /// Columns per row, and sections per column.
    width: usize,
    height: usize,
    slots: Vec<Option<Node>>,
    /// Whether each column's chunk has arrived, asked once per column.
    loaded: Vec<bool>,
}

impl Nodes {
    fn new(world: &GraphWorld<'_>) -> Self {
        let camera = world.camera_section();
        let reach = world.view_distance.max(0);
        let low = (camera.0 - reach, world.min_section_y, camera.2 - reach);
        let width = (2 * reach + 1) as usize;
        let height = (world.max_section_y - world.min_section_y + 1).max(0) as usize;
        let loaded = (0..width * width)
            .map(|column| {
                let (dx, dz) = (column / width, column % width);
                (world.loaded)(low.0 + dx as i32, low.2 + dz as i32)
            })
            .collect();
        Self {
            low,
            width,
            height,
            slots: vec![None; width * width * height],
            loaded,
        }
    }

    /// The column of a key inside the view area.
    fn column(&self, key: SectionKey) -> usize {
        (key.0 - self.low.0) as usize * self.width + (key.2 - self.low.2) as usize
    }

    /// The slot of a key inside the view area (`GraphWorld::exists`).
    fn slot(&self, key: SectionKey) -> usize {
        self.column(key) * self.height + (key.1 - self.low.1) as usize
    }

    pub(super) fn get(&self, key: SectionKey) -> Option<Node> {
        self.slots[self.slot(key)]
    }

    fn get_mut(&mut self, key: SectionKey) -> &mut Option<Node> {
        let slot = self.slot(key);
        &mut self.slots[slot]
    }

    fn loaded(&self, key: SectionKey) -> bool {
        self.loaded[self.column(key)]
    }
}

/// `SectionOcclusionGraph`'s full update: flood from the camera's section
/// through the sections whose faces see each other, and return every
/// non-empty section reached in a loaded chunk, in the order reached.
pub fn visible_sections(world: &GraphWorld<'_>) -> Vec<SectionKey> {
    let mut nodes = Nodes::new(world);
    let mut queue: VecDeque<SectionKey> = VecDeque::new();
    for (key, node) in initial_nodes(world) {
        *nodes.get_mut(key) = Some(node);
        queue.push_back(key);
    }
    let camera = world.camera_section();
    let mut reached = Vec::new();
    while let Some(key) = queue.pop_front() {
        let Some(node) = nodes.get(key) else {
            continue;
        };
        if !nodes.loaded(key) {
            continue;
        }
        let state = (world.section)(key);
        if state != SectionState::Empty {
            reached.push(key);
        }
        let distant = [key.0 - camera.0, key.1 - camera.1, key.2 - camera.2]
            .iter()
            .any(|delta| delta.abs() > 60 / 16);
        for direction in DIRECTIONS {
            let Some(next) = world.relative(camera, key, direction) else {
                continue;
            };
            if world.smart_cull && node.directions & (1 << direction.opposite() as u8) != 0 {
                continue;
            }
            if world.smart_cull && node.sources != 0 && !sees_through(state, node, direction) {
                continue;
            }
            if world.smart_cull && distant && !ray_clear(world, camera, &nodes, key, direction) {
                continue;
            }
            match nodes.get_mut(next) {
                Some(existing) => existing.sources |= 1 << direction as u8,
                empty @ None => {
                    *empty = Some(Node {
                        sources: 1 << direction as u8,
                        directions: node.directions | 1 << direction as u8,
                    });
                    queue.push_back(next);
                }
            }
        }
    }
    reached
}

/// Whether a section lets the flood through from a side it was reached
/// through towards `direction` (`facesCanSeeEachother`).
pub(super) fn sees_through(state: SectionState, node: Node, direction: Direction) -> bool {
    let visibility = match state {
        SectionState::Uncompiled => VisibilitySet::NONE,
        SectionState::Compiled(visibility) => visibility,
        SectionState::Empty => VisibilitySet::ALL,
    };
    DIRECTIONS.iter().any(|source| {
        node.sources & (1 << *source as u8) != 0
            && visibility.visibility_between(source.opposite(), direction)
    })
}

/// `initializeQueueForFullUpdate`: the camera's section, or with the camera
/// above or below the world, the nearest layer's sections nearest first,
/// each as if reached from beyond the world towards the camera's column.
pub(super) fn initial_nodes(world: &GraphWorld<'_>) -> Vec<(SectionKey, Node)> {
    let camera = world.camera_section();
    if world.exists(camera, camera) {
        return vec![(camera, Node::default())];
    }
    let below = camera.1 < world.min_section_y;
    let (layer, source) = if below {
        (world.min_section_y, Direction::Up)
    } else {
        (world.max_section_y, Direction::Down)
    };
    let mut layer_nodes: Vec<(SectionKey, Node)> = Vec::new();
    for dx in -world.view_distance..=world.view_distance {
        for dz in -world.view_distance..=world.view_distance {
            let key = (camera.0 + dx, layer, camera.2 + dz);
            if !world.exists(camera, key)
                || !in_view_distance((camera.0, camera.2), world.view_distance, (key.0, key.2))
            {
                continue;
            }
            let mut directions = 1 << source as u8;
            if dx > 0 {
                directions |= 1 << Direction::East as u8;
            } else if dx < 0 {
                directions |= 1 << Direction::West as u8;
            }
            if dz > 0 {
                directions |= 1 << Direction::South as u8;
            } else if dz < 0 {
                directions |= 1 << Direction::North as u8;
            }
            let node = Node {
                sources: 1 << source as u8,
                directions,
            };
            layer_nodes.push((key, node));
        }
    }
    let block = world.camera.map(|value| value.floor() as i64);
    let distance = |(key, _): &(SectionKey, Node)| {
        let centre = [key.0, key.1, key.2].map(|value| i64::from(value) * 16 + 8);
        (0..3)
            .map(|axis| (centre[axis] - block[axis]).pow(2))
            .sum::<i64>()
    };
    layer_nodes.sort_by_key(distance);
    layer_nodes
}

/// The ray test the flood makes for sections far from the camera: step
/// from the section's corner nearest the camera towards the camera, one
/// section diagonal at a time, and give up if a step lands in a section the
/// flood has not reached.
pub(super) fn ray_clear(
    world: &GraphWorld<'_>,
    camera: SectionKey,
    nodes: &Nodes,
    key: SectionKey,
    direction: Direction,
) -> bool {
    let centre = [camera.0, camera.1, camera.2].map(|value| value * 16 + 8);
    let origin = [key.0, key.1, key.2].map(|value| value * 16);
    let axis = direction.axis() as usize;
    let corner: [f64; 3] = std::array::from_fn(|index| {
        let beyond = if index == axis {
            centre[index] > origin[index]
        } else {
            centre[index] < origin[index]
        };
        f64::from(origin[index] + if beyond { 16 } else { 0 })
    });
    let towards: [f64; 3] = std::array::from_fn(|index| world.camera[index] - corner[index]);
    let length = towards
        .iter()
        .map(|value| value * value)
        .sum::<f64>()
        .sqrt();
    let diagonal = (3.0_f64.sqrt() * 16.0).ceil();
    let step: [f64; 3] = towards.map(|value| value / length * diagonal);
    let mut check = corner;
    let min_y = f64::from(world.min_section_y * 16);
    let max_y = f64::from(world.max_section_y * 16 + 15);
    while (0..3)
        .map(|index| (check[index] - world.camera[index]).powi(2))
        .sum::<f64>()
        > 3600.0
    {
        for index in 0..3 {
            check[index] += step[index];
        }
        if check[1] > max_y || check[1] < min_y {
            break;
        }
        let section = check.map(|value| (value.floor() as i32).div_euclid(16));
        let section = (section[0], section[1], section[2]);
        if !world.exists(camera, section) || nodes.get(section).is_none() {
            return false;
        }
    }
    true
}
