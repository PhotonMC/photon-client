use super::*;

impl Part {
    pub(crate) fn new(
        name: &'static str,
        cubes: Vec<Cube>,
        offset: [f32; 3],
        children: Vec<Part>,
    ) -> Self {
        Self::posed(name, cubes, offset, [0.0; 3], [1.0; 3], children)
    }

    /// A part with a pose of its own: `PartPose.offsetAndRotation`, and
    /// `scaled`.
    pub(crate) fn posed(
        name: &'static str,
        cubes: Vec<Cube>,
        offset: [f32; 3],
        rotation: [f32; 3],
        scale: [f32; 3],
        children: Vec<Part>,
    ) -> Self {
        let initial = PartPose {
            offset,
            rotation,
            scale,
        };
        Self {
            name,
            cubes,
            children,
            initial,
            x: offset[0],
            y: offset[1],
            z: offset[2],
            x_rot: rotation[0],
            y_rot: rotation[1],
            z_rot: rotation[2],
            scale,
            visible: true,
            skip_draw: false,
        }
    }

    /// `PartDefinition.bake`: a part made from a layer's definition, in a
    /// texture `texture` big.
    pub fn baked(definition: &photon_data::entity_model::ModelPart, texture: (u32, u32)) -> Self {
        use photon_data::entity_model::Face;
        let cubes = definition
            .cubes
            .iter()
            .map(|cube| {
                let faces: Vec<Direction> = cube
                    .faces
                    .iter()
                    .map(|face| match face {
                        Face::Down => Direction::Down,
                        Face::East => Direction::East,
                        Face::North => Direction::North,
                        Face::South => Direction::South,
                        Face::Up => Direction::Up,
                        Face::West => Direction::West,
                    })
                    .collect();
                Cube::grown(
                    (cube.uv[0] as i32, cube.uv[1] as i32),
                    cube.origin,
                    cube.dimensions,
                    cube.grow,
                    cube.mirror,
                    (
                        texture.0 as f32 * cube.uv_scale[0],
                        texture.1 as f32 * cube.uv_scale[1],
                    ),
                    &faces,
                )
            })
            .collect();
        Self::posed(
            definition.name,
            cubes,
            definition.pose.position,
            definition.pose.rotation,
            definition.pose.scale,
            definition
                .children
                .iter()
                .map(|child| Self::baked(child, texture))
                .collect(),
        )
    }

    pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn children(&self) -> &[Part] {
        &self.children
    }

    /// `ModelPart.getChild`, by name.
    /// The child at this place in the list (`children()` order).
    pub fn child_at_mut(&mut self, index: usize) -> &mut Part {
        &mut self.children[index]
    }

    pub fn child_mut(&mut self, name: &str) -> Option<&mut Part> {
        self.children.iter_mut().find(|child| child.name == name)
    }

    /// `Model.resetPose`: every part back where the mesh put it.
    pub fn reset(&mut self) {
        [self.x, self.y, self.z] = self.initial.offset;
        [self.x_rot, self.y_rot, self.z_rot] = self.initial.rotation;
        self.scale = self.initial.scale;
        for child in &mut self.children {
            child.reset();
        }
    }

    pub(crate) fn child(&mut self, name: &str) -> &mut Part {
        self.children
            .iter_mut()
            .find(|child| child.name == name)
            .expect("the player model has this part")
    }

    /// `ModelPart.rotateBy`: this part's rotation turned further by a quaternion,
    /// through the matrix and back to the three angles.
    pub fn rotate_by(&mut self, rotation: &Quaternion) {
        let mut matrix = Matrix3::rotation_zyx(self.z_rot, self.y_rot, self.x_rot);
        matrix.rotate(rotation);
        let [x, y, z] = matrix.euler_angles_zyx();
        (self.x_rot, self.y_rot, self.z_rot) = (x, y, z);
    }

    /// `ModelPart.translateAndRotate`.
    pub fn translate_and_rotate(&self, pose: &mut Pose) {
        self.translate_and_rotate_shifted(pose, 0.0);
    }

    /// `translateAndRotate` with the part's x moved by `shift` for the
    /// duration (a slim arm's hand).
    pub fn translate_and_rotate_shifted(&self, pose: &mut Pose, shift: f32) {
        pose.translate((self.x + shift) / 16.0, self.y / 16.0, self.z / 16.0);
        if self.x_rot != 0.0 || self.y_rot != 0.0 || self.z_rot != 0.0 {
            pose.rotate(&Quaternion::rotation_zyx(
                self.z_rot, self.y_rot, self.x_rot,
            ));
        }
        if self.scale != [1.0; 3] {
            pose.scale(self.scale[0], self.scale[1], self.scale[2]);
        }
    }

    /// `ModelPart.render`: every visible face, posed, each with the path of
    /// the part it belongs to.
    pub fn render(&self, pose: &Pose, path: &str, out: &mut Vec<(String, [ModelVertex; 4])>) {
        let Some(pose) = self.own_quads(pose, |quad| out.push((path.to_owned(), quad))) else {
            return;
        };
        for child in &self.children {
            child.render(&pose, &format!("{path}/{}", child.name), out);
        }
    }

    /// The same faces as `render`, in the same order, without the paths of their parts.
    pub fn render_quads(&self, pose: &Pose, out: &mut Vec<[ModelVertex; 4]>) {
        let Some(pose) = self.own_quads(pose, |quad| out.push(quad)) else {
            return;
        };
        for child in &self.children {
            child.render_quads(&pose, out);
        }
    }

    /// Hand each face of this part's own cubes to `emit`, and give back the pose its children
    /// are drawn from; nothing when the part is not drawn.
    fn own_quads(&self, pose: &Pose, mut emit: impl FnMut([ModelVertex; 4])) -> Option<Pose> {
        if !self.visible || (self.cubes.is_empty() && self.children.is_empty()) {
            return None;
        }
        let mut pose = *pose;
        self.translate_and_rotate(&mut pose);
        for cube in self.cubes.iter().filter(|_| !self.skip_draw) {
            for polygon in &cube.polygons {
                let normal = pose.transform_normal(polygon.normal);
                let corner = |index: usize| {
                    let (position, uv) = polygon.corners[index];
                    ModelVertex {
                        position: pose.pose.transform_position(
                            position[0] / 16.0,
                            position[1] / 16.0,
                            position[2] / 16.0,
                        ),
                        uv,
                        normal,
                    }
                };
                emit([corner(0), corner(1), corner(2), corner(3)]);
            }
        }
        Some(pose)
    }
}
