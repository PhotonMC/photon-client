use super::*;

/// Load a model and its ancestors, then apply the reference's inheritance.
pub fn resolve_model(
    source: &impl AssetSource,
    name: &Identifier,
) -> Result<ResolvedModel, ResolveError> {
    ModelCache::default().resolve(source, name)
}

/// The reference's inheritance over a chain, child first.
pub(super) fn inherit(chain: &[std::sync::Arc<CuboidModel>], generated: bool) -> ResolvedModel {
    // Texture maps are collected child first, which is the order the resolver
    // expects; it applies them root first so a child overrides its parent.
    let mut resolver = TextureSlotsResolver::new();
    for model in chain {
        resolver.add_last(model.texture_slots.clone());
    }
    let textures = resolver.resolve();

    // Geometry, ambient occlusion and GUI light each take the first value set
    // walking up from the child.
    let elements = chain
        .iter()
        .find_map(|model| model.elements.clone())
        .unwrap_or_default();
    let ambient_occlusion = chain
        .iter()
        .find_map(|model| model.ambient_occlusion)
        .unwrap_or(true);
    let gui_light = chain
        .iter()
        .find_map(|model| model.gui_light)
        .unwrap_or(GuiLight::Side);

    let mut display = crate::model::ItemTransforms::default();
    for context in crate::model::DisplayContext::ALL {
        display.transforms[context as usize] = chain
            .iter()
            .find_map(|model| model.display.and_then(|transforms| transforms.get(context)));
    }

    ResolvedModel {
        textures,
        elements,
        ambient_occlusion,
        gui_light,
        display,
        generated,
    }
}

/// How many mip levels a sprite set can actually carry.
///
/// A level halves both axes, so a sprite can only be halved as many times as
/// its size allows. The reference takes the smallest side across the whole set
/// and also the lowest set bit of every side, because a sprite whose side is
/// not a multiple of the level's texel size cannot be halved cleanly either.
/// Whichever is smaller decides, and the request is never exceeded.
pub fn effective_mip_level(sprites: &[(i32, i32)], max_mip_level: i32) -> i32 {
    let mut min_texel_size = i32::MAX;
    // Java masks a shift count to five bits, so an absurd request wraps here
    // rather than overflowing.
    let mut lowest_one_bit = 1_i32.wrapping_shl(max_mip_level as u32);
    for (width, height) in sprites {
        min_texel_size = min_texel_size.min(*width.min(height));
        let lowest = (width & width.wrapping_neg()).min(height & height.wrapping_neg());
        if lowest < lowest_one_bit {
            lowest_one_bit = lowest;
        }
    }
    let min_size = min_texel_size.min(lowest_one_bit);
    let min_power_of_two = photon_core::math::log2(min_size);
    if min_power_of_two < max_mip_level {
        min_power_of_two
    } else {
        max_mip_level
    }
}

/// Pack every texture a resolved model uses into one atlas.
///
/// The placement comes from the pinned packer, so the layout is the one the
/// reference would produce for the same sprite set.
pub fn build_atlas(
    source: &impl AssetSource,
    model: &ResolvedModel,
    max_size: i32,
    mip_level: i32,
) -> Result<BuiltAtlas, ResolveError> {
    build_atlas_for(source, std::slice::from_ref(&model), max_size, mip_level)
}

/// Pack every texture a whole set of models uses into one atlas.
///
/// A world draws many models at once and they have to share one atlas, both
/// because the reference stitches one and because a draw call binds one
/// texture.
pub fn build_atlas_for(
    source: &impl AssetSource,
    models: &[&ResolvedModel],
    max_size: i32,
    mip_level: i32,
) -> Result<BuiltAtlas, ResolveError> {
    build_atlas_with(source, models, &[], max_size, mip_level)
}

/// The materials to pack, by sprite: those no model names and those the
/// models' faces do.
fn wanted_materials(
    models: &[&ResolvedModel],
    extra: &[Identifier],
) -> Result<BTreeMap<String, Material>, ResolveError> {
    let mut wanted: BTreeMap<String, Material> = BTreeMap::new();
    for sprite in extra {
        wanted.insert(
            sprite.as_string(),
            Material {
                sprite: sprite.clone(),
                force_translucent: false,
            },
        );
    }
    for model in models {
        for element in &model.elements {
            for (_, face) in &element.faces {
                let material = model
                    .textures
                    .material(&face.texture)
                    .ok_or_else(|| ResolveError::UnresolvedTexture(face.texture.clone()))?;
                wanted.insert(material.sprite.as_string(), material.clone());
            }
        }
    }
    Ok(wanted)
}

/// What a sprite's alpha holds: of the whole image, the strip for an
/// animated sprite, as the reference reads it.
fn sprite_alpha((image, _, animated): &DecodedSprite) -> crate::texture::sprite_alpha::SpriteAlpha {
    let frame = (image.width, image.height);
    match animated {
        Some((strip, playback, _)) => {
            let mut frames: Vec<u32> = Vec::new();
            for &(index, _) in playback {
                if !frames.contains(&(index as u32)) {
                    frames.push(index as u32);
                }
            }
            crate::texture::sprite_alpha::SpriteAlpha::measure(strip, frame, frames)
        }
        None => crate::texture::sprite_alpha::SpriteAlpha::measure(image, frame, Vec::new()),
    }
}

