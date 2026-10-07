use super::*;

/// Build the mesh for a resolved model against a packed atlas.
///
/// Corner positions, default UVs, element rotation, the variant's whole-model
/// rotation and UV rotation all come from the pinned paths; the only thing
/// added here is the per-face shade, which is this module's own and matches
/// nothing in the reference.
pub fn build_mesh(
    model: &ResolvedModel,
    atlas: &BuiltAtlas,
    model_state: Option<&crate::block::rotation::Matrix4>,
    tints: &[u32],
    uv_transform: impl Fn(voxel_core::direction::Direction) -> crate::block::rotation::FlatMatrix,
    occluded: impl Fn(voxel_core::direction::Direction) -> bool,
) -> Result<Mesh, ResolveError> {
    let mut mesh = Mesh::default();
    for element in &model.elements {
        for (facing, face) in &element.faces {
            // A face may name the side it hides behind. The reference sorts
            // quads into one bucket per such side plus one for the rest, and
            // skips a bucket when the neighbour on that side covers it.
            if let Some(cull) = face.cull_for_direction {
                // The side is named in the model's own frame, so the variant's
                // rotation has to be applied before the neighbour is asked.
                let cull = match model_state {
                    Some(state) => crate::block::rotation::rotate_direction(state, cull),
                    None => cull,
                };
                if occluded(cull) {
                    continue;
                }
            }
            let material = model
                .textures
                .material(&face.texture)
                .ok_or_else(|| ResolveError::UnresolvedTexture(face.texture.clone()))?;
            let key = material.sprite.as_string();
            let rect = atlas
                .sprites
                .get(&key)
                .ok_or_else(|| ResolveError::BadTexture(key.clone()))?;

            let mut positions = face_vertices(element.from, element.to, *facing);
            let uvs = face
                .uvs
                .unwrap_or_else(|| default_face_uv(element.from, element.to, *facing));

            let tint = face_tint(tints, face.tint_index);
            let face_uv = uv_transform(*facing);
            let base = mesh.vertices.len() as u32;
            let mut packed_uvs = [0_u64; 4];
            for (slot, position) in positions.iter_mut().enumerate() {
                let mut rotated = match &element.rotation {
                    Some(rotation) => {
                        rotate_vertex(*position, rotation.origin, &rotation.transform)
                    }
                    None => *position,
                };
                // The variant's rotation comes after the element's and turns
                // about the block's centre, as the reference orders them.
                if let Some(state) = model_state {
                    rotated = rotate_vertex(rotated, BLOCK_CENTRE, state);
                }
                *position = rotated;
                // UV lock turns the texture coordinate back by as much as the
                // model turned, so the texture keeps its world orientation.
                // Without it the transform is identity and the coordinate
                // passes through.
                let (u, v) = face_vertex_uv(uvs, face.rotation, slot as u32);
                let (u, v) = crate::block::rotation::transform_face_uv(&face_uv, u, v);
                packed_uvs[slot] = (u64::from(u.to_bits()) << 32) | u64::from(v.to_bits());
            }
            // FaceBakery: the quad faces whichever side its normal is closest
            // to, and without an element rotation its corners are put back in
            // that side's order.
            let direction = crate::block::geometry::closest_direction(
                crate::block::geometry::triangle_normal(positions[0], positions[1], positions[2]),
            );
            if element.rotation.is_none()
                && let Some(direction) = direction
            {
                let _ = crate::block::geometry::recalculate_winding(
                    &mut positions,
                    &mut packed_uvs,
                    direction,
                );
            }
            let direction = direction.unwrap_or(voxel_core::direction::Direction::Up);
            for (position, packed) in positions.into_iter().zip(packed_uvs) {
                let u = f32::from_bits((packed >> 32) as u32);
                let v = f32::from_bits(packed as u32);
                mesh.vertices.push(BlockVertex::new(
                    position,
                    [
                        rect[0] + (rect[2] - rect[0]) * u,
                        rect[1] + (rect[3] - rect[1]) * v,
                    ],
                    tint,
                    crate::scene::FULL_BRIGHT,
                ));
            }
            mesh.quads.push(crate::scene::QuadInfo {
                direction,
                cull: face.cull_for_direction.map(|cull| match model_state {
                    Some(state) => crate::block::rotation::rotate_direction(state, cull),
                    None => cull,
                }),
                shade: element.shade,
                tint_index: face.tint_index,
            });
            mesh.indices
                .extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
        }
    }
    Ok(mesh)
}

/// A face's colour from its tint slot, as red, green, blue and alpha. A face
/// names a slot rather than a colour; a slot the caller did not resolve
/// leaves the face untinted, which is what the reference does when a block
/// has no source for that layer.
pub(super) fn face_tint(tints: &[u32], slot: i32) -> [f32; 4] {
    let tint = usize::try_from(slot)
        .ok()
        .and_then(|slot| tints.get(slot).copied())
        .unwrap_or(0xFFFF_FFFF);
    [16, 8, 0, 24].map(|bits| ((tint >> bits) & 0xFF) as f32 / 255.0)
}

/// Cut the frame an animation starts on out of its strip. Frames run left to
/// right, then down, as many to a row as fit across the image.
pub(super) fn first_animation_frame(
    image: &DecodedImage,
    animation: &AnimationFrames,
) -> Option<DecodedImage> {
    let (width, height) = (image.width as i32, image.height as i32);
    let (frame_width, frame_height) = animation.frame_size(width, height);
    if frame_width <= 0
        || frame_height <= 0
        || width % frame_width != 0
        || height % frame_height != 0
    {
        return None;
    }
    let per_row = width / frame_width;
    let count = per_row * (height / frame_height);
    if animation.first_frame < 0 || animation.first_frame >= count {
        return None;
    }
    let left = (animation.first_frame % per_row * frame_width) as usize;
    let top = (animation.first_frame / per_row * frame_height) as usize;
    let mut rgba = Vec::with_capacity((frame_width * frame_height * 4) as usize);
    for row in 0..frame_height as usize {
        let start = ((top + row) * image.width as usize + left) * 4;
        rgba.extend_from_slice(&image.rgba[start..start + frame_width as usize * 4]);
    }
    Some(DecodedImage {
        width: frame_width as u32,
        height: frame_height as u32,
        rgba,
    })
}