/// Pack a set of models' textures together with sprites no model names,
/// such as the fluid sprites the fluid renderer draws with.
pub fn build_atlas_with(
    source: &impl AssetSource,
    models: &[&ResolvedModel],
    extra: &[Identifier],
    max_size: i32,
    mip_level: i32,
) -> Result<BuiltAtlas, ResolveError> {
    let wanted = wanted_materials(models, extra)?;

    // Every sprite read and decoded in parallel; the first failure in the
    // sprites' order is the one reported.
    let listed: Vec<(&String, &Material)> = wanted.iter().collect();
    let results: Vec<Result<DecodedSprite, ResolveError>> = {
        use rayon::prelude::*;
        listed
            .par_iter()
            .map(|(key, material)| decode_sprite(source, key, material))
            .collect()
    };
    let mut decoded = BTreeMap::new();
    for ((key, _), result) in listed.into_iter().zip(results) {
        decoded.insert(key.clone(), result?);
    }
    // The level is settled before packing, because the padding the packer
    // reserves is one texel of the deepest level on each side.
    let sizes: Vec<(i32, i32)> = decoded
        .values()
        .map(|(image, _, _)| (image.width as i32, image.height as i32))
        .collect();
    let mip_level = effective_mip_level(&sizes, mip_level.max(0));
    let mut stitcher = Stitcher::new(max_size, max_size, mip_level, 0);
    for material in wanted.values() {
        let Some((image, _, _)) = decoded.get(&material.sprite.as_string()) else {
            continue;
        };
        stitcher.register_sprite(SpriteEntry {
            width: image.width as i32,
            height: image.height as i32,
            name: material.sprite.clone(),
        });
    }
    stitcher.stitch().map_err(|_| ResolveError::AtlasFull)?;

    let width = stitcher.width().max(1) as u32;
    let height = stitcher.height().max(1) as u32;
    // Every sprite's levels are built first, because the level count is the
    // same for the whole atlas and each sprite's level N goes into atlas
    // level N.
    let level_count = mip_level.max(0) as u32;
    let mut sprites = BTreeMap::new();
    let mut alpha_by_sprite = BTreeMap::new();
    let mut animations = Vec::new();
    let mut levels: Vec<DecodedImage> = (0..=level_count)
        .map(|level| {
            let level_width = (width >> level).max(1);
            let level_height = (height >> level).max(1);
            DecodedImage {
                width: level_width,
                height: level_height,
                rgba: vec![0_u8; (level_width * level_height * 4) as usize],
            }
        })
        .collect();
    // Every sprite's levels built in parallel, then written into the atlas
    // in the packer's order.
    let built: Vec<_> = {
        use rayon::prelude::*;
        stitcher
            .placements()
            .into_par_iter()
            .filter_map(|placement| {
                let key = placement.name.as_string();
                let entry = decoded.get(&key)?;
                let alpha = sprite_alpha(entry);
                let built = sprite_levels(&placement, key, entry, level_count, width, height);
                Some((built, alpha))
            })
            .collect()
    };
    for (built, alpha) in built {
        let placement = &built.placement;
        for (level, mip) in built.mips.iter().enumerate() {
            let shift = level as u32;
            blit_sprite(
                &mut levels[level],
                mip,
                (placement.x as u32 + placement.padding as u32) >> shift,
                (placement.y as u32 + placement.padding as u32) >> shift,
                placement.padding.max(0) as u32 >> shift,
            );
        }
        animations.extend(built.animation);
        alpha_by_sprite.insert(built.key.clone(), alpha);
        sprites.insert(built.key, built.rect);
    }
    Ok(BuiltAtlas {
        width,
        height,
        levels,
        sprites,
        alpha: alpha_by_sprite,
        animations,
    })
}

/// Read and decode one sprite of the atlas.
pub(super) fn decode_sprite(
    source: &impl AssetSource,
    key: &str,
    material: &Material,
) -> Result<DecodedSprite, ResolveError> {
    let bad = || ResolveError::BadTexture(key.to_owned());
    let bytes = source.texture(&material.sprite).ok_or_else(bad)?;
    let mut image = decode_rgba(&bytes).map_err(|_| bad())?;
    let mut animated = None;
    if let Some(animation) = source.texture_animation(&material.sprite) {
        let strip = image;
        image = first_animation_frame(&strip, &animation).ok_or_else(bad)?;
        let per_row = strip.width / image.width;
        let count = per_row * (strip.height / image.height);
        if let Some(playback) = animation.playback(count as i32) {
            // Still sprites start on the list's first frame; animated ones on
            // the first frame that plays.
            image = first_animation_frame(
                &strip,
                &AnimationFrames {
                    first_frame: playback[0].0,
                    ..animation.clone()
                },
            )
            .ok_or_else(bad)?;
            animated = Some((strip, playback, animation.interpolate));
        }
    }
    let metadata = source
        .texture_metadata(&material.sprite)
        .unwrap_or_default();
    Ok((image, metadata, animated))
}

/// A placed sprite's mip levels; an animated sprite's are its first
/// frame's, with every frame's kept for playback.
pub(super) fn sprite_levels(
    placement: &crate::texture::atlas::SpritePlacement,
    key: String,
    (image, metadata, animated): &DecodedSprite,
    level_count: u32,
    width: u32,
    height: u32,
) -> BuiltSprite {
    let sprite = AtlasSprite::new(
        placement,
        image.width as i32,
        image.height as i32,
        width as i32,
        height as i32,
    );
    let origin_x = placement.x as u32 + placement.padding as u32;
    let origin_y = placement.y as u32 + placement.padding as u32;
    let mut animation = None;
    let mips = if let Some((strip, playback, interpolate)) = animated {
        // The reference builds the levels of the whole strip, then cuts each
        // frame out of every level.
        let strip_mips = generate_mip_levels_with(
            placement.name.path(),
            strip,
            level_count,
            metadata.mipmap_strategy,
            metadata.alpha_cutoff_bias,
            compute_transparency(strip),
        );
        let per_row = strip.width / image.width;
        let mut frame_levels = BTreeMap::new();
        for &(index, _) in playback {
            frame_levels.entry(index).or_insert_with(|| {
                let left = index as u32 % per_row * image.width;
                let top = index as u32 / per_row * image.height;
                strip_mips
                    .iter()
                    .enumerate()
                    .map(|(level, mip)| {
                        let shift = level as u32;
                        crop(
                            mip,
                            left >> shift,
                            top >> shift,
                            image.width >> shift,
                            image.height >> shift,
                        )
                    })
                    .collect::<Vec<_>>()
            });
        }
        let first = frame_levels[&playback[0].0].clone();
        animation = Some(SpriteAnimation {
            name: key.clone(),
            x: origin_x,
            y: origin_y,
            padding: placement.padding.max(0) as u32,
            frame_levels,
            playback: playback.clone(),
            interpolate: *interpolate,
            frame: 0,
            sub_frame: 0,
            dirty: true,
        });
        first
    } else {
        generate_mip_levels_with(
            placement.name.path(),
            image,
            level_count,
            metadata.mipmap_strategy,
            metadata.alpha_cutoff_bias,
            compute_transparency(image),
        )
    };
    BuiltSprite {
        placement: placement.clone(),
        key,
        rect: [sprite.u0, sprite.v0, sprite.u1, sprite.v1],
        mips,
        animation,
    }
}

/// A rectangle of an image.
pub(super) fn crop(
    image: &DecodedImage,
    left: u32,
    top: u32,
    width: u32,
    height: u32,
) -> DecodedImage {
    let mut rgba = Vec::with_capacity((width * height * 4) as usize);
    for row in top..top + height {
        let start = ((row * image.width + left) * 4) as usize;
        rgba.extend_from_slice(&image.rgba[start..start + width as usize * 4]);
    }
    DecodedImage {
        width,
        height,
        rgba,
    }
}

/// Write one sprite level into one atlas level, extending its edge texels into
/// the padding around it.
///
/// The reference fills the atlas with a draw per sprite whose quad covers the
/// padded slot and whose texture coordinates run from `-padding/width` to
/// `1 + padding/width`, sampled at the nearest texel with clamping at the
/// sprite's edge. Working that through, atlas texel `i` of the slot reads
/// sprite texel `i - padding` clamped into range, so the padding carries the
/// edge texel repeated -- which is the point, since it is what a sampler reads
/// when a face lands right on the sprite's border.
pub(super) fn blit_sprite(
    target: &mut DecodedImage,
    sprite: &DecodedImage,
    origin_x: u32,
    origin_y: u32,
    padding: u32,
) {
    if sprite.width == 0 || sprite.height == 0 {
        return;
    }
    let start_x = origin_x.saturating_sub(padding);
    let start_y = origin_y.saturating_sub(padding);
    let span_x = sprite.width + padding * 2;
    let span_y = sprite.height + padding * 2;
    for row in 0..span_y {
        let target_y = start_y + row;
        if target_y >= target.height {
            break;
        }
        let source_y = row.saturating_sub(padding).min(sprite.height - 1);
        for column in 0..span_x {
            let target_x = start_x + column;
            if target_x >= target.width {
                break;
            }
            let source_x = column.saturating_sub(padding).min(sprite.width - 1);
            let from = ((source_y * sprite.width) + source_x) as usize * 4;
            let to = ((target_y * target.width) + target_x) as usize * 4;
            target.rgba[to..to + 4].copy_from_slice(&sprite.rgba[from..from + 4]);
        }
    }
}
