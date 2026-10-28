//! Every block, in registry order, as the reference's `Blocks` declares it.
//!
//! This table is maintained by hand; the parity families compare it with the reference.

use crate::block::classes::BlockClass;
use crate::block::definition::{Block, Emission, Fluid, OffsetType, PushReaction, Rule};
use crate::block::properties::*;

#[rustfmt::skip]
pub static BLOCKS: [Block; 1196] = [
    Block::of("minecraft:air", BlockClass::AirBlock).no_collision().air().replaceable(),
    Block::of("minecraft:stone", BlockClass::Block).strength(1.5, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:granite", BlockClass::Block).strength(1.5, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:polished_granite", BlockClass::Block).strength(1.5, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:diorite", BlockClass::Block).strength(1.5, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:polished_diorite", BlockClass::Block).strength(1.5, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:andesite", BlockClass::Block).strength(1.5, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:polished_andesite", BlockClass::Block).strength(1.5, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:grass_block", BlockClass::GrassBlock).properties(&[&SNOWY], &["false"]).strength(0.6, 0.6).random_ticks(),
    Block::of("minecraft:dirt", BlockClass::Block).strength(0.5, 0.5),
    Block::of("minecraft:coarse_dirt", BlockClass::Block).strength(0.5, 0.5),
    Block::of("minecraft:podzol", BlockClass::SnowyBlock).properties(&[&SNOWY], &["false"]).strength(0.5, 0.5),
    Block::of("minecraft:cobblestone", BlockClass::Block).strength(2.0, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:oak_planks", BlockClass::Block).strength(2.0, 3.0).ignited_by_lava().instrument("bass"),
    Block::of("minecraft:spruce_planks", BlockClass::Block).strength(2.0, 3.0).ignited_by_lava().instrument("bass"),
    Block::of("minecraft:birch_planks", BlockClass::Block).strength(2.0, 3.0).ignited_by_lava().instrument("bass"),
    Block::of("minecraft:jungle_planks", BlockClass::Block).strength(2.0, 3.0).ignited_by_lava().instrument("bass"),
    Block::of("minecraft:acacia_planks", BlockClass::Block).strength(2.0, 3.0).ignited_by_lava().instrument("bass"),
    Block::of("minecraft:cherry_planks", BlockClass::Block).strength(2.0, 3.0).ignited_by_lava().instrument("bass"),
    Block::of("minecraft:dark_oak_planks", BlockClass::Block).strength(2.0, 3.0).ignited_by_lava().instrument("bass"),
    Block::of("minecraft:pale_oak_wood", BlockClass::RotatedPillarBlock).properties(&[&AXIS], &["y"]).strength(2.0, 2.0).ignited_by_lava().instrument("bass"),
    Block::of("minecraft:pale_oak_planks", BlockClass::Block).strength(2.0, 3.0).ignited_by_lava().instrument("bass"),
    Block::of("minecraft:mangrove_planks", BlockClass::Block).strength(2.0, 3.0).ignited_by_lava().instrument("bass"),
    Block::of("minecraft:bamboo_planks", BlockClass::Block).strength(2.0, 3.0).ignited_by_lava().instrument("bass"),
    Block::of("minecraft:bamboo_mosaic", BlockClass::Block).strength(2.0, 3.0).ignited_by_lava().instrument("bass"),
    Block::of("minecraft:oak_sapling", BlockClass::SaplingBlock).properties(&[&STAGE], &["0"]).no_collision().random_ticks().push_reaction(PushReaction::Destroy),
    Block::of("minecraft:spruce_sapling", BlockClass::SaplingBlock).properties(&[&STAGE], &["0"]).no_collision().random_ticks().push_reaction(PushReaction::Destroy),
    Block::of("minecraft:birch_sapling", BlockClass::SaplingBlock).properties(&[&STAGE], &["0"]).no_collision().random_ticks().push_reaction(PushReaction::Destroy),
    Block::of("minecraft:jungle_sapling", BlockClass::SaplingBlock).properties(&[&STAGE], &["0"]).no_collision().random_ticks().push_reaction(PushReaction::Destroy),
    Block::of("minecraft:acacia_sapling", BlockClass::SaplingBlock).properties(&[&STAGE], &["0"]).no_collision().random_ticks().push_reaction(PushReaction::Destroy),
    Block::of("minecraft:cherry_sapling", BlockClass::SaplingBlock).properties(&[&STAGE], &["0"]).no_collision().random_ticks().push_reaction(PushReaction::Destroy),
    Block::of("minecraft:dark_oak_sapling", BlockClass::SaplingBlock).properties(&[&STAGE], &["0"]).no_collision().random_ticks().push_reaction(PushReaction::Destroy),
    Block::of("minecraft:pale_oak_sapling", BlockClass::SaplingBlock).properties(&[&STAGE], &["0"]).no_collision().random_ticks().push_reaction(PushReaction::Destroy),
    Block::of("minecraft:mangrove_propagule", BlockClass::MangrovePropaguleBlock).properties(&[&AGE_4, &HANGING, &STAGE, &WATERLOGGED], &["0", "false", "0", "false"]).no_collision().random_ticks().offset_type(OffsetType::Xz).push_reaction(PushReaction::Destroy),
    Block::of("minecraft:bedrock", BlockClass::Block).strength(-1.0, 3600000.0).instrument("basedrum"),
    Block::of("minecraft:water", BlockClass::LiquidBlock).properties(&[&LEVEL], &["0"]).strength(100.0, 100.0).no_collision().liquid().replaceable().push_reaction(PushReaction::Destroy).item("minecraft:air").fluid(Fluid::Water),
    Block::of("minecraft:lava", BlockClass::LiquidBlock).properties(&[&LEVEL], &["0"]).strength(100.0, 100.0).no_collision().random_ticks().liquid().replaceable().push_reaction(PushReaction::Destroy).item("minecraft:air").fluid(Fluid::Lava).light_level(Emission::Constant(15)),
    Block::of("minecraft:sand", BlockClass::SandBlock).strength(0.5, 0.5).instrument("snare"),
    Block::of("minecraft:suspicious_sand", BlockClass::BrushableBlock).properties(&[&DUSTED], &["0"]).strength(0.25, 0.25).push_reaction(PushReaction::Destroy).instrument("snare"),
    Block::of("minecraft:red_sand", BlockClass::SandBlock).strength(0.5, 0.5).instrument("snare"),
    Block::of("minecraft:gravel", BlockClass::ColoredFallingBlock).strength(0.6, 0.6).instrument("snare"),
    Block::of("minecraft:suspicious_gravel", BlockClass::BrushableBlock).properties(&[&DUSTED], &["0"]).strength(0.25, 0.25).push_reaction(PushReaction::Destroy).instrument("snare"),
    Block::of("minecraft:gold_ore", BlockClass::DropExperienceBlock).strength(3.0, 3.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:deepslate_gold_ore", BlockClass::DropExperienceBlock).strength(4.5, 3.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:iron_ore", BlockClass::DropExperienceBlock).strength(3.0, 3.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:deepslate_iron_ore", BlockClass::DropExperienceBlock).strength(4.5, 3.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:coal_ore", BlockClass::DropExperienceBlock).strength(3.0, 3.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:deepslate_coal_ore", BlockClass::DropExperienceBlock).strength(4.5, 3.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:nether_gold_ore", BlockClass::DropExperienceBlock).strength(3.0, 3.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:oak_log", BlockClass::RotatedPillarBlock).properties(&[&AXIS], &["y"]).strength(2.0, 2.0).ignited_by_lava().instrument("bass"),
    Block::of("minecraft:spruce_log", BlockClass::RotatedPillarBlock).properties(&[&AXIS], &["y"]).strength(2.0, 2.0).ignited_by_lava().instrument("bass"),
    Block::of("minecraft:birch_log", BlockClass::RotatedPillarBlock).properties(&[&AXIS], &["y"]).strength(2.0, 2.0).ignited_by_lava().instrument("bass"),
    Block::of("minecraft:jungle_log", BlockClass::RotatedPillarBlock).properties(&[&AXIS], &["y"]).strength(2.0, 2.0).ignited_by_lava().instrument("bass"),
    Block::of("minecraft:acacia_log", BlockClass::RotatedPillarBlock).properties(&[&AXIS], &["y"]).strength(2.0, 2.0).ignited_by_lava().instrument("bass"),
    Block::of("minecraft:cherry_log", BlockClass::RotatedPillarBlock).properties(&[&AXIS], &["y"]).strength(2.0, 2.0).ignited_by_lava().instrument("bass"),
    Block::of("minecraft:dark_oak_log", BlockClass::RotatedPillarBlock).properties(&[&AXIS], &["y"]).strength(2.0, 2.0).ignited_by_lava().instrument("bass"),
    Block::of("minecraft:pale_oak_log", BlockClass::RotatedPillarBlock).properties(&[&AXIS], &["y"]).strength(2.0, 2.0).ignited_by_lava().instrument("bass"),
    Block::of("minecraft:mangrove_log", BlockClass::RotatedPillarBlock).properties(&[&AXIS], &["y"]).strength(2.0, 2.0).ignited_by_lava().instrument("bass"),
    Block::of("minecraft:mangrove_roots", BlockClass::MangroveRootsBlock).properties(&[&WATERLOGGED], &["false"]).strength(0.7, 0.7).no_occlusion().ignited_by_lava().instrument("bass").is_suffocating(Rule::Never).is_view_blocking(Rule::Never),
    Block::of("minecraft:muddy_mangrove_roots", BlockClass::RotatedPillarBlock).properties(&[&AXIS], &["y"]).strength(0.7, 0.7),
    Block::of("minecraft:bamboo_block", BlockClass::RotatedPillarBlock).properties(&[&AXIS], &["y"]).strength(2.0, 2.0).ignited_by_lava().instrument("bass"),
    Block::of("minecraft:stripped_spruce_log", BlockClass::RotatedPillarBlock).properties(&[&AXIS], &["y"]).strength(2.0, 2.0).ignited_by_lava().instrument("bass"),
    Block::of("minecraft:stripped_birch_log", BlockClass::RotatedPillarBlock).properties(&[&AXIS], &["y"]).strength(2.0, 2.0).ignited_by_lava().instrument("bass"),
    Block::of("minecraft:stripped_jungle_log", BlockClass::RotatedPillarBlock).properties(&[&AXIS], &["y"]).strength(2.0, 2.0).ignited_by_lava().instrument("bass"),
    Block::of("minecraft:stripped_acacia_log", BlockClass::RotatedPillarBlock).properties(&[&AXIS], &["y"]).strength(2.0, 2.0).ignited_by_lava().instrument("bass"),
    Block::of("minecraft:stripped_cherry_log", BlockClass::RotatedPillarBlock).properties(&[&AXIS], &["y"]).strength(2.0, 2.0).ignited_by_lava().instrument("bass"),
    Block::of("minecraft:stripped_dark_oak_log", BlockClass::RotatedPillarBlock).properties(&[&AXIS], &["y"]).strength(2.0, 2.0).ignited_by_lava().instrument("bass"),
    Block::of("minecraft:stripped_pale_oak_log", BlockClass::RotatedPillarBlock).properties(&[&AXIS], &["y"]).strength(2.0, 2.0).ignited_by_lava().instrument("bass"),
    Block::of("minecraft:stripped_oak_log", BlockClass::RotatedPillarBlock).properties(&[&AXIS], &["y"]).strength(2.0, 2.0).ignited_by_lava().instrument("bass"),
    Block::of("minecraft:stripped_mangrove_log", BlockClass::RotatedPillarBlock).properties(&[&AXIS], &["y"]).strength(2.0, 2.0).ignited_by_lava().instrument("bass"),
    Block::of("minecraft:stripped_bamboo_block", BlockClass::RotatedPillarBlock).properties(&[&AXIS], &["y"]).strength(2.0, 2.0).ignited_by_lava().instrument("bass"),
    Block::of("minecraft:oak_wood", BlockClass::RotatedPillarBlock).properties(&[&AXIS], &["y"]).strength(2.0, 2.0).ignited_by_lava().instrument("bass"),
    Block::of("minecraft:spruce_wood", BlockClass::RotatedPillarBlock).properties(&[&AXIS], &["y"]).strength(2.0, 2.0).ignited_by_lava().instrument("bass"),
    Block::of("minecraft:birch_wood", BlockClass::RotatedPillarBlock).properties(&[&AXIS], &["y"]).strength(2.0, 2.0).ignited_by_lava().instrument("bass"),
    Block::of("minecraft:jungle_wood", BlockClass::RotatedPillarBlock).properties(&[&AXIS], &["y"]).strength(2.0, 2.0).ignited_by_lava().instrument("bass"),
    Block::of("minecraft:acacia_wood", BlockClass::RotatedPillarBlock).properties(&[&AXIS], &["y"]).strength(2.0, 2.0).ignited_by_lava().instrument("bass"),
    Block::of("minecraft:cherry_wood", BlockClass::RotatedPillarBlock).properties(&[&AXIS], &["y"]).strength(2.0, 2.0).ignited_by_lava().instrument("bass"),
    Block::of("minecraft:dark_oak_wood", BlockClass::RotatedPillarBlock).properties(&[&AXIS], &["y"]).strength(2.0, 2.0).ignited_by_lava().instrument("bass"),
    Block::of("minecraft:mangrove_wood", BlockClass::RotatedPillarBlock).properties(&[&AXIS], &["y"]).strength(2.0, 2.0).ignited_by_lava().instrument("bass"),
    Block::of("minecraft:stripped_oak_wood", BlockClass::RotatedPillarBlock).properties(&[&AXIS], &["y"]).strength(2.0, 2.0).ignited_by_lava().instrument("bass"),
    Block::of("minecraft:stripped_spruce_wood", BlockClass::RotatedPillarBlock).properties(&[&AXIS], &["y"]).strength(2.0, 2.0).ignited_by_lava().instrument("bass"),
    Block::of("minecraft:stripped_birch_wood", BlockClass::RotatedPillarBlock).properties(&[&AXIS], &["y"]).strength(2.0, 2.0).ignited_by_lava().instrument("bass"),
    Block::of("minecraft:stripped_jungle_wood", BlockClass::RotatedPillarBlock).properties(&[&AXIS], &["y"]).strength(2.0, 2.0).ignited_by_lava().instrument("bass"),
    Block::of("minecraft:stripped_acacia_wood", BlockClass::RotatedPillarBlock).properties(&[&AXIS], &["y"]).strength(2.0, 2.0).ignited_by_lava().instrument("bass"),
    Block::of("minecraft:stripped_cherry_wood", BlockClass::RotatedPillarBlock).properties(&[&AXIS], &["y"]).strength(2.0, 2.0).ignited_by_lava().instrument("bass"),
    Block::of("minecraft:stripped_dark_oak_wood", BlockClass::RotatedPillarBlock).properties(&[&AXIS], &["y"]).strength(2.0, 2.0).ignited_by_lava().instrument("bass"),
    Block::of("minecraft:stripped_pale_oak_wood", BlockClass::RotatedPillarBlock).properties(&[&AXIS], &["y"]).strength(2.0, 2.0).ignited_by_lava().instrument("bass"),
    Block::of("minecraft:stripped_mangrove_wood", BlockClass::RotatedPillarBlock).properties(&[&AXIS], &["y"]).strength(2.0, 2.0).ignited_by_lava().instrument("bass"),
    Block::of("minecraft:oak_leaves", BlockClass::TintedParticleLeavesBlock).properties(&[&DISTANCE, &PERSISTENT, &WATERLOGGED], &["7", "false", "false"]).strength(0.2, 0.2).no_occlusion().random_ticks().ignited_by_lava().push_reaction(PushReaction::Destroy).is_suffocating(Rule::Never).is_view_blocking(Rule::Never).is_redstone_conductor(Rule::Never),
    Block::of("minecraft:spruce_leaves", BlockClass::TintedParticleLeavesBlock).properties(&[&DISTANCE, &PERSISTENT, &WATERLOGGED], &["7", "false", "false"]).strength(0.2, 0.2).no_occlusion().random_ticks().ignited_by_lava().push_reaction(PushReaction::Destroy).is_suffocating(Rule::Never).is_view_blocking(Rule::Never).is_redstone_conductor(Rule::Never),
    Block::of("minecraft:birch_leaves", BlockClass::TintedParticleLeavesBlock).properties(&[&DISTANCE, &PERSISTENT, &WATERLOGGED], &["7", "false", "false"]).strength(0.2, 0.2).no_occlusion().random_ticks().ignited_by_lava().push_reaction(PushReaction::Destroy).is_suffocating(Rule::Never).is_view_blocking(Rule::Never).is_redstone_conductor(Rule::Never),
    Block::of("minecraft:jungle_leaves", BlockClass::TintedParticleLeavesBlock).properties(&[&DISTANCE, &PERSISTENT, &WATERLOGGED], &["7", "false", "false"]).strength(0.2, 0.2).no_occlusion().random_ticks().ignited_by_lava().push_reaction(PushReaction::Destroy).is_suffocating(Rule::Never).is_view_blocking(Rule::Never).is_redstone_conductor(Rule::Never),
    Block::of("minecraft:acacia_leaves", BlockClass::TintedParticleLeavesBlock).properties(&[&DISTANCE, &PERSISTENT, &WATERLOGGED], &["7", "false", "false"]).strength(0.2, 0.2).no_occlusion().random_ticks().ignited_by_lava().push_reaction(PushReaction::Destroy).is_suffocating(Rule::Never).is_view_blocking(Rule::Never).is_redstone_conductor(Rule::Never),
    Block::of("minecraft:cherry_leaves", BlockClass::UntintedParticleLeavesBlock).properties(&[&DISTANCE, &PERSISTENT, &WATERLOGGED], &["7", "false", "false"]).strength(0.2, 0.2).no_occlusion().random_ticks().ignited_by_lava().push_reaction(PushReaction::Destroy).is_suffocating(Rule::Never).is_view_blocking(Rule::Never).is_redstone_conductor(Rule::Never),
    Block::of("minecraft:dark_oak_leaves", BlockClass::TintedParticleLeavesBlock).properties(&[&DISTANCE, &PERSISTENT, &WATERLOGGED], &["7", "false", "false"]).strength(0.2, 0.2).no_occlusion().random_ticks().ignited_by_lava().push_reaction(PushReaction::Destroy).is_suffocating(Rule::Never).is_view_blocking(Rule::Never).is_redstone_conductor(Rule::Never),
    Block::of("minecraft:pale_oak_leaves", BlockClass::UntintedParticleLeavesBlock).properties(&[&DISTANCE, &PERSISTENT, &WATERLOGGED], &["7", "false", "false"]).strength(0.2, 0.2).no_occlusion().random_ticks().ignited_by_lava().push_reaction(PushReaction::Destroy).is_suffocating(Rule::Never).is_view_blocking(Rule::Never).is_redstone_conductor(Rule::Never),
    Block::of("minecraft:mangrove_leaves", BlockClass::MangroveLeavesBlock).properties(&[&DISTANCE, &PERSISTENT, &WATERLOGGED], &["7", "false", "false"]).strength(0.2, 0.2).no_occlusion().random_ticks().ignited_by_lava().push_reaction(PushReaction::Destroy).is_suffocating(Rule::Never).is_view_blocking(Rule::Never).is_redstone_conductor(Rule::Never),
    Block::of("minecraft:azalea_leaves", BlockClass::UntintedParticleLeavesBlock).properties(&[&DISTANCE, &PERSISTENT, &WATERLOGGED], &["7", "false", "false"]).strength(0.2, 0.2).no_occlusion().random_ticks().ignited_by_lava().push_reaction(PushReaction::Destroy).is_suffocating(Rule::Never).is_view_blocking(Rule::Never).is_redstone_conductor(Rule::Never),
    Block::of("minecraft:flowering_azalea_leaves", BlockClass::UntintedParticleLeavesBlock).properties(&[&DISTANCE, &PERSISTENT, &WATERLOGGED], &["7", "false", "false"]).strength(0.2, 0.2).no_occlusion().random_ticks().ignited_by_lava().push_reaction(PushReaction::Destroy).is_suffocating(Rule::Never).is_view_blocking(Rule::Never).is_redstone_conductor(Rule::Never),
    Block::of("minecraft:sponge", BlockClass::SpongeBlock).strength(0.6, 0.6),
    Block::of("minecraft:wet_sponge", BlockClass::WetSpongeBlock).strength(0.6, 0.6),
    Block::of("minecraft:glass", BlockClass::TransparentBlock).strength(0.3, 0.3).no_occlusion().instrument("hat").is_suffocating(Rule::Never).is_view_blocking(Rule::Never).is_redstone_conductor(Rule::Never),
    Block::of("minecraft:lapis_ore", BlockClass::DropExperienceBlock).strength(3.0, 3.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:deepslate_lapis_ore", BlockClass::DropExperienceBlock).strength(4.5, 3.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:lapis_block", BlockClass::Block).strength(3.0, 3.0).requires_correct_tool_for_drops(),
    Block::of("minecraft:dispenser", BlockClass::DispenserBlock).properties(&[&FACING, &TRIGGERED], &["north", "false"]).strength(3.5, 3.5).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:sandstone", BlockClass::Block).strength(0.8, 0.8).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:chiseled_sandstone", BlockClass::Block).strength(0.8, 0.8).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:cut_sandstone", BlockClass::Block).strength(0.8, 0.8).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:note_block", BlockClass::NoteBlock).properties(&[&NOTEBLOCK_INSTRUMENT, &NOTE, &POWERED], &["harp", "0", "false"]).strength(0.8, 0.8).ignited_by_lava().instrument("bass"),
    Block::of("minecraft:white_bed", BlockClass::BedBlock).properties(&[&HORIZONTAL_FACING, &OCCUPIED, &BED_PART], &["north", "false", "foot"]).strength(0.2, 0.2).bounce_restitution(0.75).no_occlusion().ignited_by_lava().push_reaction(PushReaction::Destroy),
    Block::of("minecraft:orange_bed", BlockClass::BedBlock).properties(&[&HORIZONTAL_FACING, &OCCUPIED, &BED_PART], &["north", "false", "foot"]).strength(0.2, 0.2).bounce_restitution(0.75).no_occlusion().ignited_by_lava().push_reaction(PushReaction::Destroy),
    Block::of("minecraft:magenta_bed", BlockClass::BedBlock).properties(&[&HORIZONTAL_FACING, &OCCUPIED, &BED_PART], &["north", "false", "foot"]).strength(0.2, 0.2).bounce_restitution(0.75).no_occlusion().ignited_by_lava().push_reaction(PushReaction::Destroy),
    Block::of("minecraft:light_blue_bed", BlockClass::BedBlock).properties(&[&HORIZONTAL_FACING, &OCCUPIED, &BED_PART], &["north", "false", "foot"]).strength(0.2, 0.2).bounce_restitution(0.75).no_occlusion().ignited_by_lava().push_reaction(PushReaction::Destroy),
    Block::of("minecraft:yellow_bed", BlockClass::BedBlock).properties(&[&HORIZONTAL_FACING, &OCCUPIED, &BED_PART], &["north", "false", "foot"]).strength(0.2, 0.2).bounce_restitution(0.75).no_occlusion().ignited_by_lava().push_reaction(PushReaction::Destroy),
    Block::of("minecraft:lime_bed", BlockClass::BedBlock).properties(&[&HORIZONTAL_FACING, &OCCUPIED, &BED_PART], &["north", "false", "foot"]).strength(0.2, 0.2).bounce_restitution(0.75).no_occlusion().ignited_by_lava().push_reaction(PushReaction::Destroy),
    Block::of("minecraft:pink_bed", BlockClass::BedBlock).properties(&[&HORIZONTAL_FACING, &OCCUPIED, &BED_PART], &["north", "false", "foot"]).strength(0.2, 0.2).bounce_restitution(0.75).no_occlusion().ignited_by_lava().push_reaction(PushReaction::Destroy),
    Block::of("minecraft:gray_bed", BlockClass::BedBlock).properties(&[&HORIZONTAL_FACING, &OCCUPIED, &BED_PART], &["north", "false", "foot"]).strength(0.2, 0.2).bounce_restitution(0.75).no_occlusion().ignited_by_lava().push_reaction(PushReaction::Destroy),
    Block::of("minecraft:light_gray_bed", BlockClass::BedBlock).properties(&[&HORIZONTAL_FACING, &OCCUPIED, &BED_PART], &["north", "false", "foot"]).strength(0.2, 0.2).bounce_restitution(0.75).no_occlusion().ignited_by_lava().push_reaction(PushReaction::Destroy),
    Block::of("minecraft:cyan_bed", BlockClass::BedBlock).properties(&[&HORIZONTAL_FACING, &OCCUPIED, &BED_PART], &["north", "false", "foot"]).strength(0.2, 0.2).bounce_restitution(0.75).no_occlusion().ignited_by_lava().push_reaction(PushReaction::Destroy),
    Block::of("minecraft:purple_bed", BlockClass::BedBlock).properties(&[&HORIZONTAL_FACING, &OCCUPIED, &BED_PART], &["north", "false", "foot"]).strength(0.2, 0.2).bounce_restitution(0.75).no_occlusion().ignited_by_lava().push_reaction(PushReaction::Destroy),
    Block::of("minecraft:blue_bed", BlockClass::BedBlock).properties(&[&HORIZONTAL_FACING, &OCCUPIED, &BED_PART], &["north", "false", "foot"]).strength(0.2, 0.2).bounce_restitution(0.75).no_occlusion().ignited_by_lava().push_reaction(PushReaction::Destroy),
    Block::of("minecraft:brown_bed", BlockClass::BedBlock).properties(&[&HORIZONTAL_FACING, &OCCUPIED, &BED_PART], &["north", "false", "foot"]).strength(0.2, 0.2).bounce_restitution(0.75).no_occlusion().ignited_by_lava().push_reaction(PushReaction::Destroy),
    Block::of("minecraft:green_bed", BlockClass::BedBlock).properties(&[&HORIZONTAL_FACING, &OCCUPIED, &BED_PART], &["north", "false", "foot"]).strength(0.2, 0.2).bounce_restitution(0.75).no_occlusion().ignited_by_lava().push_reaction(PushReaction::Destroy),
    Block::of("minecraft:red_bed", BlockClass::BedBlock).properties(&[&HORIZONTAL_FACING, &OCCUPIED, &BED_PART], &["north", "false", "foot"]).strength(0.2, 0.2).bounce_restitution(0.75).no_occlusion().ignited_by_lava().push_reaction(PushReaction::Destroy),
    Block::of("minecraft:black_bed", BlockClass::BedBlock).properties(&[&HORIZONTAL_FACING, &OCCUPIED, &BED_PART], &["north", "false", "foot"]).strength(0.2, 0.2).bounce_restitution(0.75).no_occlusion().ignited_by_lava().push_reaction(PushReaction::Destroy),
    Block::of("minecraft:powered_rail", BlockClass::PoweredRailBlock).properties(&[&POWERED, &RAIL_SHAPE_STRAIGHT, &WATERLOGGED], &["false", "north_south", "false"]).strength(0.7, 0.7).no_collision(),
    Block::of("minecraft:detector_rail", BlockClass::DetectorRailBlock).properties(&[&POWERED, &RAIL_SHAPE_STRAIGHT, &WATERLOGGED], &["false", "north_south", "false"]).strength(0.7, 0.7).no_collision(),
    Block::of("minecraft:sticky_piston", BlockClass::PistonBaseBlock).properties(&[&EXTENDED, &FACING], &["false", "north"]).strength(1.5, 1.5).push_reaction(PushReaction::Block).is_suffocating(Rule::States(&[false, false, false, false, false, false, true, true, true, true, true, true])).is_view_blocking(Rule::States(&[false, false, false, false, false, false, true, true, true, true, true, true])).is_redstone_conductor(Rule::Never),
    Block::of("minecraft:cobweb", BlockClass::WebBlock).strength(4.0, 4.0).no_collision().requires_correct_tool_for_drops().force_solid_on().push_reaction(PushReaction::Destroy),
    Block::of("minecraft:short_grass", BlockClass::TallGrassBlock).no_collision().ignited_by_lava().replaceable().offset_type(OffsetType::Xyz).push_reaction(PushReaction::Destroy),
    Block::of("minecraft:fern", BlockClass::TallGrassBlock).no_collision().ignited_by_lava().replaceable().offset_type(OffsetType::Xyz).push_reaction(PushReaction::Destroy),
    Block::of("minecraft:dead_bush", BlockClass::DryVegetationBlock).no_collision().ignited_by_lava().replaceable().push_reaction(PushReaction::Destroy),
    Block::of("minecraft:bush", BlockClass::BushBlock).no_collision().ignited_by_lava().replaceable().push_reaction(PushReaction::Destroy),
    Block::of("minecraft:short_dry_grass", BlockClass::ShortDryGrassBlock).no_collision().ignited_by_lava().replaceable().offset_type(OffsetType::Xyz).push_reaction(PushReaction::Destroy),
    Block::of("minecraft:tall_dry_grass", BlockClass::TallDryGrassBlock).no_collision().ignited_by_lava().replaceable().offset_type(OffsetType::Xyz).push_reaction(PushReaction::Destroy),
    Block::of("minecraft:seagrass", BlockClass::SeagrassBlock).no_collision().replaceable().push_reaction(PushReaction::Destroy),
    Block::of("minecraft:tall_seagrass", BlockClass::TallSeagrassBlock).properties(&[&DOUBLE_BLOCK_HALF], &["lower"]).no_collision().replaceable().offset_type(OffsetType::Xz).push_reaction(PushReaction::Destroy).item("minecraft:air"),
    Block::of("minecraft:piston", BlockClass::PistonBaseBlock).properties(&[&EXTENDED, &FACING], &["false", "north"]).strength(1.5, 1.5).push_reaction(PushReaction::Block).is_suffocating(Rule::States(&[false, false, false, false, false, false, true, true, true, true, true, true])).is_view_blocking(Rule::States(&[false, false, false, false, false, false, true, true, true, true, true, true])).is_redstone_conductor(Rule::Never),
    Block::of("minecraft:piston_head", BlockClass::PistonHeadBlock).properties(&[&FACING, &SHORT, &PISTON_TYPE], &["north", "false", "normal"]).strength(1.5, 1.5).push_reaction(PushReaction::Block).item("minecraft:air"),
    Block::of("minecraft:white_wool", BlockClass::Block).strength(0.8, 0.8).ignited_by_lava().instrument("guitar"),
    Block::of("minecraft:orange_wool", BlockClass::Block).strength(0.8, 0.8).ignited_by_lava().instrument("guitar"),
    Block::of("minecraft:magenta_wool", BlockClass::Block).strength(0.8, 0.8).ignited_by_lava().instrument("guitar"),
    Block::of("minecraft:light_blue_wool", BlockClass::Block).strength(0.8, 0.8).ignited_by_lava().instrument("guitar"),
    Block::of("minecraft:yellow_wool", BlockClass::Block).strength(0.8, 0.8).ignited_by_lava().instrument("guitar"),
    Block::of("minecraft:lime_wool", BlockClass::Block).strength(0.8, 0.8).ignited_by_lava().instrument("guitar"),
    Block::of("minecraft:pink_wool", BlockClass::Block).strength(0.8, 0.8).ignited_by_lava().instrument("guitar"),
    Block::of("minecraft:gray_wool", BlockClass::Block).strength(0.8, 0.8).ignited_by_lava().instrument("guitar"),
    Block::of("minecraft:light_gray_wool", BlockClass::Block).strength(0.8, 0.8).ignited_by_lava().instrument("guitar"),
    Block::of("minecraft:cyan_wool", BlockClass::Block).strength(0.8, 0.8).ignited_by_lava().instrument("guitar"),
    Block::of("minecraft:purple_wool", BlockClass::Block).strength(0.8, 0.8).ignited_by_lava().instrument("guitar"),
    Block::of("minecraft:blue_wool", BlockClass::Block).strength(0.8, 0.8).ignited_by_lava().instrument("guitar"),
    Block::of("minecraft:brown_wool", BlockClass::Block).strength(0.8, 0.8).ignited_by_lava().instrument("guitar"),
    Block::of("minecraft:green_wool", BlockClass::Block).strength(0.8, 0.8).ignited_by_lava().instrument("guitar"),
    Block::of("minecraft:red_wool", BlockClass::Block).strength(0.8, 0.8).ignited_by_lava().instrument("guitar"),
    Block::of("minecraft:black_wool", BlockClass::Block).strength(0.8, 0.8).ignited_by_lava().instrument("guitar"),
    Block::of("minecraft:moving_piston", BlockClass::MovingPistonBlock).properties(&[&FACING, &PISTON_TYPE], &["north", "normal"]).strength(-1.0, 0.0).no_occlusion().force_solid_on().dynamic_shape().push_reaction(PushReaction::Block).item("minecraft:air").is_suffocating(Rule::Never).is_view_blocking(Rule::Never).is_redstone_conductor(Rule::Never),
    Block::of("minecraft:dandelion", BlockClass::FlowerBlock).no_collision().offset_type(OffsetType::Xz).push_reaction(PushReaction::Destroy),
    Block::of("minecraft:golden_dandelion", BlockClass::FlowerBlock).no_collision().offset_type(OffsetType::Xz).push_reaction(PushReaction::Destroy),
    Block::of("minecraft:torchflower", BlockClass::FlowerBlock).no_collision().offset_type(OffsetType::Xz).push_reaction(PushReaction::Destroy),
    Block::of("minecraft:poppy", BlockClass::FlowerBlock).no_collision().offset_type(OffsetType::Xz).push_reaction(PushReaction::Destroy),
    Block::of("minecraft:blue_orchid", BlockClass::FlowerBlock).no_collision().offset_type(OffsetType::Xz).push_reaction(PushReaction::Destroy),
    Block::of("minecraft:allium", BlockClass::FlowerBlock).no_collision().offset_type(OffsetType::Xz).push_reaction(PushReaction::Destroy),
    Block::of("minecraft:azure_bluet", BlockClass::FlowerBlock).no_collision().offset_type(OffsetType::Xz).push_reaction(PushReaction::Destroy),
    Block::of("minecraft:red_tulip", BlockClass::FlowerBlock).no_collision().offset_type(OffsetType::Xz).push_reaction(PushReaction::Destroy),
    Block::of("minecraft:orange_tulip", BlockClass::FlowerBlock).no_collision().offset_type(OffsetType::Xz).push_reaction(PushReaction::Destroy),
    Block::of("minecraft:white_tulip", BlockClass::FlowerBlock).no_collision().offset_type(OffsetType::Xz).push_reaction(PushReaction::Destroy),
    Block::of("minecraft:pink_tulip", BlockClass::FlowerBlock).no_collision().offset_type(OffsetType::Xz).push_reaction(PushReaction::Destroy),
    Block::of("minecraft:oxeye_daisy", BlockClass::FlowerBlock).no_collision().offset_type(OffsetType::Xz).push_reaction(PushReaction::Destroy),
    Block::of("minecraft:cornflower", BlockClass::FlowerBlock).no_collision().offset_type(OffsetType::Xz).push_reaction(PushReaction::Destroy),
    Block::of("minecraft:wither_rose", BlockClass::WitherRoseBlock).no_collision().offset_type(OffsetType::Xz).push_reaction(PushReaction::Destroy),
    Block::of("minecraft:lily_of_the_valley", BlockClass::FlowerBlock).no_collision().offset_type(OffsetType::Xz).push_reaction(PushReaction::Destroy),
    Block::of("minecraft:brown_mushroom", BlockClass::MushroomBlock).no_collision().random_ticks().push_reaction(PushReaction::Destroy).light_level(Emission::Constant(1)),
    Block::of("minecraft:red_mushroom", BlockClass::MushroomBlock).no_collision().random_ticks().push_reaction(PushReaction::Destroy),
    Block::of("minecraft:gold_block", BlockClass::Block).strength(3.0, 6.0).requires_correct_tool_for_drops().instrument("bell"),
    Block::of("minecraft:iron_block", BlockClass::Block).strength(5.0, 6.0).requires_correct_tool_for_drops().instrument("iron_xylophone"),
    Block::of("minecraft:bricks", BlockClass::Block).strength(2.0, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:tnt", BlockClass::TntBlock).properties(&[&UNSTABLE], &["false"]).ignited_by_lava().is_redstone_conductor(Rule::Never),
    Block::of("minecraft:bookshelf", BlockClass::Block).strength(1.5, 1.5).ignited_by_lava().instrument("bass"),
    Block::of("minecraft:chiseled_bookshelf", BlockClass::ChiseledBookShelfBlock).properties(&[&HORIZONTAL_FACING, &SLOT_0_OCCUPIED, &SLOT_1_OCCUPIED, &SLOT_2_OCCUPIED, &SLOT_3_OCCUPIED, &SLOT_4_OCCUPIED, &SLOT_5_OCCUPIED], &["north", "false", "false", "false", "false", "false", "false"]).strength(1.5, 1.5).ignited_by_lava().instrument("bass"),
    Block::of("minecraft:acacia_shelf", BlockClass::ShelfBlock).properties(&[&HORIZONTAL_FACING, &POWERED, &SIDE_CHAIN_PART, &WATERLOGGED], &["north", "false", "unconnected", "false"]).strength(2.0, 3.0).ignited_by_lava().instrument("bass"),
    Block::of("minecraft:bamboo_shelf", BlockClass::ShelfBlock).properties(&[&HORIZONTAL_FACING, &POWERED, &SIDE_CHAIN_PART, &WATERLOGGED], &["north", "false", "unconnected", "false"]).strength(2.0, 3.0).ignited_by_lava().instrument("bass"),
    Block::of("minecraft:birch_shelf", BlockClass::ShelfBlock).properties(&[&HORIZONTAL_FACING, &POWERED, &SIDE_CHAIN_PART, &WATERLOGGED], &["north", "false", "unconnected", "false"]).strength(2.0, 3.0).ignited_by_lava().instrument("bass"),
    Block::of("minecraft:cherry_shelf", BlockClass::ShelfBlock).properties(&[&HORIZONTAL_FACING, &POWERED, &SIDE_CHAIN_PART, &WATERLOGGED], &["north", "false", "unconnected", "false"]).strength(2.0, 3.0).ignited_by_lava().instrument("bass"),
    Block::of("minecraft:crimson_shelf", BlockClass::ShelfBlock).properties(&[&HORIZONTAL_FACING, &POWERED, &SIDE_CHAIN_PART, &WATERLOGGED], &["north", "false", "unconnected", "false"]).strength(2.0, 3.0).ignited_by_lava().instrument("bass"),
    Block::of("minecraft:dark_oak_shelf", BlockClass::ShelfBlock).properties(&[&HORIZONTAL_FACING, &POWERED, &SIDE_CHAIN_PART, &WATERLOGGED], &["north", "false", "unconnected", "false"]).strength(2.0, 3.0).ignited_by_lava().instrument("bass"),
    Block::of("minecraft:jungle_shelf", BlockClass::ShelfBlock).properties(&[&HORIZONTAL_FACING, &POWERED, &SIDE_CHAIN_PART, &WATERLOGGED], &["north", "false", "unconnected", "false"]).strength(2.0, 3.0).ignited_by_lava().instrument("bass"),
    Block::of("minecraft:mangrove_shelf", BlockClass::ShelfBlock).properties(&[&HORIZONTAL_FACING, &POWERED, &SIDE_CHAIN_PART, &WATERLOGGED], &["north", "false", "unconnected", "false"]).strength(2.0, 3.0).ignited_by_lava().instrument("bass"),
    Block::of("minecraft:oak_shelf", BlockClass::ShelfBlock).properties(&[&HORIZONTAL_FACING, &POWERED, &SIDE_CHAIN_PART, &WATERLOGGED], &["north", "false", "unconnected", "false"]).strength(2.0, 3.0).ignited_by_lava().instrument("bass"),
    Block::of("minecraft:pale_oak_shelf", BlockClass::ShelfBlock).properties(&[&HORIZONTAL_FACING, &POWERED, &SIDE_CHAIN_PART, &WATERLOGGED], &["north", "false", "unconnected", "false"]).strength(2.0, 3.0).ignited_by_lava().instrument("bass"),
    Block::of("minecraft:spruce_shelf", BlockClass::ShelfBlock).properties(&[&HORIZONTAL_FACING, &POWERED, &SIDE_CHAIN_PART, &WATERLOGGED], &["north", "false", "unconnected", "false"]).strength(2.0, 3.0).ignited_by_lava().instrument("bass"),
    Block::of("minecraft:warped_shelf", BlockClass::ShelfBlock).properties(&[&HORIZONTAL_FACING, &POWERED, &SIDE_CHAIN_PART, &WATERLOGGED], &["north", "false", "unconnected", "false"]).strength(2.0, 3.0).ignited_by_lava().instrument("bass"),
    Block::of("minecraft:mossy_cobblestone", BlockClass::Block).strength(2.0, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:obsidian", BlockClass::Block).strength(50.0, 1200.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:torch", BlockClass::TorchBlock).no_collision().push_reaction(PushReaction::Destroy).light_level(Emission::Constant(14)),
    Block::of("minecraft:wall_torch", BlockClass::WallTorchBlock).properties(&[&HORIZONTAL_FACING], &["north"]).no_collision().push_reaction(PushReaction::Destroy).item("minecraft:torch").light_level(Emission::Constant(14)),
    Block::of("minecraft:fire", BlockClass::FireBlock).properties(&[&AGE_15, &EAST, &NORTH, &SOUTH, &UP, &WEST], &["0", "false", "false", "false", "false", "false"]).no_collision().replaceable().push_reaction(PushReaction::Destroy).item("minecraft:air").light_level(Emission::Constant(15)),
    Block::of("minecraft:soul_fire", BlockClass::SoulFireBlock).no_collision().replaceable().push_reaction(PushReaction::Destroy).item("minecraft:air").light_level(Emission::Constant(10)),
    Block::of("minecraft:spawner", BlockClass::SpawnerBlock).strength(5.0, 5.0).no_occlusion().requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:creaking_heart", BlockClass::CreakingHeartBlock).properties(&[&AXIS, &CREAKING_HEART_STATE, &NATURAL], &["y", "uprooted", "false"]).strength(10.0, 10.0).instrument("basedrum"),
    Block::of("minecraft:oak_stairs", BlockClass::StairBlock).properties(&[&HORIZONTAL_FACING, &HALF, &STAIRS_SHAPE, &WATERLOGGED], &["north", "bottom", "straight", "false"]).strength(2.0, 3.0).ignited_by_lava().instrument("bass"),
    Block::of("minecraft:chest", BlockClass::ChestBlock).properties(&[&HORIZONTAL_FACING, &CHEST_TYPE, &WATERLOGGED], &["north", "single", "false"]).strength(2.5, 2.5).ignited_by_lava().instrument("bass"),
    Block::of("minecraft:redstone_wire", BlockClass::RedStoneWireBlock).properties(&[&EAST_REDSTONE, &NORTH_REDSTONE, &POWER, &SOUTH_REDSTONE, &WEST_REDSTONE], &["none", "none", "0", "none", "none"]).no_collision().push_reaction(PushReaction::Destroy).item("minecraft:redstone"),
    Block::of("minecraft:diamond_ore", BlockClass::DropExperienceBlock).strength(3.0, 3.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:deepslate_diamond_ore", BlockClass::DropExperienceBlock).strength(4.5, 3.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:diamond_block", BlockClass::Block).strength(5.0, 6.0).requires_correct_tool_for_drops(),
    Block::of("minecraft:crafting_table", BlockClass::CraftingTableBlock).strength(2.5, 2.5).ignited_by_lava().instrument("bass"),
    Block::of("minecraft:wheat", BlockClass::CropBlock).properties(&[&AGE_7], &["0"]).no_collision().random_ticks().push_reaction(PushReaction::Destroy).item("minecraft:wheat_seeds"),
    Block::of("minecraft:farmland", BlockClass::FarmlandBlock).properties(&[&MOISTURE], &["0"]).strength(0.6, 0.6).random_ticks().is_suffocating(Rule::Always).is_view_blocking(Rule::Always),
    Block::of("minecraft:furnace", BlockClass::FurnaceBlock).properties(&[&HORIZONTAL_FACING, &LIT], &["north", "false"]).strength(3.5, 3.5).requires_correct_tool_for_drops().instrument("basedrum").light_level(Emission::States(&[13, 0, 13, 0, 13, 0, 13, 0])),
    Block::of("minecraft:oak_sign", BlockClass::StandingSignBlock).properties(&[&ROTATION_16, &WATERLOGGED], &["8", "false"]).strength(1.0, 1.0).no_collision().ignited_by_lava().force_solid_on().instrument("bass"),
    Block::of("minecraft:spruce_sign", BlockClass::StandingSignBlock).properties(&[&ROTATION_16, &WATERLOGGED], &["8", "false"]).strength(1.0, 1.0).no_collision().ignited_by_lava().force_solid_on().instrument("bass"),
    Block::of("minecraft:birch_sign", BlockClass::StandingSignBlock).properties(&[&ROTATION_16, &WATERLOGGED], &["8", "false"]).strength(1.0, 1.0).no_collision().ignited_by_lava().force_solid_on().instrument("bass"),
    Block::of("minecraft:acacia_sign", BlockClass::StandingSignBlock).properties(&[&ROTATION_16, &WATERLOGGED], &["8", "false"]).strength(1.0, 1.0).no_collision().ignited_by_lava().force_solid_on().instrument("bass"),
    Block::of("minecraft:cherry_sign", BlockClass::StandingSignBlock).properties(&[&ROTATION_16, &WATERLOGGED], &["8", "false"]).strength(1.0, 1.0).no_collision().ignited_by_lava().force_solid_on().instrument("bass"),
    Block::of("minecraft:jungle_sign", BlockClass::StandingSignBlock).properties(&[&ROTATION_16, &WATERLOGGED], &["8", "false"]).strength(1.0, 1.0).no_collision().ignited_by_lava().force_solid_on().instrument("bass"),
    Block::of("minecraft:dark_oak_sign", BlockClass::StandingSignBlock).properties(&[&ROTATION_16, &WATERLOGGED], &["8", "false"]).strength(1.0, 1.0).no_collision().ignited_by_lava().force_solid_on().instrument("bass"),
    Block::of("minecraft:pale_oak_sign", BlockClass::StandingSignBlock).properties(&[&ROTATION_16, &WATERLOGGED], &["8", "false"]).strength(1.0, 1.0).no_collision().ignited_by_lava().force_solid_on().instrument("bass"),
    Block::of("minecraft:mangrove_sign", BlockClass::StandingSignBlock).properties(&[&ROTATION_16, &WATERLOGGED], &["8", "false"]).strength(1.0, 1.0).no_collision().ignited_by_lava().force_solid_on().instrument("bass"),
    Block::of("minecraft:bamboo_sign", BlockClass::StandingSignBlock).properties(&[&ROTATION_16, &WATERLOGGED], &["8", "false"]).strength(1.0, 1.0).no_collision().ignited_by_lava().force_solid_on().instrument("bass"),
    Block::of("minecraft:oak_door", BlockClass::DoorBlock).properties(&[&HORIZONTAL_FACING, &DOUBLE_BLOCK_HALF, &DOOR_HINGE, &OPEN, &POWERED], &["north", "lower", "left", "false", "false"]).strength(3.0, 3.0).no_occlusion().ignited_by_lava().push_reaction(PushReaction::Destroy).instrument("bass"),
    Block::of("minecraft:ladder", BlockClass::LadderBlock).properties(&[&HORIZONTAL_FACING, &WATERLOGGED], &["north", "false"]).strength(0.4, 0.4).no_occlusion().force_solid_off().push_reaction(PushReaction::Destroy),
    Block::of("minecraft:rail", BlockClass::RailBlock).properties(&[&RAIL_SHAPE, &WATERLOGGED], &["north_south", "false"]).strength(0.7, 0.7).no_collision(),
    Block::of("minecraft:cobblestone_stairs", BlockClass::StairBlock).properties(&[&HORIZONTAL_FACING, &HALF, &STAIRS_SHAPE, &WATERLOGGED], &["north", "bottom", "straight", "false"]).strength(2.0, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:oak_wall_sign", BlockClass::WallSignBlock).properties(&[&HORIZONTAL_FACING, &WATERLOGGED], &["north", "false"]).strength(1.0, 1.0).no_collision().ignited_by_lava().force_solid_on().instrument("bass").item("minecraft:oak_sign"),
    Block::of("minecraft:spruce_wall_sign", BlockClass::WallSignBlock).properties(&[&HORIZONTAL_FACING, &WATERLOGGED], &["north", "false"]).strength(1.0, 1.0).no_collision().ignited_by_lava().force_solid_on().instrument("bass").item("minecraft:spruce_sign"),
    Block::of("minecraft:birch_wall_sign", BlockClass::WallSignBlock).properties(&[&HORIZONTAL_FACING, &WATERLOGGED], &["north", "false"]).strength(1.0, 1.0).no_collision().ignited_by_lava().force_solid_on().instrument("bass").item("minecraft:birch_sign"),
    Block::of("minecraft:acacia_wall_sign", BlockClass::WallSignBlock).properties(&[&HORIZONTAL_FACING, &WATERLOGGED], &["north", "false"]).strength(1.0, 1.0).no_collision().ignited_by_lava().force_solid_on().instrument("bass").item("minecraft:acacia_sign"),
    Block::of("minecraft:cherry_wall_sign", BlockClass::WallSignBlock).properties(&[&HORIZONTAL_FACING, &WATERLOGGED], &["north", "false"]).strength(1.0, 1.0).no_collision().ignited_by_lava().force_solid_on().instrument("bass").item("minecraft:cherry_sign"),
    Block::of("minecraft:jungle_wall_sign", BlockClass::WallSignBlock).properties(&[&HORIZONTAL_FACING, &WATERLOGGED], &["north", "false"]).strength(1.0, 1.0).no_collision().ignited_by_lava().force_solid_on().instrument("bass").item("minecraft:jungle_sign"),
    Block::of("minecraft:dark_oak_wall_sign", BlockClass::WallSignBlock).properties(&[&HORIZONTAL_FACING, &WATERLOGGED], &["north", "false"]).strength(1.0, 1.0).no_collision().ignited_by_lava().force_solid_on().instrument("bass").item("minecraft:dark_oak_sign"),
    Block::of("minecraft:pale_oak_wall_sign", BlockClass::WallSignBlock).properties(&[&HORIZONTAL_FACING, &WATERLOGGED], &["north", "false"]).strength(1.0, 1.0).no_collision().ignited_by_lava().force_solid_on().instrument("bass").item("minecraft:pale_oak_sign"),
    Block::of("minecraft:mangrove_wall_sign", BlockClass::WallSignBlock).properties(&[&HORIZONTAL_FACING, &WATERLOGGED], &["north", "false"]).strength(1.0, 1.0).no_collision().ignited_by_lava().force_solid_on().instrument("bass").item("minecraft:mangrove_sign"),
    Block::of("minecraft:bamboo_wall_sign", BlockClass::WallSignBlock).properties(&[&HORIZONTAL_FACING, &WATERLOGGED], &["north", "false"]).strength(1.0, 1.0).no_collision().ignited_by_lava().force_solid_on().instrument("bass").item("minecraft:bamboo_sign"),
    Block::of("minecraft:oak_hanging_sign", BlockClass::CeilingHangingSignBlock).properties(&[&ATTACHED, &ROTATION_16, &WATERLOGGED], &["false", "8", "false"]).strength(1.0, 1.0).no_collision().ignited_by_lava().force_solid_on().instrument("bass"),
    Block::of("minecraft:spruce_hanging_sign", BlockClass::CeilingHangingSignBlock).properties(&[&ATTACHED, &ROTATION_16, &WATERLOGGED], &["false", "8", "false"]).strength(1.0, 1.0).no_collision().ignited_by_lava().force_solid_on().instrument("bass"),
    Block::of("minecraft:birch_hanging_sign", BlockClass::CeilingHangingSignBlock).properties(&[&ATTACHED, &ROTATION_16, &WATERLOGGED], &["false", "8", "false"]).strength(1.0, 1.0).no_collision().ignited_by_lava().force_solid_on().instrument("bass"),
    Block::of("minecraft:acacia_hanging_sign", BlockClass::CeilingHangingSignBlock).properties(&[&ATTACHED, &ROTATION_16, &WATERLOGGED], &["false", "8", "false"]).strength(1.0, 1.0).no_collision().ignited_by_lava().force_solid_on().instrument("bass"),
    Block::of("minecraft:cherry_hanging_sign", BlockClass::CeilingHangingSignBlock).properties(&[&ATTACHED, &ROTATION_16, &WATERLOGGED], &["false", "8", "false"]).strength(1.0, 1.0).no_collision().ignited_by_lava().force_solid_on().instrument("bass"),
    Block::of("minecraft:jungle_hanging_sign", BlockClass::CeilingHangingSignBlock).properties(&[&ATTACHED, &ROTATION_16, &WATERLOGGED], &["false", "8", "false"]).strength(1.0, 1.0).no_collision().ignited_by_lava().force_solid_on().instrument("bass"),
    Block::of("minecraft:dark_oak_hanging_sign", BlockClass::CeilingHangingSignBlock).properties(&[&ATTACHED, &ROTATION_16, &WATERLOGGED], &["false", "8", "false"]).strength(1.0, 1.0).no_collision().ignited_by_lava().force_solid_on().instrument("bass"),
    Block::of("minecraft:pale_oak_hanging_sign", BlockClass::CeilingHangingSignBlock).properties(&[&ATTACHED, &ROTATION_16, &WATERLOGGED], &["false", "8", "false"]).strength(1.0, 1.0).no_collision().ignited_by_lava().force_solid_on().instrument("bass"),
    Block::of("minecraft:crimson_hanging_sign", BlockClass::CeilingHangingSignBlock).properties(&[&ATTACHED, &ROTATION_16, &WATERLOGGED], &["false", "8", "false"]).strength(1.0, 1.0).no_collision().force_solid_on().instrument("bass"),
    Block::of("minecraft:warped_hanging_sign", BlockClass::CeilingHangingSignBlock).properties(&[&ATTACHED, &ROTATION_16, &WATERLOGGED], &["false", "8", "false"]).strength(1.0, 1.0).no_collision().force_solid_on().instrument("bass"),
    Block::of("minecraft:mangrove_hanging_sign", BlockClass::CeilingHangingSignBlock).properties(&[&ATTACHED, &ROTATION_16, &WATERLOGGED], &["false", "8", "false"]).strength(1.0, 1.0).no_collision().ignited_by_lava().force_solid_on().instrument("bass"),
    Block::of("minecraft:bamboo_hanging_sign", BlockClass::CeilingHangingSignBlock).properties(&[&ATTACHED, &ROTATION_16, &WATERLOGGED], &["false", "8", "false"]).strength(1.0, 1.0).no_collision().ignited_by_lava().force_solid_on().instrument("bass"),
    Block::of("minecraft:oak_wall_hanging_sign", BlockClass::WallHangingSignBlock).properties(&[&HORIZONTAL_FACING, &WATERLOGGED], &["north", "false"]).strength(1.0, 1.0).no_collision().ignited_by_lava().force_solid_on().instrument("bass").item("minecraft:oak_hanging_sign"),
    Block::of("minecraft:spruce_wall_hanging_sign", BlockClass::WallHangingSignBlock).properties(&[&HORIZONTAL_FACING, &WATERLOGGED], &["north", "false"]).strength(1.0, 1.0).no_collision().ignited_by_lava().force_solid_on().instrument("bass").item("minecraft:spruce_hanging_sign"),
    Block::of("minecraft:birch_wall_hanging_sign", BlockClass::WallHangingSignBlock).properties(&[&HORIZONTAL_FACING, &WATERLOGGED], &["north", "false"]).strength(1.0, 1.0).no_collision().ignited_by_lava().force_solid_on().instrument("bass").item("minecraft:birch_hanging_sign"),
    Block::of("minecraft:acacia_wall_hanging_sign", BlockClass::WallHangingSignBlock).properties(&[&HORIZONTAL_FACING, &WATERLOGGED], &["north", "false"]).strength(1.0, 1.0).no_collision().ignited_by_lava().force_solid_on().instrument("bass").item("minecraft:acacia_hanging_sign"),
    Block::of("minecraft:cherry_wall_hanging_sign", BlockClass::WallHangingSignBlock).properties(&[&HORIZONTAL_FACING, &WATERLOGGED], &["north", "false"]).strength(1.0, 1.0).no_collision().ignited_by_lava().force_solid_on().instrument("bass").item("minecraft:cherry_hanging_sign"),
    Block::of("minecraft:jungle_wall_hanging_sign", BlockClass::WallHangingSignBlock).properties(&[&HORIZONTAL_FACING, &WATERLOGGED], &["north", "false"]).strength(1.0, 1.0).no_collision().ignited_by_lava().force_solid_on().instrument("bass").item("minecraft:jungle_hanging_sign"),
    Block::of("minecraft:dark_oak_wall_hanging_sign", BlockClass::WallHangingSignBlock).properties(&[&HORIZONTAL_FACING, &WATERLOGGED], &["north", "false"]).strength(1.0, 1.0).no_collision().ignited_by_lava().force_solid_on().instrument("bass").item("minecraft:dark_oak_hanging_sign"),
    Block::of("minecraft:pale_oak_wall_hanging_sign", BlockClass::WallHangingSignBlock).properties(&[&HORIZONTAL_FACING, &WATERLOGGED], &["north", "false"]).strength(1.0, 1.0).no_collision().ignited_by_lava().force_solid_on().instrument("bass").item("minecraft:pale_oak_hanging_sign"),
    Block::of("minecraft:mangrove_wall_hanging_sign", BlockClass::WallHangingSignBlock).properties(&[&HORIZONTAL_FACING, &WATERLOGGED], &["north", "false"]).strength(1.0, 1.0).no_collision().ignited_by_lava().force_solid_on().instrument("bass").item("minecraft:mangrove_hanging_sign"),
    Block::of("minecraft:crimson_wall_hanging_sign", BlockClass::WallHangingSignBlock).properties(&[&HORIZONTAL_FACING, &WATERLOGGED], &["north", "false"]).strength(1.0, 1.0).no_collision().force_solid_on().instrument("bass").item("minecraft:crimson_hanging_sign"),
    Block::of("minecraft:warped_wall_hanging_sign", BlockClass::WallHangingSignBlock).properties(&[&HORIZONTAL_FACING, &WATERLOGGED], &["north", "false"]).strength(1.0, 1.0).no_collision().force_solid_on().instrument("bass").item("minecraft:warped_hanging_sign"),
    Block::of("minecraft:bamboo_wall_hanging_sign", BlockClass::WallHangingSignBlock).properties(&[&HORIZONTAL_FACING, &WATERLOGGED], &["north", "false"]).strength(1.0, 1.0).no_collision().ignited_by_lava().force_solid_on().instrument("bass").item("minecraft:bamboo_hanging_sign"),
    Block::of("minecraft:lever", BlockClass::LeverBlock).properties(&[&ATTACH_FACE, &HORIZONTAL_FACING, &POWERED], &["wall", "north", "false"]).strength(0.5, 0.5).no_collision().push_reaction(PushReaction::Destroy),
    Block::of("minecraft:stone_pressure_plate", BlockClass::PressurePlateBlock).properties(&[&POWERED], &["false"]).strength(0.5, 0.5).no_collision().force_solid_on().push_reaction(PushReaction::Destroy).instrument("basedrum"),
    Block::of("minecraft:iron_door", BlockClass::DoorBlock).properties(&[&HORIZONTAL_FACING, &DOUBLE_BLOCK_HALF, &DOOR_HINGE, &OPEN, &POWERED], &["north", "lower", "left", "false", "false"]).strength(5.0, 5.0).no_occlusion().push_reaction(PushReaction::Destroy),
    Block::of("minecraft:oak_pressure_plate", BlockClass::PressurePlateBlock).properties(&[&POWERED], &["false"]).strength(0.5, 0.5).no_collision().ignited_by_lava().force_solid_on().push_reaction(PushReaction::Destroy).instrument("bass"),
    Block::of("minecraft:spruce_pressure_plate", BlockClass::PressurePlateBlock).properties(&[&POWERED], &["false"]).strength(0.5, 0.5).no_collision().ignited_by_lava().force_solid_on().push_reaction(PushReaction::Destroy).instrument("bass"),
    Block::of("minecraft:birch_pressure_plate", BlockClass::PressurePlateBlock).properties(&[&POWERED], &["false"]).strength(0.5, 0.5).no_collision().ignited_by_lava().force_solid_on().push_reaction(PushReaction::Destroy).instrument("bass"),
    Block::of("minecraft:jungle_pressure_plate", BlockClass::PressurePlateBlock).properties(&[&POWERED], &["false"]).strength(0.5, 0.5).no_collision().ignited_by_lava().force_solid_on().push_reaction(PushReaction::Destroy).instrument("bass"),
    Block::of("minecraft:acacia_pressure_plate", BlockClass::PressurePlateBlock).properties(&[&POWERED], &["false"]).strength(0.5, 0.5).no_collision().ignited_by_lava().force_solid_on().push_reaction(PushReaction::Destroy).instrument("bass"),
    Block::of("minecraft:cherry_pressure_plate", BlockClass::PressurePlateBlock).properties(&[&POWERED], &["false"]).strength(0.5, 0.5).no_collision().ignited_by_lava().force_solid_on().push_reaction(PushReaction::Destroy).instrument("bass"),
    Block::of("minecraft:dark_oak_pressure_plate", BlockClass::PressurePlateBlock).properties(&[&POWERED], &["false"]).strength(0.5, 0.5).no_collision().ignited_by_lava().force_solid_on().push_reaction(PushReaction::Destroy).instrument("bass"),
    Block::of("minecraft:pale_oak_pressure_plate", BlockClass::PressurePlateBlock).properties(&[&POWERED], &["false"]).strength(0.5, 0.5).no_collision().ignited_by_lava().force_solid_on().push_reaction(PushReaction::Destroy).instrument("bass"),
    Block::of("minecraft:mangrove_pressure_plate", BlockClass::PressurePlateBlock).properties(&[&POWERED], &["false"]).strength(0.5, 0.5).no_collision().ignited_by_lava().force_solid_on().push_reaction(PushReaction::Destroy).instrument("bass"),
    Block::of("minecraft:bamboo_pressure_plate", BlockClass::PressurePlateBlock).properties(&[&POWERED], &["false"]).strength(0.5, 0.5).no_collision().ignited_by_lava().force_solid_on().push_reaction(PushReaction::Destroy).instrument("bass"),
    Block::of("minecraft:redstone_ore", BlockClass::RedStoneOreBlock).properties(&[&LIT], &["false"]).strength(3.0, 3.0).requires_correct_tool_for_drops().random_ticks().instrument("basedrum").light_level(Emission::States(&[9, 0])),
    Block::of("minecraft:deepslate_redstone_ore", BlockClass::RedStoneOreBlock).properties(&[&LIT], &["false"]).strength(4.5, 3.0).requires_correct_tool_for_drops().random_ticks().instrument("basedrum").light_level(Emission::States(&[9, 0])),
    Block::of("minecraft:redstone_torch", BlockClass::RedstoneTorchBlock).properties(&[&LIT], &["true"]).no_collision().push_reaction(PushReaction::Destroy).light_level(Emission::States(&[7, 0])),
    Block::of("minecraft:redstone_wall_torch", BlockClass::RedstoneWallTorchBlock).properties(&[&HORIZONTAL_FACING, &LIT], &["north", "true"]).no_collision().push_reaction(PushReaction::Destroy).item("minecraft:redstone_torch").light_level(Emission::States(&[7, 0, 7, 0, 7, 0, 7, 0])),
    Block::of("minecraft:stone_button", BlockClass::ButtonBlock).properties(&[&ATTACH_FACE, &HORIZONTAL_FACING, &POWERED], &["wall", "north", "false"]).strength(0.5, 0.5).no_collision().push_reaction(PushReaction::Destroy),
    Block::of("minecraft:snow", BlockClass::SnowLayerBlock).properties(&[&LAYERS], &["1"]).strength(0.1, 0.1).requires_correct_tool_for_drops().random_ticks().force_solid_off().replaceable().push_reaction(PushReaction::Destroy).is_view_blocking(Rule::States(&[false, false, false, false, false, false, false, true])),
    Block::of("minecraft:ice", BlockClass::IceBlock).strength(0.5, 0.5).friction(0.98).no_occlusion().random_ticks().is_redstone_conductor(Rule::Never),
    Block::of("minecraft:snow_block", BlockClass::Block).strength(0.2, 0.2).requires_correct_tool_for_drops(),
    Block::of("minecraft:cactus", BlockClass::CactusBlock).properties(&[&AGE_15], &["0"]).strength(0.4, 0.4).random_ticks().push_reaction(PushReaction::Destroy),
    Block::of("minecraft:cactus_flower", BlockClass::CactusFlowerBlock).no_collision().ignited_by_lava().push_reaction(PushReaction::Destroy),
    Block::of("minecraft:clay", BlockClass::Block).strength(0.6, 0.6).instrument("flute"),
    Block::of("minecraft:sugar_cane", BlockClass::SugarCaneBlock).properties(&[&AGE_15], &["0"]).no_collision().random_ticks().push_reaction(PushReaction::Destroy),
    Block::of("minecraft:jukebox", BlockClass::JukeboxBlock).properties(&[&HAS_RECORD], &["false"]).strength(2.0, 6.0).ignited_by_lava().instrument("bass"),
    Block::of("minecraft:oak_fence", BlockClass::FenceBlock).properties(&[&EAST, &NORTH, &SOUTH, &WATERLOGGED, &WEST], &["false", "false", "false", "false", "false"]).strength(2.0, 3.0).ignited_by_lava().force_solid_on().instrument("bass"),
    Block::of("minecraft:netherrack", BlockClass::NetherrackBlock).strength(0.4, 0.4).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:soul_sand", BlockClass::SoulSandBlock).strength(0.5, 0.5).speed_factor(0.4).instrument("cow_bell").is_suffocating(Rule::Always).is_view_blocking(Rule::Always).is_redstone_conductor(Rule::Always),
    Block::of("minecraft:soul_soil", BlockClass::Block).strength(0.5, 0.5),
    Block::of("minecraft:basalt", BlockClass::RotatedPillarBlock).properties(&[&AXIS], &["y"]).strength(1.25, 4.2).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:polished_basalt", BlockClass::RotatedPillarBlock).properties(&[&AXIS], &["y"]).strength(1.25, 4.2).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:soul_torch", BlockClass::TorchBlock).no_collision().push_reaction(PushReaction::Destroy).light_level(Emission::Constant(10)),
    Block::of("minecraft:soul_wall_torch", BlockClass::WallTorchBlock).properties(&[&HORIZONTAL_FACING], &["north"]).no_collision().push_reaction(PushReaction::Destroy).item("minecraft:soul_torch").light_level(Emission::Constant(10)),
    Block::of("minecraft:copper_torch", BlockClass::TorchBlock).no_collision().push_reaction(PushReaction::Destroy).light_level(Emission::Constant(14)),
    Block::of("minecraft:copper_wall_torch", BlockClass::WallTorchBlock).properties(&[&HORIZONTAL_FACING], &["north"]).no_collision().push_reaction(PushReaction::Destroy).item("minecraft:copper_torch").light_level(Emission::Constant(14)),
    Block::of("minecraft:glowstone", BlockClass::Block).strength(0.3, 0.3).instrument("pling").light_level(Emission::Constant(15)).is_redstone_conductor(Rule::Never),
    Block::of("minecraft:nether_portal", BlockClass::NetherPortalBlock).properties(&[&HORIZONTAL_AXIS], &["x"]).strength(-1.0, 0.0).no_collision().random_ticks().push_reaction(PushReaction::Block).item("minecraft:air").light_level(Emission::Constant(11)),
    Block::of("minecraft:carved_pumpkin", BlockClass::CarvedPumpkinBlock).properties(&[&HORIZONTAL_FACING], &["north"]).strength(1.0, 1.0).push_reaction(PushReaction::Destroy),
    Block::of("minecraft:jack_o_lantern", BlockClass::CarvedPumpkinBlock).properties(&[&HORIZONTAL_FACING], &["north"]).strength(1.0, 1.0).push_reaction(PushReaction::Destroy).light_level(Emission::Constant(15)),
    Block::of("minecraft:cake", BlockClass::CakeBlock).properties(&[&BITES], &["0"]).strength(0.5, 0.5).force_solid_on().push_reaction(PushReaction::Destroy),
    Block::of("minecraft:repeater", BlockClass::RepeaterBlock).properties(&[&DELAY, &HORIZONTAL_FACING, &LOCKED, &POWERED], &["1", "north", "false", "false"]).push_reaction(PushReaction::Destroy),
    Block::of("minecraft:white_stained_glass", BlockClass::StainedGlassBlock).strength(0.3, 0.3).no_occlusion().instrument("hat").is_suffocating(Rule::Never).is_view_blocking(Rule::Never).is_redstone_conductor(Rule::Never),
    Block::of("minecraft:orange_stained_glass", BlockClass::StainedGlassBlock).strength(0.3, 0.3).no_occlusion().instrument("hat").is_suffocating(Rule::Never).is_view_blocking(Rule::Never).is_redstone_conductor(Rule::Never),
    Block::of("minecraft:magenta_stained_glass", BlockClass::StainedGlassBlock).strength(0.3, 0.3).no_occlusion().instrument("hat").is_suffocating(Rule::Never).is_view_blocking(Rule::Never).is_redstone_conductor(Rule::Never),
    Block::of("minecraft:light_blue_stained_glass", BlockClass::StainedGlassBlock).strength(0.3, 0.3).no_occlusion().instrument("hat").is_suffocating(Rule::Never).is_view_blocking(Rule::Never).is_redstone_conductor(Rule::Never),
    Block::of("minecraft:yellow_stained_glass", BlockClass::StainedGlassBlock).strength(0.3, 0.3).no_occlusion().instrument("hat").is_suffocating(Rule::Never).is_view_blocking(Rule::Never).is_redstone_conductor(Rule::Never),
    Block::of("minecraft:lime_stained_glass", BlockClass::StainedGlassBlock).strength(0.3, 0.3).no_occlusion().instrument("hat").is_suffocating(Rule::Never).is_view_blocking(Rule::Never).is_redstone_conductor(Rule::Never),
    Block::of("minecraft:pink_stained_glass", BlockClass::StainedGlassBlock).strength(0.3, 0.3).no_occlusion().instrument("hat").is_suffocating(Rule::Never).is_view_blocking(Rule::Never).is_redstone_conductor(Rule::Never),
    Block::of("minecraft:gray_stained_glass", BlockClass::StainedGlassBlock).strength(0.3, 0.3).no_occlusion().instrument("hat").is_suffocating(Rule::Never).is_view_blocking(Rule::Never).is_redstone_conductor(Rule::Never),
    Block::of("minecraft:light_gray_stained_glass", BlockClass::StainedGlassBlock).strength(0.3, 0.3).no_occlusion().instrument("hat").is_suffocating(Rule::Never).is_view_blocking(Rule::Never).is_redstone_conductor(Rule::Never),
    Block::of("minecraft:cyan_stained_glass", BlockClass::StainedGlassBlock).strength(0.3, 0.3).no_occlusion().instrument("hat").is_suffocating(Rule::Never).is_view_blocking(Rule::Never).is_redstone_conductor(Rule::Never),
    Block::of("minecraft:purple_stained_glass", BlockClass::StainedGlassBlock).strength(0.3, 0.3).no_occlusion().instrument("hat").is_suffocating(Rule::Never).is_view_blocking(Rule::Never).is_redstone_conductor(Rule::Never),
    Block::of("minecraft:blue_stained_glass", BlockClass::StainedGlassBlock).strength(0.3, 0.3).no_occlusion().instrument("hat").is_suffocating(Rule::Never).is_view_blocking(Rule::Never).is_redstone_conductor(Rule::Never),
    Block::of("minecraft:brown_stained_glass", BlockClass::StainedGlassBlock).strength(0.3, 0.3).no_occlusion().instrument("hat").is_suffocating(Rule::Never).is_view_blocking(Rule::Never).is_redstone_conductor(Rule::Never),
    Block::of("minecraft:green_stained_glass", BlockClass::StainedGlassBlock).strength(0.3, 0.3).no_occlusion().instrument("hat").is_suffocating(Rule::Never).is_view_blocking(Rule::Never).is_redstone_conductor(Rule::Never),
    Block::of("minecraft:red_stained_glass", BlockClass::StainedGlassBlock).strength(0.3, 0.3).no_occlusion().instrument("hat").is_suffocating(Rule::Never).is_view_blocking(Rule::Never).is_redstone_conductor(Rule::Never),
    Block::of("minecraft:black_stained_glass", BlockClass::StainedGlassBlock).strength(0.3, 0.3).no_occlusion().instrument("hat").is_suffocating(Rule::Never).is_view_blocking(Rule::Never).is_redstone_conductor(Rule::Never),
    Block::of("minecraft:oak_trapdoor", BlockClass::TrapDoorBlock).properties(&[&HORIZONTAL_FACING, &HALF, &OPEN, &POWERED, &WATERLOGGED], &["north", "bottom", "false", "false", "false"]).strength(3.0, 3.0).no_occlusion().ignited_by_lava().instrument("bass"),
    Block::of("minecraft:spruce_trapdoor", BlockClass::TrapDoorBlock).properties(&[&HORIZONTAL_FACING, &HALF, &OPEN, &POWERED, &WATERLOGGED], &["north", "bottom", "false", "false", "false"]).strength(3.0, 3.0).no_occlusion().ignited_by_lava().instrument("bass"),
    Block::of("minecraft:birch_trapdoor", BlockClass::TrapDoorBlock).properties(&[&HORIZONTAL_FACING, &HALF, &OPEN, &POWERED, &WATERLOGGED], &["north", "bottom", "false", "false", "false"]).strength(3.0, 3.0).no_occlusion().ignited_by_lava().instrument("bass"),
    Block::of("minecraft:jungle_trapdoor", BlockClass::TrapDoorBlock).properties(&[&HORIZONTAL_FACING, &HALF, &OPEN, &POWERED, &WATERLOGGED], &["north", "bottom", "false", "false", "false"]).strength(3.0, 3.0).no_occlusion().ignited_by_lava().instrument("bass"),
    Block::of("minecraft:acacia_trapdoor", BlockClass::TrapDoorBlock).properties(&[&HORIZONTAL_FACING, &HALF, &OPEN, &POWERED, &WATERLOGGED], &["north", "bottom", "false", "false", "false"]).strength(3.0, 3.0).no_occlusion().ignited_by_lava().instrument("bass"),
    Block::of("minecraft:cherry_trapdoor", BlockClass::TrapDoorBlock).properties(&[&HORIZONTAL_FACING, &HALF, &OPEN, &POWERED, &WATERLOGGED], &["north", "bottom", "false", "false", "false"]).strength(3.0, 3.0).no_occlusion().ignited_by_lava().instrument("bass"),
    Block::of("minecraft:dark_oak_trapdoor", BlockClass::TrapDoorBlock).properties(&[&HORIZONTAL_FACING, &HALF, &OPEN, &POWERED, &WATERLOGGED], &["north", "bottom", "false", "false", "false"]).strength(3.0, 3.0).no_occlusion().ignited_by_lava().instrument("bass"),
    Block::of("minecraft:pale_oak_trapdoor", BlockClass::TrapDoorBlock).properties(&[&HORIZONTAL_FACING, &HALF, &OPEN, &POWERED, &WATERLOGGED], &["north", "bottom", "false", "false", "false"]).strength(3.0, 3.0).no_occlusion().ignited_by_lava().instrument("bass"),
    Block::of("minecraft:mangrove_trapdoor", BlockClass::TrapDoorBlock).properties(&[&HORIZONTAL_FACING, &HALF, &OPEN, &POWERED, &WATERLOGGED], &["north", "bottom", "false", "false", "false"]).strength(3.0, 3.0).no_occlusion().ignited_by_lava().instrument("bass"),
    Block::of("minecraft:bamboo_trapdoor", BlockClass::TrapDoorBlock).properties(&[&HORIZONTAL_FACING, &HALF, &OPEN, &POWERED, &WATERLOGGED], &["north", "bottom", "false", "false", "false"]).strength(3.0, 3.0).no_occlusion().ignited_by_lava().instrument("bass"),
    Block::of("minecraft:stone_bricks", BlockClass::Block).strength(1.5, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:mossy_stone_bricks", BlockClass::Block).strength(1.5, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:cracked_stone_bricks", BlockClass::Block).strength(1.5, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:chiseled_stone_bricks", BlockClass::Block).strength(1.5, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:packed_mud", BlockClass::Block).strength(1.0, 3.0),
    Block::of("minecraft:mud_bricks", BlockClass::Block).strength(1.5, 3.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:infested_stone", BlockClass::InfestedBlock).strength(0.75, 0.75),
    Block::of("minecraft:infested_cobblestone", BlockClass::InfestedBlock).strength(1.0, 0.75),
    Block::of("minecraft:infested_stone_bricks", BlockClass::InfestedBlock).strength(0.75, 0.75),
    Block::of("minecraft:infested_mossy_stone_bricks", BlockClass::InfestedBlock).strength(0.75, 0.75),
    Block::of("minecraft:infested_cracked_stone_bricks", BlockClass::InfestedBlock).strength(0.75, 0.75),
    Block::of("minecraft:infested_chiseled_stone_bricks", BlockClass::InfestedBlock).strength(0.75, 0.75),
    Block::of("minecraft:brown_mushroom_block", BlockClass::HugeMushroomBlock).properties(&[&DOWN, &EAST, &NORTH, &SOUTH, &UP, &WEST], &["true", "true", "true", "true", "true", "true"]).strength(0.2, 0.2).ignited_by_lava().instrument("bass"),
    Block::of("minecraft:red_mushroom_block", BlockClass::HugeMushroomBlock).properties(&[&DOWN, &EAST, &NORTH, &SOUTH, &UP, &WEST], &["true", "true", "true", "true", "true", "true"]).strength(0.2, 0.2).ignited_by_lava().instrument("bass"),
    Block::of("minecraft:mushroom_stem", BlockClass::HugeMushroomBlock).properties(&[&DOWN, &EAST, &NORTH, &SOUTH, &UP, &WEST], &["true", "true", "true", "true", "true", "true"]).strength(0.2, 0.2).ignited_by_lava().instrument("bass"),
    Block::of("minecraft:iron_bars", BlockClass::IronBarsBlock).properties(&[&EAST, &NORTH, &SOUTH, &WATERLOGGED, &WEST], &["false", "false", "false", "false", "false"]).strength(5.0, 6.0).no_occlusion().requires_correct_tool_for_drops(),
    Block::of("minecraft:copper_bars", BlockClass::WeatheringCopperBarsBlock).properties(&[&EAST, &NORTH, &SOUTH, &WATERLOGGED, &WEST], &["false", "false", "false", "false", "false"]).strength(5.0, 6.0).no_occlusion().requires_correct_tool_for_drops(),
    Block::of("minecraft:exposed_copper_bars", BlockClass::WeatheringCopperBarsBlock).properties(&[&EAST, &NORTH, &SOUTH, &WATERLOGGED, &WEST], &["false", "false", "false", "false", "false"]).strength(5.0, 6.0).no_occlusion().requires_correct_tool_for_drops(),
    Block::of("minecraft:weathered_copper_bars", BlockClass::WeatheringCopperBarsBlock).properties(&[&EAST, &NORTH, &SOUTH, &WATERLOGGED, &WEST], &["false", "false", "false", "false", "false"]).strength(5.0, 6.0).no_occlusion().requires_correct_tool_for_drops(),
    Block::of("minecraft:oxidized_copper_bars", BlockClass::WeatheringCopperBarsBlock).properties(&[&EAST, &NORTH, &SOUTH, &WATERLOGGED, &WEST], &["false", "false", "false", "false", "false"]).strength(5.0, 6.0).no_occlusion().requires_correct_tool_for_drops(),
    Block::of("minecraft:waxed_copper_bars", BlockClass::IronBarsBlock).properties(&[&EAST, &NORTH, &SOUTH, &WATERLOGGED, &WEST], &["false", "false", "false", "false", "false"]).strength(5.0, 6.0).no_occlusion().requires_correct_tool_for_drops(),
    Block::of("minecraft:waxed_exposed_copper_bars", BlockClass::IronBarsBlock).properties(&[&EAST, &NORTH, &SOUTH, &WATERLOGGED, &WEST], &["false", "false", "false", "false", "false"]).strength(5.0, 6.0).no_occlusion().requires_correct_tool_for_drops(),
    Block::of("minecraft:waxed_weathered_copper_bars", BlockClass::IronBarsBlock).properties(&[&EAST, &NORTH, &SOUTH, &WATERLOGGED, &WEST], &["false", "false", "false", "false", "false"]).strength(5.0, 6.0).no_occlusion().requires_correct_tool_for_drops(),
    Block::of("minecraft:waxed_oxidized_copper_bars", BlockClass::IronBarsBlock).properties(&[&EAST, &NORTH, &SOUTH, &WATERLOGGED, &WEST], &["false", "false", "false", "false", "false"]).strength(5.0, 6.0).no_occlusion().requires_correct_tool_for_drops(),
    Block::of("minecraft:iron_chain", BlockClass::ChainBlock).properties(&[&AXIS, &WATERLOGGED], &["y", "false"]).strength(5.0, 6.0).no_occlusion().requires_correct_tool_for_drops().force_solid_on(),
    Block::of("minecraft:copper_chain", BlockClass::WeatheringCopperChainBlock).properties(&[&AXIS, &WATERLOGGED], &["y", "false"]).strength(5.0, 6.0).no_occlusion().requires_correct_tool_for_drops().force_solid_on(),
    Block::of("minecraft:exposed_copper_chain", BlockClass::WeatheringCopperChainBlock).properties(&[&AXIS, &WATERLOGGED], &["y", "false"]).strength(5.0, 6.0).no_occlusion().requires_correct_tool_for_drops().force_solid_on(),
    Block::of("minecraft:weathered_copper_chain", BlockClass::WeatheringCopperChainBlock).properties(&[&AXIS, &WATERLOGGED], &["y", "false"]).strength(5.0, 6.0).no_occlusion().requires_correct_tool_for_drops().force_solid_on(),
    Block::of("minecraft:oxidized_copper_chain", BlockClass::WeatheringCopperChainBlock).properties(&[&AXIS, &WATERLOGGED], &["y", "false"]).strength(5.0, 6.0).no_occlusion().requires_correct_tool_for_drops().force_solid_on(),
    Block::of("minecraft:waxed_copper_chain", BlockClass::ChainBlock).properties(&[&AXIS, &WATERLOGGED], &["y", "false"]).strength(5.0, 6.0).no_occlusion().requires_correct_tool_for_drops().force_solid_on(),
    Block::of("minecraft:waxed_exposed_copper_chain", BlockClass::ChainBlock).properties(&[&AXIS, &WATERLOGGED], &["y", "false"]).strength(5.0, 6.0).no_occlusion().requires_correct_tool_for_drops().force_solid_on(),
    Block::of("minecraft:waxed_weathered_copper_chain", BlockClass::ChainBlock).properties(&[&AXIS, &WATERLOGGED], &["y", "false"]).strength(5.0, 6.0).no_occlusion().requires_correct_tool_for_drops().force_solid_on(),
    Block::of("minecraft:waxed_oxidized_copper_chain", BlockClass::ChainBlock).properties(&[&AXIS, &WATERLOGGED], &["y", "false"]).strength(5.0, 6.0).no_occlusion().requires_correct_tool_for_drops().force_solid_on(),
    Block::of("minecraft:glass_pane", BlockClass::IronBarsBlock).properties(&[&EAST, &NORTH, &SOUTH, &WATERLOGGED, &WEST], &["false", "false", "false", "false", "false"]).strength(0.3, 0.3).no_occlusion().instrument("hat"),
    Block::of("minecraft:pumpkin", BlockClass::PumpkinBlock).strength(1.0, 1.0).push_reaction(PushReaction::Destroy).instrument("didgeridoo"),
    Block::of("minecraft:melon", BlockClass::Block).strength(1.0, 1.0).push_reaction(PushReaction::Destroy),
    Block::of("minecraft:attached_pumpkin_stem", BlockClass::AttachedStemBlock).properties(&[&HORIZONTAL_FACING], &["north"]).no_collision().push_reaction(PushReaction::Destroy).item("minecraft:air"),
    Block::of("minecraft:attached_melon_stem", BlockClass::AttachedStemBlock).properties(&[&HORIZONTAL_FACING], &["north"]).no_collision().push_reaction(PushReaction::Destroy).item("minecraft:air"),
    Block::of("minecraft:pumpkin_stem", BlockClass::StemBlock).properties(&[&AGE_7], &["0"]).no_collision().random_ticks().push_reaction(PushReaction::Destroy).item("minecraft:pumpkin_seeds"),
    Block::of("minecraft:melon_stem", BlockClass::StemBlock).properties(&[&AGE_7], &["0"]).no_collision().random_ticks().push_reaction(PushReaction::Destroy).item("minecraft:melon_seeds"),
    Block::of("minecraft:vine", BlockClass::VineBlock).properties(&[&EAST, &NORTH, &SOUTH, &UP, &WEST], &["false", "false", "false", "false", "false"]).strength(0.2, 0.2).no_collision().random_ticks().ignited_by_lava().replaceable().push_reaction(PushReaction::Destroy),
    Block::of("minecraft:glow_lichen", BlockClass::GlowLichenBlock).properties(&[&DOWN, &EAST, &NORTH, &SOUTH, &UP, &WATERLOGGED, &WEST], &["false", "false", "false", "false", "false", "false", "false"]).strength(0.2, 0.2).no_collision().ignited_by_lava().replaceable().push_reaction(PushReaction::Destroy).light_level(Emission::States(&[7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 7, 0, 7, 0])),
    Block::of("minecraft:resin_clump", BlockClass::MultifaceBlock).properties(&[&DOWN, &EAST, &NORTH, &SOUTH, &UP, &WATERLOGGED, &WEST], &["false", "false", "false", "false", "false", "false", "false"]).no_collision().ignited_by_lava().replaceable().push_reaction(PushReaction::Destroy),
    Block::of("minecraft:oak_fence_gate", BlockClass::FenceGateBlock).properties(&[&HORIZONTAL_FACING, &IN_WALL, &OPEN, &POWERED], &["north", "false", "false", "false"]).strength(2.0, 3.0).ignited_by_lava().force_solid_on().instrument("bass"),
    Block::of("minecraft:brick_stairs", BlockClass::StairBlock).properties(&[&HORIZONTAL_FACING, &HALF, &STAIRS_SHAPE, &WATERLOGGED], &["north", "bottom", "straight", "false"]).strength(2.0, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:stone_brick_stairs", BlockClass::StairBlock).properties(&[&HORIZONTAL_FACING, &HALF, &STAIRS_SHAPE, &WATERLOGGED], &["north", "bottom", "straight", "false"]).strength(1.5, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:mud_brick_stairs", BlockClass::StairBlock).properties(&[&HORIZONTAL_FACING, &HALF, &STAIRS_SHAPE, &WATERLOGGED], &["north", "bottom", "straight", "false"]).strength(1.5, 3.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:mycelium", BlockClass::MyceliumBlock).properties(&[&SNOWY], &["false"]).strength(0.6, 0.6).random_ticks(),
    Block::of("minecraft:lily_pad", BlockClass::LilyPadBlock).no_occlusion().push_reaction(PushReaction::Destroy),
    Block::of("minecraft:resin_block", BlockClass::Block).instrument("basedrum"),
    Block::of("minecraft:resin_bricks", BlockClass::Block).strength(1.5, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:resin_brick_stairs", BlockClass::StairBlock).properties(&[&HORIZONTAL_FACING, &HALF, &STAIRS_SHAPE, &WATERLOGGED], &["north", "bottom", "straight", "false"]).strength(1.5, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:resin_brick_slab", BlockClass::SlabBlock).properties(&[&SLAB_TYPE, &WATERLOGGED], &["bottom", "false"]).strength(1.5, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:resin_brick_wall", BlockClass::WallBlock).properties(&[&EAST_WALL, &NORTH_WALL, &SOUTH_WALL, &UP, &WATERLOGGED, &WEST_WALL], &["none", "none", "none", "true", "false", "none"]).strength(1.5, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:chiseled_resin_bricks", BlockClass::Block).strength(1.5, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:nether_bricks", BlockClass::Block).strength(2.0, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:nether_brick_fence", BlockClass::FenceBlock).properties(&[&EAST, &NORTH, &SOUTH, &WATERLOGGED, &WEST], &["false", "false", "false", "false", "false"]).strength(2.0, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:nether_brick_stairs", BlockClass::StairBlock).properties(&[&HORIZONTAL_FACING, &HALF, &STAIRS_SHAPE, &WATERLOGGED], &["north", "bottom", "straight", "false"]).strength(2.0, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:nether_wart", BlockClass::NetherWartBlock).properties(&[&AGE_3], &["0"]).no_collision().random_ticks().push_reaction(PushReaction::Destroy),
    Block::of("minecraft:enchanting_table", BlockClass::EnchantingTableBlock).strength(5.0, 1200.0).requires_correct_tool_for_drops().instrument("basedrum").light_level(Emission::Constant(7)),
    Block::of("minecraft:brewing_stand", BlockClass::BrewingStandBlock).properties(&[&HAS_BOTTLE_0, &HAS_BOTTLE_1, &HAS_BOTTLE_2], &["false", "false", "false"]).strength(0.5, 0.5).no_occlusion().light_level(Emission::Constant(1)),
    Block::of("minecraft:cauldron", BlockClass::CauldronBlock).strength(2.0, 2.0).no_occlusion().requires_correct_tool_for_drops(),
    Block::of("minecraft:water_cauldron", BlockClass::LayeredCauldronBlock).properties(&[&LEVEL_CAULDRON], &["1"]).strength(2.0, 2.0).no_occlusion().requires_correct_tool_for_drops().item("minecraft:cauldron"),
    Block::of("minecraft:lava_cauldron", BlockClass::LavaCauldronBlock).strength(2.0, 2.0).no_occlusion().requires_correct_tool_for_drops().item("minecraft:cauldron").light_level(Emission::Constant(15)),
    Block::of("minecraft:powder_snow_cauldron", BlockClass::LayeredCauldronBlock).properties(&[&LEVEL_CAULDRON], &["1"]).strength(2.0, 2.0).no_occlusion().requires_correct_tool_for_drops().item("minecraft:cauldron"),
    Block::of("minecraft:end_portal", BlockClass::EndPortalBlock).strength(-1.0, 3600000.0).no_collision().push_reaction(PushReaction::Block).item("minecraft:air").light_level(Emission::Constant(15)),
    Block::of("minecraft:end_portal_frame", BlockClass::EndPortalFrameBlock).properties(&[&EYE, &HORIZONTAL_FACING], &["false", "north"]).strength(-1.0, 3600000.0).instrument("basedrum").light_level(Emission::Constant(1)),
    Block::of("minecraft:end_stone", BlockClass::Block).strength(3.0, 9.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:dragon_egg", BlockClass::DragonEggBlock).strength(3.0, 9.0).no_occlusion().push_reaction(PushReaction::Destroy).light_level(Emission::Constant(1)),
    Block::of("minecraft:redstone_lamp", BlockClass::RedstoneLampBlock).properties(&[&LIT], &["false"]).strength(0.3, 0.3).light_level(Emission::States(&[15, 0])),
    Block::of("minecraft:cocoa", BlockClass::CocoaBlock).properties(&[&AGE_2, &HORIZONTAL_FACING], &["0", "north"]).strength(0.2, 3.0).no_occlusion().random_ticks().push_reaction(PushReaction::Destroy).item("minecraft:cocoa_beans"),
    Block::of("minecraft:sandstone_stairs", BlockClass::StairBlock).properties(&[&HORIZONTAL_FACING, &HALF, &STAIRS_SHAPE, &WATERLOGGED], &["north", "bottom", "straight", "false"]).strength(0.8, 0.8).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:emerald_ore", BlockClass::DropExperienceBlock).strength(3.0, 3.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:deepslate_emerald_ore", BlockClass::DropExperienceBlock).strength(4.5, 3.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:ender_chest", BlockClass::EnderChestBlock).properties(&[&HORIZONTAL_FACING, &WATERLOGGED], &["north", "false"]).strength(22.5, 600.0).instrument("basedrum").light_level(Emission::Constant(7)),
    Block::of("minecraft:tripwire_hook", BlockClass::TripWireHookBlock).properties(&[&ATTACHED, &HORIZONTAL_FACING, &POWERED], &["false", "north", "false"]).no_collision().push_reaction(PushReaction::Destroy),
    Block::of("minecraft:tripwire", BlockClass::TripWireBlock).properties(&[&ATTACHED, &DISARMED, &EAST, &NORTH, &POWERED, &SOUTH, &WEST], &["false", "false", "false", "false", "false", "false", "false"]).no_collision().push_reaction(PushReaction::Destroy).item("minecraft:string"),
    Block::of("minecraft:emerald_block", BlockClass::Block).strength(5.0, 6.0).requires_correct_tool_for_drops().instrument("bit"),
    Block::of("minecraft:spruce_stairs", BlockClass::StairBlock).properties(&[&HORIZONTAL_FACING, &HALF, &STAIRS_SHAPE, &WATERLOGGED], &["north", "bottom", "straight", "false"]).strength(2.0, 3.0).ignited_by_lava().instrument("bass"),
    Block::of("minecraft:birch_stairs", BlockClass::StairBlock).properties(&[&HORIZONTAL_FACING, &HALF, &STAIRS_SHAPE, &WATERLOGGED], &["north", "bottom", "straight", "false"]).strength(2.0, 3.0).ignited_by_lava().instrument("bass"),
    Block::of("minecraft:jungle_stairs", BlockClass::StairBlock).properties(&[&HORIZONTAL_FACING, &HALF, &STAIRS_SHAPE, &WATERLOGGED], &["north", "bottom", "straight", "false"]).strength(2.0, 3.0).ignited_by_lava().instrument("bass"),
    Block::of("minecraft:command_block", BlockClass::CommandBlock).properties(&[&CONDITIONAL, &FACING], &["false", "north"]).strength(-1.0, 3600000.0).requires_correct_tool_for_drops(),
    Block::of("minecraft:beacon", BlockClass::BeaconBlock).strength(3.0, 3.0).no_occlusion().instrument("hat").light_level(Emission::Constant(15)).is_redstone_conductor(Rule::Never),
    Block::of("minecraft:cobblestone_wall", BlockClass::WallBlock).properties(&[&EAST_WALL, &NORTH_WALL, &SOUTH_WALL, &UP, &WATERLOGGED, &WEST_WALL], &["none", "none", "none", "true", "false", "none"]).strength(2.0, 6.0).requires_correct_tool_for_drops().force_solid_on().instrument("basedrum"),
    Block::of("minecraft:mossy_cobblestone_wall", BlockClass::WallBlock).properties(&[&EAST_WALL, &NORTH_WALL, &SOUTH_WALL, &UP, &WATERLOGGED, &WEST_WALL], &["none", "none", "none", "true", "false", "none"]).strength(2.0, 6.0).requires_correct_tool_for_drops().force_solid_on().instrument("basedrum"),
    Block::of("minecraft:flower_pot", BlockClass::FlowerPotBlock).no_occlusion().push_reaction(PushReaction::Destroy),
    Block::of("minecraft:potted_torchflower", BlockClass::FlowerPotBlock).no_occlusion().push_reaction(PushReaction::Destroy).item("minecraft:air"),
    Block::of("minecraft:potted_oak_sapling", BlockClass::FlowerPotBlock).no_occlusion().push_reaction(PushReaction::Destroy).item("minecraft:air"),
    Block::of("minecraft:potted_spruce_sapling", BlockClass::FlowerPotBlock).no_occlusion().push_reaction(PushReaction::Destroy).item("minecraft:air"),
    Block::of("minecraft:potted_birch_sapling", BlockClass::FlowerPotBlock).no_occlusion().push_reaction(PushReaction::Destroy).item("minecraft:air"),
    Block::of("minecraft:potted_jungle_sapling", BlockClass::FlowerPotBlock).no_occlusion().push_reaction(PushReaction::Destroy).item("minecraft:air"),
    Block::of("minecraft:potted_acacia_sapling", BlockClass::FlowerPotBlock).no_occlusion().push_reaction(PushReaction::Destroy).item("minecraft:air"),
    Block::of("minecraft:potted_cherry_sapling", BlockClass::FlowerPotBlock).no_occlusion().push_reaction(PushReaction::Destroy).item("minecraft:air"),
    Block::of("minecraft:potted_dark_oak_sapling", BlockClass::FlowerPotBlock).no_occlusion().push_reaction(PushReaction::Destroy).item("minecraft:air"),
    Block::of("minecraft:potted_pale_oak_sapling", BlockClass::FlowerPotBlock).no_occlusion().push_reaction(PushReaction::Destroy).item("minecraft:air"),
    Block::of("minecraft:potted_mangrove_propagule", BlockClass::FlowerPotBlock).no_occlusion().push_reaction(PushReaction::Destroy).item("minecraft:air"),
    Block::of("minecraft:potted_fern", BlockClass::FlowerPotBlock).no_occlusion().push_reaction(PushReaction::Destroy).item("minecraft:air"),
    Block::of("minecraft:potted_dandelion", BlockClass::FlowerPotBlock).no_occlusion().push_reaction(PushReaction::Destroy).item("minecraft:air"),
    Block::of("minecraft:potted_golden_dandelion", BlockClass::FlowerPotBlock).no_occlusion().push_reaction(PushReaction::Destroy).item("minecraft:air"),
    Block::of("minecraft:potted_poppy", BlockClass::FlowerPotBlock).no_occlusion().push_reaction(PushReaction::Destroy).item("minecraft:air"),
    Block::of("minecraft:potted_blue_orchid", BlockClass::FlowerPotBlock).no_occlusion().push_reaction(PushReaction::Destroy).item("minecraft:air"),
    Block::of("minecraft:potted_allium", BlockClass::FlowerPotBlock).no_occlusion().push_reaction(PushReaction::Destroy).item("minecraft:air"),
    Block::of("minecraft:potted_azure_bluet", BlockClass::FlowerPotBlock).no_occlusion().push_reaction(PushReaction::Destroy).item("minecraft:air"),
    Block::of("minecraft:potted_red_tulip", BlockClass::FlowerPotBlock).no_occlusion().push_reaction(PushReaction::Destroy).item("minecraft:air"),
    Block::of("minecraft:potted_orange_tulip", BlockClass::FlowerPotBlock).no_occlusion().push_reaction(PushReaction::Destroy).item("minecraft:air"),
    Block::of("minecraft:potted_white_tulip", BlockClass::FlowerPotBlock).no_occlusion().push_reaction(PushReaction::Destroy).item("minecraft:air"),
    Block::of("minecraft:potted_pink_tulip", BlockClass::FlowerPotBlock).no_occlusion().push_reaction(PushReaction::Destroy).item("minecraft:air"),
    Block::of("minecraft:potted_oxeye_daisy", BlockClass::FlowerPotBlock).no_occlusion().push_reaction(PushReaction::Destroy).item("minecraft:air"),
    Block::of("minecraft:potted_cornflower", BlockClass::FlowerPotBlock).no_occlusion().push_reaction(PushReaction::Destroy).item("minecraft:air"),
    Block::of("minecraft:potted_lily_of_the_valley", BlockClass::FlowerPotBlock).no_occlusion().push_reaction(PushReaction::Destroy).item("minecraft:air"),
    Block::of("minecraft:potted_wither_rose", BlockClass::FlowerPotBlock).no_occlusion().push_reaction(PushReaction::Destroy).item("minecraft:air"),
    Block::of("minecraft:potted_red_mushroom", BlockClass::FlowerPotBlock).no_occlusion().push_reaction(PushReaction::Destroy).item("minecraft:air"),
    Block::of("minecraft:potted_brown_mushroom", BlockClass::FlowerPotBlock).no_occlusion().push_reaction(PushReaction::Destroy).item("minecraft:air"),
    Block::of("minecraft:potted_dead_bush", BlockClass::FlowerPotBlock).no_occlusion().push_reaction(PushReaction::Destroy).item("minecraft:air"),
    Block::of("minecraft:potted_cactus", BlockClass::FlowerPotBlock).no_occlusion().push_reaction(PushReaction::Destroy).item("minecraft:air"),
    Block::of("minecraft:carrots", BlockClass::CarrotBlock).properties(&[&AGE_7], &["0"]).no_collision().random_ticks().push_reaction(PushReaction::Destroy).item("minecraft:carrot"),
    Block::of("minecraft:potatoes", BlockClass::PotatoBlock).properties(&[&AGE_7], &["0"]).no_collision().random_ticks().push_reaction(PushReaction::Destroy).item("minecraft:potato"),
    Block::of("minecraft:oak_button", BlockClass::ButtonBlock).properties(&[&ATTACH_FACE, &HORIZONTAL_FACING, &POWERED], &["wall", "north", "false"]).strength(0.5, 0.5).no_collision().push_reaction(PushReaction::Destroy),
    Block::of("minecraft:spruce_button", BlockClass::ButtonBlock).properties(&[&ATTACH_FACE, &HORIZONTAL_FACING, &POWERED], &["wall", "north", "false"]).strength(0.5, 0.5).no_collision().push_reaction(PushReaction::Destroy),
    Block::of("minecraft:birch_button", BlockClass::ButtonBlock).properties(&[&ATTACH_FACE, &HORIZONTAL_FACING, &POWERED], &["wall", "north", "false"]).strength(0.5, 0.5).no_collision().push_reaction(PushReaction::Destroy),
    Block::of("minecraft:jungle_button", BlockClass::ButtonBlock).properties(&[&ATTACH_FACE, &HORIZONTAL_FACING, &POWERED], &["wall", "north", "false"]).strength(0.5, 0.5).no_collision().push_reaction(PushReaction::Destroy),
    Block::of("minecraft:acacia_button", BlockClass::ButtonBlock).properties(&[&ATTACH_FACE, &HORIZONTAL_FACING, &POWERED], &["wall", "north", "false"]).strength(0.5, 0.5).no_collision().push_reaction(PushReaction::Destroy),
    Block::of("minecraft:cherry_button", BlockClass::ButtonBlock).properties(&[&ATTACH_FACE, &HORIZONTAL_FACING, &POWERED], &["wall", "north", "false"]).strength(0.5, 0.5).no_collision().push_reaction(PushReaction::Destroy),
    Block::of("minecraft:dark_oak_button", BlockClass::ButtonBlock).properties(&[&ATTACH_FACE, &HORIZONTAL_FACING, &POWERED], &["wall", "north", "false"]).strength(0.5, 0.5).no_collision().push_reaction(PushReaction::Destroy),
    Block::of("minecraft:pale_oak_button", BlockClass::ButtonBlock).properties(&[&ATTACH_FACE, &HORIZONTAL_FACING, &POWERED], &["wall", "north", "false"]).strength(0.5, 0.5).no_collision().push_reaction(PushReaction::Destroy),
    Block::of("minecraft:mangrove_button", BlockClass::ButtonBlock).properties(&[&ATTACH_FACE, &HORIZONTAL_FACING, &POWERED], &["wall", "north", "false"]).strength(0.5, 0.5).no_collision().push_reaction(PushReaction::Destroy),
    Block::of("minecraft:bamboo_button", BlockClass::ButtonBlock).properties(&[&ATTACH_FACE, &HORIZONTAL_FACING, &POWERED], &["wall", "north", "false"]).strength(0.5, 0.5).no_collision().push_reaction(PushReaction::Destroy),
    Block::of("minecraft:skeleton_skull", BlockClass::SkullBlock).properties(&[&POWERED, &ROTATION_16], &["false", "0"]).strength(1.0, 1.0).no_occlusion().push_reaction(PushReaction::Destroy).instrument("skeleton"),
    Block::of("minecraft:skeleton_wall_skull", BlockClass::WallSkullBlock).properties(&[&HORIZONTAL_FACING, &POWERED], &["north", "false"]).strength(1.0, 1.0).push_reaction(PushReaction::Destroy).item("minecraft:skeleton_skull"),
    Block::of("minecraft:wither_skeleton_skull", BlockClass::WitherSkullBlock).properties(&[&POWERED, &ROTATION_16], &["false", "0"]).strength(1.0, 1.0).no_occlusion().push_reaction(PushReaction::Destroy).instrument("wither_skeleton"),
    Block::of("minecraft:wither_skeleton_wall_skull", BlockClass::WitherWallSkullBlock).properties(&[&HORIZONTAL_FACING, &POWERED], &["north", "false"]).strength(1.0, 1.0).push_reaction(PushReaction::Destroy).item("minecraft:wither_skeleton_skull"),
    Block::of("minecraft:zombie_head", BlockClass::SkullBlock).properties(&[&POWERED, &ROTATION_16], &["false", "0"]).strength(1.0, 1.0).no_occlusion().push_reaction(PushReaction::Destroy).instrument("zombie"),
    Block::of("minecraft:zombie_wall_head", BlockClass::WallSkullBlock).properties(&[&HORIZONTAL_FACING, &POWERED], &["north", "false"]).strength(1.0, 1.0).push_reaction(PushReaction::Destroy).item("minecraft:zombie_head"),
    Block::of("minecraft:player_head", BlockClass::PlayerHeadBlock).properties(&[&POWERED, &ROTATION_16], &["false", "0"]).strength(1.0, 1.0).no_occlusion().push_reaction(PushReaction::Destroy).instrument("custom_head"),
    Block::of("minecraft:player_wall_head", BlockClass::PlayerWallHeadBlock).properties(&[&HORIZONTAL_FACING, &POWERED], &["north", "false"]).strength(1.0, 1.0).push_reaction(PushReaction::Destroy).item("minecraft:player_head"),
    Block::of("minecraft:creeper_head", BlockClass::SkullBlock).properties(&[&POWERED, &ROTATION_16], &["false", "0"]).strength(1.0, 1.0).no_occlusion().push_reaction(PushReaction::Destroy).instrument("creeper"),
    Block::of("minecraft:creeper_wall_head", BlockClass::WallSkullBlock).properties(&[&HORIZONTAL_FACING, &POWERED], &["north", "false"]).strength(1.0, 1.0).push_reaction(PushReaction::Destroy).item("minecraft:creeper_head"),
    Block::of("minecraft:dragon_head", BlockClass::SkullBlock).properties(&[&POWERED, &ROTATION_16], &["false", "0"]).strength(1.0, 1.0).no_occlusion().push_reaction(PushReaction::Destroy).instrument("dragon"),
    Block::of("minecraft:dragon_wall_head", BlockClass::WallSkullBlock).properties(&[&HORIZONTAL_FACING, &POWERED], &["north", "false"]).strength(1.0, 1.0).push_reaction(PushReaction::Destroy).item("minecraft:dragon_head"),
    Block::of("minecraft:piglin_head", BlockClass::SkullBlock).properties(&[&POWERED, &ROTATION_16], &["false", "0"]).strength(1.0, 1.0).no_occlusion().push_reaction(PushReaction::Destroy).instrument("piglin"),
    Block::of("minecraft:piglin_wall_head", BlockClass::PiglinWallSkullBlock).properties(&[&HORIZONTAL_FACING, &POWERED], &["north", "false"]).strength(1.0, 1.0).push_reaction(PushReaction::Destroy).item("minecraft:piglin_head"),
    Block::of("minecraft:anvil", BlockClass::AnvilBlock).properties(&[&HORIZONTAL_FACING], &["north"]).strength(5.0, 1200.0).requires_correct_tool_for_drops().push_reaction(PushReaction::Block),
    Block::of("minecraft:chipped_anvil", BlockClass::AnvilBlock).properties(&[&HORIZONTAL_FACING], &["north"]).strength(5.0, 1200.0).requires_correct_tool_for_drops().push_reaction(PushReaction::Block),
    Block::of("minecraft:damaged_anvil", BlockClass::AnvilBlock).properties(&[&HORIZONTAL_FACING], &["north"]).strength(5.0, 1200.0).requires_correct_tool_for_drops().push_reaction(PushReaction::Block),
    Block::of("minecraft:trapped_chest", BlockClass::TrappedChestBlock).properties(&[&HORIZONTAL_FACING, &CHEST_TYPE, &WATERLOGGED], &["north", "single", "false"]).strength(2.5, 2.5).ignited_by_lava().instrument("bass"),
    Block::of("minecraft:light_weighted_pressure_plate", BlockClass::WeightedPressurePlateBlock).properties(&[&POWER], &["0"]).strength(0.5, 0.5).no_collision().force_solid_on().push_reaction(PushReaction::Destroy),
    Block::of("minecraft:heavy_weighted_pressure_plate", BlockClass::WeightedPressurePlateBlock).properties(&[&POWER], &["0"]).strength(0.5, 0.5).no_collision().force_solid_on().push_reaction(PushReaction::Destroy),
    Block::of("minecraft:comparator", BlockClass::ComparatorBlock).properties(&[&HORIZONTAL_FACING, &MODE_COMPARATOR, &POWERED], &["north", "compare", "false"]).push_reaction(PushReaction::Destroy),
    Block::of("minecraft:daylight_detector", BlockClass::DaylightDetectorBlock).properties(&[&INVERTED, &POWER], &["false", "0"]).strength(0.2, 0.2).ignited_by_lava().instrument("bass"),
    Block::of("minecraft:redstone_block", BlockClass::PoweredBlock).strength(5.0, 6.0).requires_correct_tool_for_drops().is_redstone_conductor(Rule::Never),
    Block::of("minecraft:nether_quartz_ore", BlockClass::DropExperienceBlock).strength(3.0, 3.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:hopper", BlockClass::HopperBlock).properties(&[&ENABLED, &FACING_HOPPER], &["true", "down"]).strength(3.0, 4.8).no_occlusion().requires_correct_tool_for_drops(),
    Block::of("minecraft:quartz_block", BlockClass::Block).strength(0.8, 0.8).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:chiseled_quartz_block", BlockClass::Block).strength(0.8, 0.8).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:quartz_pillar", BlockClass::RotatedPillarBlock).properties(&[&AXIS], &["y"]).strength(0.8, 0.8).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:quartz_stairs", BlockClass::StairBlock).properties(&[&HORIZONTAL_FACING, &HALF, &STAIRS_SHAPE, &WATERLOGGED], &["north", "bottom", "straight", "false"]).strength(0.8, 0.8).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:activator_rail", BlockClass::PoweredRailBlock).properties(&[&POWERED, &RAIL_SHAPE_STRAIGHT, &WATERLOGGED], &["false", "north_south", "false"]).strength(0.7, 0.7).no_collision(),
    Block::of("minecraft:dropper", BlockClass::DropperBlock).properties(&[&FACING, &TRIGGERED], &["north", "false"]).strength(3.5, 3.5).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:white_terracotta", BlockClass::Block).strength(1.25, 4.2).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:orange_terracotta", BlockClass::Block).strength(1.25, 4.2).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:magenta_terracotta", BlockClass::Block).strength(1.25, 4.2).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:light_blue_terracotta", BlockClass::Block).strength(1.25, 4.2).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:yellow_terracotta", BlockClass::Block).strength(1.25, 4.2).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:lime_terracotta", BlockClass::Block).strength(1.25, 4.2).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:pink_terracotta", BlockClass::Block).strength(1.25, 4.2).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:gray_terracotta", BlockClass::Block).strength(1.25, 4.2).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:light_gray_terracotta", BlockClass::Block).strength(1.25, 4.2).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:cyan_terracotta", BlockClass::Block).strength(1.25, 4.2).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:purple_terracotta", BlockClass::Block).strength(1.25, 4.2).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:blue_terracotta", BlockClass::Block).strength(1.25, 4.2).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:brown_terracotta", BlockClass::Block).strength(1.25, 4.2).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:green_terracotta", BlockClass::Block).strength(1.25, 4.2).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:red_terracotta", BlockClass::Block).strength(1.25, 4.2).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:black_terracotta", BlockClass::Block).strength(1.25, 4.2).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:white_stained_glass_pane", BlockClass::StainedGlassPaneBlock).properties(&[&EAST, &NORTH, &SOUTH, &WATERLOGGED, &WEST], &["false", "false", "false", "false", "false"]).strength(0.3, 0.3).no_occlusion().instrument("hat"),
    Block::of("minecraft:orange_stained_glass_pane", BlockClass::StainedGlassPaneBlock).properties(&[&EAST, &NORTH, &SOUTH, &WATERLOGGED, &WEST], &["false", "false", "false", "false", "false"]).strength(0.3, 0.3).no_occlusion().instrument("hat"),
    Block::of("minecraft:magenta_stained_glass_pane", BlockClass::StainedGlassPaneBlock).properties(&[&EAST, &NORTH, &SOUTH, &WATERLOGGED, &WEST], &["false", "false", "false", "false", "false"]).strength(0.3, 0.3).no_occlusion().instrument("hat"),
    Block::of("minecraft:light_blue_stained_glass_pane", BlockClass::StainedGlassPaneBlock).properties(&[&EAST, &NORTH, &SOUTH, &WATERLOGGED, &WEST], &["false", "false", "false", "false", "false"]).strength(0.3, 0.3).no_occlusion().instrument("hat"),
    Block::of("minecraft:yellow_stained_glass_pane", BlockClass::StainedGlassPaneBlock).properties(&[&EAST, &NORTH, &SOUTH, &WATERLOGGED, &WEST], &["false", "false", "false", "false", "false"]).strength(0.3, 0.3).no_occlusion().instrument("hat"),
    Block::of("minecraft:lime_stained_glass_pane", BlockClass::StainedGlassPaneBlock).properties(&[&EAST, &NORTH, &SOUTH, &WATERLOGGED, &WEST], &["false", "false", "false", "false", "false"]).strength(0.3, 0.3).no_occlusion().instrument("hat"),
    Block::of("minecraft:pink_stained_glass_pane", BlockClass::StainedGlassPaneBlock).properties(&[&EAST, &NORTH, &SOUTH, &WATERLOGGED, &WEST], &["false", "false", "false", "false", "false"]).strength(0.3, 0.3).no_occlusion().instrument("hat"),
    Block::of("minecraft:gray_stained_glass_pane", BlockClass::StainedGlassPaneBlock).properties(&[&EAST, &NORTH, &SOUTH, &WATERLOGGED, &WEST], &["false", "false", "false", "false", "false"]).strength(0.3, 0.3).no_occlusion().instrument("hat"),
    Block::of("minecraft:light_gray_stained_glass_pane", BlockClass::StainedGlassPaneBlock).properties(&[&EAST, &NORTH, &SOUTH, &WATERLOGGED, &WEST], &["false", "false", "false", "false", "false"]).strength(0.3, 0.3).no_occlusion().instrument("hat"),
    Block::of("minecraft:cyan_stained_glass_pane", BlockClass::StainedGlassPaneBlock).properties(&[&EAST, &NORTH, &SOUTH, &WATERLOGGED, &WEST], &["false", "false", "false", "false", "false"]).strength(0.3, 0.3).no_occlusion().instrument("hat"),
    Block::of("minecraft:purple_stained_glass_pane", BlockClass::StainedGlassPaneBlock).properties(&[&EAST, &NORTH, &SOUTH, &WATERLOGGED, &WEST], &["false", "false", "false", "false", "false"]).strength(0.3, 0.3).no_occlusion().instrument("hat"),
    Block::of("minecraft:blue_stained_glass_pane", BlockClass::StainedGlassPaneBlock).properties(&[&EAST, &NORTH, &SOUTH, &WATERLOGGED, &WEST], &["false", "false", "false", "false", "false"]).strength(0.3, 0.3).no_occlusion().instrument("hat"),
    Block::of("minecraft:brown_stained_glass_pane", BlockClass::StainedGlassPaneBlock).properties(&[&EAST, &NORTH, &SOUTH, &WATERLOGGED, &WEST], &["false", "false", "false", "false", "false"]).strength(0.3, 0.3).no_occlusion().instrument("hat"),
    Block::of("minecraft:green_stained_glass_pane", BlockClass::StainedGlassPaneBlock).properties(&[&EAST, &NORTH, &SOUTH, &WATERLOGGED, &WEST], &["false", "false", "false", "false", "false"]).strength(0.3, 0.3).no_occlusion().instrument("hat"),
    Block::of("minecraft:red_stained_glass_pane", BlockClass::StainedGlassPaneBlock).properties(&[&EAST, &NORTH, &SOUTH, &WATERLOGGED, &WEST], &["false", "false", "false", "false", "false"]).strength(0.3, 0.3).no_occlusion().instrument("hat"),
    Block::of("minecraft:black_stained_glass_pane", BlockClass::StainedGlassPaneBlock).properties(&[&EAST, &NORTH, &SOUTH, &WATERLOGGED, &WEST], &["false", "false", "false", "false", "false"]).strength(0.3, 0.3).no_occlusion().instrument("hat"),
    Block::of("minecraft:acacia_stairs", BlockClass::StairBlock).properties(&[&HORIZONTAL_FACING, &HALF, &STAIRS_SHAPE, &WATERLOGGED], &["north", "bottom", "straight", "false"]).strength(2.0, 3.0).ignited_by_lava().instrument("bass"),
    Block::of("minecraft:cherry_stairs", BlockClass::StairBlock).properties(&[&HORIZONTAL_FACING, &HALF, &STAIRS_SHAPE, &WATERLOGGED], &["north", "bottom", "straight", "false"]).strength(2.0, 3.0).ignited_by_lava().instrument("bass"),
    Block::of("minecraft:dark_oak_stairs", BlockClass::StairBlock).properties(&[&HORIZONTAL_FACING, &HALF, &STAIRS_SHAPE, &WATERLOGGED], &["north", "bottom", "straight", "false"]).strength(2.0, 3.0).ignited_by_lava().instrument("bass"),
    Block::of("minecraft:pale_oak_stairs", BlockClass::StairBlock).properties(&[&HORIZONTAL_FACING, &HALF, &STAIRS_SHAPE, &WATERLOGGED], &["north", "bottom", "straight", "false"]).strength(2.0, 3.0).ignited_by_lava().instrument("bass"),
    Block::of("minecraft:mangrove_stairs", BlockClass::StairBlock).properties(&[&HORIZONTAL_FACING, &HALF, &STAIRS_SHAPE, &WATERLOGGED], &["north", "bottom", "straight", "false"]).strength(2.0, 3.0).ignited_by_lava().instrument("bass"),
    Block::of("minecraft:bamboo_stairs", BlockClass::StairBlock).properties(&[&HORIZONTAL_FACING, &HALF, &STAIRS_SHAPE, &WATERLOGGED], &["north", "bottom", "straight", "false"]).strength(2.0, 3.0).ignited_by_lava().instrument("bass"),
    Block::of("minecraft:bamboo_mosaic_stairs", BlockClass::StairBlock).properties(&[&HORIZONTAL_FACING, &HALF, &STAIRS_SHAPE, &WATERLOGGED], &["north", "bottom", "straight", "false"]).strength(2.0, 3.0).ignited_by_lava().instrument("bass"),
    Block::of("minecraft:slime_block", BlockClass::SlimeBlock).friction(0.8).bounce_restitution(1.0).no_occlusion(),
    Block::of("minecraft:barrier", BlockClass::BarrierBlock).properties(&[&WATERLOGGED], &["false"]).strength(-1.0, 3600000.8).no_occlusion().no_terrain_particles().push_reaction(PushReaction::Block),
    Block::of("minecraft:light", BlockClass::LightBlock).properties(&[&LEVEL, &WATERLOGGED], &["15", "false"]).strength(-1.0, 3600000.8).no_occlusion().replaceable().light_level(Emission::States(&[0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12, 13, 13, 14, 14, 15, 15])),
    Block::of("minecraft:iron_trapdoor", BlockClass::TrapDoorBlock).properties(&[&HORIZONTAL_FACING, &HALF, &OPEN, &POWERED, &WATERLOGGED], &["north", "bottom", "false", "false", "false"]).strength(5.0, 5.0).no_occlusion().requires_correct_tool_for_drops(),
    Block::of("minecraft:prismarine", BlockClass::Block).strength(1.5, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:prismarine_bricks", BlockClass::Block).strength(1.5, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:dark_prismarine", BlockClass::Block).strength(1.5, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:prismarine_stairs", BlockClass::StairBlock).properties(&[&HORIZONTAL_FACING, &HALF, &STAIRS_SHAPE, &WATERLOGGED], &["north", "bottom", "straight", "false"]).strength(1.5, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:prismarine_brick_stairs", BlockClass::StairBlock).properties(&[&HORIZONTAL_FACING, &HALF, &STAIRS_SHAPE, &WATERLOGGED], &["north", "bottom", "straight", "false"]).strength(1.5, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:dark_prismarine_stairs", BlockClass::StairBlock).properties(&[&HORIZONTAL_FACING, &HALF, &STAIRS_SHAPE, &WATERLOGGED], &["north", "bottom", "straight", "false"]).strength(1.5, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:prismarine_slab", BlockClass::SlabBlock).properties(&[&SLAB_TYPE, &WATERLOGGED], &["bottom", "false"]).strength(1.5, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:prismarine_brick_slab", BlockClass::SlabBlock).properties(&[&SLAB_TYPE, &WATERLOGGED], &["bottom", "false"]).strength(1.5, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:dark_prismarine_slab", BlockClass::SlabBlock).properties(&[&SLAB_TYPE, &WATERLOGGED], &["bottom", "false"]).strength(1.5, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:sea_lantern", BlockClass::Block).strength(0.3, 0.3).instrument("hat").light_level(Emission::Constant(15)).is_redstone_conductor(Rule::Never),
    Block::of("minecraft:hay_block", BlockClass::HayBlock).properties(&[&AXIS], &["y"]).strength(0.5, 0.5).instrument("banjo"),
    Block::of("minecraft:white_carpet", BlockClass::WoolCarpetBlock).strength(0.1, 0.1).ignited_by_lava(),
    Block::of("minecraft:orange_carpet", BlockClass::WoolCarpetBlock).strength(0.1, 0.1).ignited_by_lava(),
    Block::of("minecraft:magenta_carpet", BlockClass::WoolCarpetBlock).strength(0.1, 0.1).ignited_by_lava(),
    Block::of("minecraft:light_blue_carpet", BlockClass::WoolCarpetBlock).strength(0.1, 0.1).ignited_by_lava(),
    Block::of("minecraft:yellow_carpet", BlockClass::WoolCarpetBlock).strength(0.1, 0.1).ignited_by_lava(),
    Block::of("minecraft:lime_carpet", BlockClass::WoolCarpetBlock).strength(0.1, 0.1).ignited_by_lava(),
    Block::of("minecraft:pink_carpet", BlockClass::WoolCarpetBlock).strength(0.1, 0.1).ignited_by_lava(),
    Block::of("minecraft:gray_carpet", BlockClass::WoolCarpetBlock).strength(0.1, 0.1).ignited_by_lava(),
    Block::of("minecraft:light_gray_carpet", BlockClass::WoolCarpetBlock).strength(0.1, 0.1).ignited_by_lava(),
    Block::of("minecraft:cyan_carpet", BlockClass::WoolCarpetBlock).strength(0.1, 0.1).ignited_by_lava(),
    Block::of("minecraft:purple_carpet", BlockClass::WoolCarpetBlock).strength(0.1, 0.1).ignited_by_lava(),
    Block::of("minecraft:blue_carpet", BlockClass::WoolCarpetBlock).strength(0.1, 0.1).ignited_by_lava(),
    Block::of("minecraft:brown_carpet", BlockClass::WoolCarpetBlock).strength(0.1, 0.1).ignited_by_lava(),
    Block::of("minecraft:green_carpet", BlockClass::WoolCarpetBlock).strength(0.1, 0.1).ignited_by_lava(),
    Block::of("minecraft:red_carpet", BlockClass::WoolCarpetBlock).strength(0.1, 0.1).ignited_by_lava(),
    Block::of("minecraft:black_carpet", BlockClass::WoolCarpetBlock).strength(0.1, 0.1).ignited_by_lava(),
    Block::of("minecraft:terracotta", BlockClass::Block).strength(1.25, 4.2).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:coal_block", BlockClass::Block).strength(5.0, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:packed_ice", BlockClass::Block).strength(0.5, 0.5).friction(0.98).instrument("chime"),
    Block::of("minecraft:sunflower", BlockClass::TallFlowerBlock).properties(&[&DOUBLE_BLOCK_HALF], &["lower"]).no_collision().ignited_by_lava().offset_type(OffsetType::Xz).push_reaction(PushReaction::Destroy),
    Block::of("minecraft:lilac", BlockClass::TallFlowerBlock).properties(&[&DOUBLE_BLOCK_HALF], &["lower"]).no_collision().ignited_by_lava().offset_type(OffsetType::Xz).push_reaction(PushReaction::Destroy),
    Block::of("minecraft:rose_bush", BlockClass::TallFlowerBlock).properties(&[&DOUBLE_BLOCK_HALF], &["lower"]).no_collision().ignited_by_lava().offset_type(OffsetType::Xz).push_reaction(PushReaction::Destroy),
    Block::of("minecraft:peony", BlockClass::TallFlowerBlock).properties(&[&DOUBLE_BLOCK_HALF], &["lower"]).no_collision().ignited_by_lava().offset_type(OffsetType::Xz).push_reaction(PushReaction::Destroy),
    Block::of("minecraft:tall_grass", BlockClass::DoublePlantBlock).properties(&[&DOUBLE_BLOCK_HALF], &["lower"]).no_collision().ignited_by_lava().replaceable().offset_type(OffsetType::Xz).push_reaction(PushReaction::Destroy),
    Block::of("minecraft:large_fern", BlockClass::DoublePlantBlock).properties(&[&DOUBLE_BLOCK_HALF], &["lower"]).no_collision().ignited_by_lava().replaceable().offset_type(OffsetType::Xz).push_reaction(PushReaction::Destroy),
    Block::of("minecraft:white_banner", BlockClass::BannerBlock).properties(&[&ROTATION_16], &["8"]).strength(1.0, 1.0).no_collision().ignited_by_lava().force_solid_on().instrument("bass"),
    Block::of("minecraft:orange_banner", BlockClass::BannerBlock).properties(&[&ROTATION_16], &["8"]).strength(1.0, 1.0).no_collision().ignited_by_lava().force_solid_on().instrument("bass"),
    Block::of("minecraft:magenta_banner", BlockClass::BannerBlock).properties(&[&ROTATION_16], &["8"]).strength(1.0, 1.0).no_collision().ignited_by_lava().force_solid_on().instrument("bass"),
    Block::of("minecraft:light_blue_banner", BlockClass::BannerBlock).properties(&[&ROTATION_16], &["8"]).strength(1.0, 1.0).no_collision().ignited_by_lava().force_solid_on().instrument("bass"),
    Block::of("minecraft:yellow_banner", BlockClass::BannerBlock).properties(&[&ROTATION_16], &["8"]).strength(1.0, 1.0).no_collision().ignited_by_lava().force_solid_on().instrument("bass"),
    Block::of("minecraft:lime_banner", BlockClass::BannerBlock).properties(&[&ROTATION_16], &["8"]).strength(1.0, 1.0).no_collision().ignited_by_lava().force_solid_on().instrument("bass"),
    Block::of("minecraft:pink_banner", BlockClass::BannerBlock).properties(&[&ROTATION_16], &["8"]).strength(1.0, 1.0).no_collision().ignited_by_lava().force_solid_on().instrument("bass"),
    Block::of("minecraft:gray_banner", BlockClass::BannerBlock).properties(&[&ROTATION_16], &["8"]).strength(1.0, 1.0).no_collision().ignited_by_lava().force_solid_on().instrument("bass"),
    Block::of("minecraft:light_gray_banner", BlockClass::BannerBlock).properties(&[&ROTATION_16], &["8"]).strength(1.0, 1.0).no_collision().ignited_by_lava().force_solid_on().instrument("bass"),
    Block::of("minecraft:cyan_banner", BlockClass::BannerBlock).properties(&[&ROTATION_16], &["8"]).strength(1.0, 1.0).no_collision().ignited_by_lava().force_solid_on().instrument("bass"),
    Block::of("minecraft:purple_banner", BlockClass::BannerBlock).properties(&[&ROTATION_16], &["8"]).strength(1.0, 1.0).no_collision().ignited_by_lava().force_solid_on().instrument("bass"),
    Block::of("minecraft:blue_banner", BlockClass::BannerBlock).properties(&[&ROTATION_16], &["8"]).strength(1.0, 1.0).no_collision().ignited_by_lava().force_solid_on().instrument("bass"),
    Block::of("minecraft:brown_banner", BlockClass::BannerBlock).properties(&[&ROTATION_16], &["8"]).strength(1.0, 1.0).no_collision().ignited_by_lava().force_solid_on().instrument("bass"),
    Block::of("minecraft:green_banner", BlockClass::BannerBlock).properties(&[&ROTATION_16], &["8"]).strength(1.0, 1.0).no_collision().ignited_by_lava().force_solid_on().instrument("bass"),
    Block::of("minecraft:red_banner", BlockClass::BannerBlock).properties(&[&ROTATION_16], &["8"]).strength(1.0, 1.0).no_collision().ignited_by_lava().force_solid_on().instrument("bass"),
    Block::of("minecraft:black_banner", BlockClass::BannerBlock).properties(&[&ROTATION_16], &["8"]).strength(1.0, 1.0).no_collision().ignited_by_lava().force_solid_on().instrument("bass"),
    Block::of("minecraft:white_wall_banner", BlockClass::WallBannerBlock).properties(&[&HORIZONTAL_FACING], &["north"]).strength(1.0, 1.0).no_collision().ignited_by_lava().force_solid_on().instrument("bass").item("minecraft:white_banner"),
    Block::of("minecraft:orange_wall_banner", BlockClass::WallBannerBlock).properties(&[&HORIZONTAL_FACING], &["north"]).strength(1.0, 1.0).no_collision().ignited_by_lava().force_solid_on().instrument("bass").item("minecraft:orange_banner"),
    Block::of("minecraft:magenta_wall_banner", BlockClass::WallBannerBlock).properties(&[&HORIZONTAL_FACING], &["north"]).strength(1.0, 1.0).no_collision().ignited_by_lava().force_solid_on().instrument("bass").item("minecraft:magenta_banner"),
    Block::of("minecraft:light_blue_wall_banner", BlockClass::WallBannerBlock).properties(&[&HORIZONTAL_FACING], &["north"]).strength(1.0, 1.0).no_collision().ignited_by_lava().force_solid_on().instrument("bass").item("minecraft:light_blue_banner"),
    Block::of("minecraft:yellow_wall_banner", BlockClass::WallBannerBlock).properties(&[&HORIZONTAL_FACING], &["north"]).strength(1.0, 1.0).no_collision().ignited_by_lava().force_solid_on().instrument("bass").item("minecraft:yellow_banner"),
    Block::of("minecraft:lime_wall_banner", BlockClass::WallBannerBlock).properties(&[&HORIZONTAL_FACING], &["north"]).strength(1.0, 1.0).no_collision().ignited_by_lava().force_solid_on().instrument("bass").item("minecraft:lime_banner"),
    Block::of("minecraft:pink_wall_banner", BlockClass::WallBannerBlock).properties(&[&HORIZONTAL_FACING], &["north"]).strength(1.0, 1.0).no_collision().ignited_by_lava().force_solid_on().instrument("bass").item("minecraft:pink_banner"),
    Block::of("minecraft:gray_wall_banner", BlockClass::WallBannerBlock).properties(&[&HORIZONTAL_FACING], &["north"]).strength(1.0, 1.0).no_collision().ignited_by_lava().force_solid_on().instrument("bass").item("minecraft:gray_banner"),
    Block::of("minecraft:light_gray_wall_banner", BlockClass::WallBannerBlock).properties(&[&HORIZONTAL_FACING], &["north"]).strength(1.0, 1.0).no_collision().ignited_by_lava().force_solid_on().instrument("bass").item("minecraft:light_gray_banner"),
    Block::of("minecraft:cyan_wall_banner", BlockClass::WallBannerBlock).properties(&[&HORIZONTAL_FACING], &["north"]).strength(1.0, 1.0).no_collision().ignited_by_lava().force_solid_on().instrument("bass").item("minecraft:cyan_banner"),
    Block::of("minecraft:purple_wall_banner", BlockClass::WallBannerBlock).properties(&[&HORIZONTAL_FACING], &["north"]).strength(1.0, 1.0).no_collision().ignited_by_lava().force_solid_on().instrument("bass").item("minecraft:purple_banner"),
    Block::of("minecraft:blue_wall_banner", BlockClass::WallBannerBlock).properties(&[&HORIZONTAL_FACING], &["north"]).strength(1.0, 1.0).no_collision().ignited_by_lava().force_solid_on().instrument("bass").item("minecraft:blue_banner"),
    Block::of("minecraft:brown_wall_banner", BlockClass::WallBannerBlock).properties(&[&HORIZONTAL_FACING], &["north"]).strength(1.0, 1.0).no_collision().ignited_by_lava().force_solid_on().instrument("bass").item("minecraft:brown_banner"),
    Block::of("minecraft:green_wall_banner", BlockClass::WallBannerBlock).properties(&[&HORIZONTAL_FACING], &["north"]).strength(1.0, 1.0).no_collision().ignited_by_lava().force_solid_on().instrument("bass").item("minecraft:green_banner"),
    Block::of("minecraft:red_wall_banner", BlockClass::WallBannerBlock).properties(&[&HORIZONTAL_FACING], &["north"]).strength(1.0, 1.0).no_collision().ignited_by_lava().force_solid_on().instrument("bass").item("minecraft:red_banner"),
    Block::of("minecraft:black_wall_banner", BlockClass::WallBannerBlock).properties(&[&HORIZONTAL_FACING], &["north"]).strength(1.0, 1.0).no_collision().ignited_by_lava().force_solid_on().instrument("bass").item("minecraft:black_banner"),
    Block::of("minecraft:red_sandstone", BlockClass::Block).strength(0.8, 0.8).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:chiseled_red_sandstone", BlockClass::Block).strength(0.8, 0.8).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:cut_red_sandstone", BlockClass::Block).strength(0.8, 0.8).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:red_sandstone_stairs", BlockClass::StairBlock).properties(&[&HORIZONTAL_FACING, &HALF, &STAIRS_SHAPE, &WATERLOGGED], &["north", "bottom", "straight", "false"]).strength(0.8, 0.8).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:oak_slab", BlockClass::SlabBlock).properties(&[&SLAB_TYPE, &WATERLOGGED], &["bottom", "false"]).strength(2.0, 3.0).ignited_by_lava().instrument("bass"),
    Block::of("minecraft:spruce_slab", BlockClass::SlabBlock).properties(&[&SLAB_TYPE, &WATERLOGGED], &["bottom", "false"]).strength(2.0, 3.0).ignited_by_lava().instrument("bass"),
    Block::of("minecraft:birch_slab", BlockClass::SlabBlock).properties(&[&SLAB_TYPE, &WATERLOGGED], &["bottom", "false"]).strength(2.0, 3.0).ignited_by_lava().instrument("bass"),
    Block::of("minecraft:jungle_slab", BlockClass::SlabBlock).properties(&[&SLAB_TYPE, &WATERLOGGED], &["bottom", "false"]).strength(2.0, 3.0).ignited_by_lava().instrument("bass"),
    Block::of("minecraft:acacia_slab", BlockClass::SlabBlock).properties(&[&SLAB_TYPE, &WATERLOGGED], &["bottom", "false"]).strength(2.0, 3.0).ignited_by_lava().instrument("bass"),
    Block::of("minecraft:cherry_slab", BlockClass::SlabBlock).properties(&[&SLAB_TYPE, &WATERLOGGED], &["bottom", "false"]).strength(2.0, 3.0).ignited_by_lava().instrument("bass"),
    Block::of("minecraft:dark_oak_slab", BlockClass::SlabBlock).properties(&[&SLAB_TYPE, &WATERLOGGED], &["bottom", "false"]).strength(2.0, 3.0).ignited_by_lava().instrument("bass"),
    Block::of("minecraft:pale_oak_slab", BlockClass::SlabBlock).properties(&[&SLAB_TYPE, &WATERLOGGED], &["bottom", "false"]).strength(2.0, 3.0).ignited_by_lava().instrument("bass"),
    Block::of("minecraft:mangrove_slab", BlockClass::SlabBlock).properties(&[&SLAB_TYPE, &WATERLOGGED], &["bottom", "false"]).strength(2.0, 3.0).ignited_by_lava().instrument("bass"),
    Block::of("minecraft:bamboo_slab", BlockClass::SlabBlock).properties(&[&SLAB_TYPE, &WATERLOGGED], &["bottom", "false"]).strength(2.0, 3.0).ignited_by_lava().instrument("bass"),
    Block::of("minecraft:bamboo_mosaic_slab", BlockClass::SlabBlock).properties(&[&SLAB_TYPE, &WATERLOGGED], &["bottom", "false"]).strength(2.0, 3.0).ignited_by_lava().instrument("bass"),
    Block::of("minecraft:stone_slab", BlockClass::SlabBlock).properties(&[&SLAB_TYPE, &WATERLOGGED], &["bottom", "false"]).strength(2.0, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:smooth_stone_slab", BlockClass::SlabBlock).properties(&[&SLAB_TYPE, &WATERLOGGED], &["bottom", "false"]).strength(2.0, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:sandstone_slab", BlockClass::SlabBlock).properties(&[&SLAB_TYPE, &WATERLOGGED], &["bottom", "false"]).strength(2.0, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:cut_sandstone_slab", BlockClass::SlabBlock).properties(&[&SLAB_TYPE, &WATERLOGGED], &["bottom", "false"]).strength(2.0, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:petrified_oak_slab", BlockClass::SlabBlock).properties(&[&SLAB_TYPE, &WATERLOGGED], &["bottom", "false"]).strength(2.0, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:cobblestone_slab", BlockClass::SlabBlock).properties(&[&SLAB_TYPE, &WATERLOGGED], &["bottom", "false"]).strength(2.0, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:brick_slab", BlockClass::SlabBlock).properties(&[&SLAB_TYPE, &WATERLOGGED], &["bottom", "false"]).strength(2.0, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:stone_brick_slab", BlockClass::SlabBlock).properties(&[&SLAB_TYPE, &WATERLOGGED], &["bottom", "false"]).strength(2.0, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:mud_brick_slab", BlockClass::SlabBlock).properties(&[&SLAB_TYPE, &WATERLOGGED], &["bottom", "false"]).strength(1.5, 3.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:nether_brick_slab", BlockClass::SlabBlock).properties(&[&SLAB_TYPE, &WATERLOGGED], &["bottom", "false"]).strength(2.0, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:quartz_slab", BlockClass::SlabBlock).properties(&[&SLAB_TYPE, &WATERLOGGED], &["bottom", "false"]).strength(2.0, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:red_sandstone_slab", BlockClass::SlabBlock).properties(&[&SLAB_TYPE, &WATERLOGGED], &["bottom", "false"]).strength(2.0, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:cut_red_sandstone_slab", BlockClass::SlabBlock).properties(&[&SLAB_TYPE, &WATERLOGGED], &["bottom", "false"]).strength(2.0, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:purpur_slab", BlockClass::SlabBlock).properties(&[&SLAB_TYPE, &WATERLOGGED], &["bottom", "false"]).strength(2.0, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:smooth_stone", BlockClass::Block).strength(2.0, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:smooth_sandstone", BlockClass::Block).strength(2.0, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:smooth_quartz", BlockClass::Block).strength(2.0, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:smooth_red_sandstone", BlockClass::Block).strength(2.0, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:spruce_fence_gate", BlockClass::FenceGateBlock).properties(&[&HORIZONTAL_FACING, &IN_WALL, &OPEN, &POWERED], &["north", "false", "false", "false"]).strength(2.0, 3.0).ignited_by_lava().force_solid_on().instrument("bass"),
    Block::of("minecraft:birch_fence_gate", BlockClass::FenceGateBlock).properties(&[&HORIZONTAL_FACING, &IN_WALL, &OPEN, &POWERED], &["north", "false", "false", "false"]).strength(2.0, 3.0).ignited_by_lava().force_solid_on().instrument("bass"),
    Block::of("minecraft:jungle_fence_gate", BlockClass::FenceGateBlock).properties(&[&HORIZONTAL_FACING, &IN_WALL, &OPEN, &POWERED], &["north", "false", "false", "false"]).strength(2.0, 3.0).ignited_by_lava().force_solid_on().instrument("bass"),
    Block::of("minecraft:acacia_fence_gate", BlockClass::FenceGateBlock).properties(&[&HORIZONTAL_FACING, &IN_WALL, &OPEN, &POWERED], &["north", "false", "false", "false"]).strength(2.0, 3.0).ignited_by_lava().force_solid_on().instrument("bass"),
    Block::of("minecraft:cherry_fence_gate", BlockClass::FenceGateBlock).properties(&[&HORIZONTAL_FACING, &IN_WALL, &OPEN, &POWERED], &["north", "false", "false", "false"]).strength(2.0, 3.0).ignited_by_lava().force_solid_on().instrument("bass"),
    Block::of("minecraft:dark_oak_fence_gate", BlockClass::FenceGateBlock).properties(&[&HORIZONTAL_FACING, &IN_WALL, &OPEN, &POWERED], &["north", "false", "false", "false"]).strength(2.0, 3.0).ignited_by_lava().force_solid_on().instrument("bass"),
    Block::of("minecraft:pale_oak_fence_gate", BlockClass::FenceGateBlock).properties(&[&HORIZONTAL_FACING, &IN_WALL, &OPEN, &POWERED], &["north", "false", "false", "false"]).strength(2.0, 3.0).ignited_by_lava().force_solid_on().instrument("bass"),
    Block::of("minecraft:mangrove_fence_gate", BlockClass::FenceGateBlock).properties(&[&HORIZONTAL_FACING, &IN_WALL, &OPEN, &POWERED], &["north", "false", "false", "false"]).strength(2.0, 3.0).ignited_by_lava().force_solid_on().instrument("bass"),
    Block::of("minecraft:bamboo_fence_gate", BlockClass::FenceGateBlock).properties(&[&HORIZONTAL_FACING, &IN_WALL, &OPEN, &POWERED], &["north", "false", "false", "false"]).strength(2.0, 3.0).ignited_by_lava().force_solid_on().instrument("bass"),
    Block::of("minecraft:spruce_fence", BlockClass::FenceBlock).properties(&[&EAST, &NORTH, &SOUTH, &WATERLOGGED, &WEST], &["false", "false", "false", "false", "false"]).strength(2.0, 3.0).ignited_by_lava().instrument("bass"),
    Block::of("minecraft:birch_fence", BlockClass::FenceBlock).properties(&[&EAST, &NORTH, &SOUTH, &WATERLOGGED, &WEST], &["false", "false", "false", "false", "false"]).strength(2.0, 3.0).ignited_by_lava().instrument("bass"),
    Block::of("minecraft:jungle_fence", BlockClass::FenceBlock).properties(&[&EAST, &NORTH, &SOUTH, &WATERLOGGED, &WEST], &["false", "false", "false", "false", "false"]).strength(2.0, 3.0).ignited_by_lava().instrument("bass"),
    Block::of("minecraft:acacia_fence", BlockClass::FenceBlock).properties(&[&EAST, &NORTH, &SOUTH, &WATERLOGGED, &WEST], &["false", "false", "false", "false", "false"]).strength(2.0, 3.0).ignited_by_lava().instrument("bass"),
    Block::of("minecraft:cherry_fence", BlockClass::FenceBlock).properties(&[&EAST, &NORTH, &SOUTH, &WATERLOGGED, &WEST], &["false", "false", "false", "false", "false"]).strength(2.0, 3.0).ignited_by_lava().instrument("bass"),
    Block::of("minecraft:dark_oak_fence", BlockClass::FenceBlock).properties(&[&EAST, &NORTH, &SOUTH, &WATERLOGGED, &WEST], &["false", "false", "false", "false", "false"]).strength(2.0, 3.0).ignited_by_lava().instrument("bass"),
    Block::of("minecraft:pale_oak_fence", BlockClass::FenceBlock).properties(&[&EAST, &NORTH, &SOUTH, &WATERLOGGED, &WEST], &["false", "false", "false", "false", "false"]).strength(2.0, 3.0).ignited_by_lava().instrument("bass"),
    Block::of("minecraft:mangrove_fence", BlockClass::FenceBlock).properties(&[&EAST, &NORTH, &SOUTH, &WATERLOGGED, &WEST], &["false", "false", "false", "false", "false"]).strength(2.0, 3.0).ignited_by_lava().instrument("bass"),
    Block::of("minecraft:bamboo_fence", BlockClass::FenceBlock).properties(&[&EAST, &NORTH, &SOUTH, &WATERLOGGED, &WEST], &["false", "false", "false", "false", "false"]).strength(2.0, 3.0).ignited_by_lava().instrument("bass"),
    Block::of("minecraft:spruce_door", BlockClass::DoorBlock).properties(&[&HORIZONTAL_FACING, &DOUBLE_BLOCK_HALF, &DOOR_HINGE, &OPEN, &POWERED], &["north", "lower", "left", "false", "false"]).strength(3.0, 3.0).no_occlusion().ignited_by_lava().push_reaction(PushReaction::Destroy).instrument("bass"),
    Block::of("minecraft:birch_door", BlockClass::DoorBlock).properties(&[&HORIZONTAL_FACING, &DOUBLE_BLOCK_HALF, &DOOR_HINGE, &OPEN, &POWERED], &["north", "lower", "left", "false", "false"]).strength(3.0, 3.0).no_occlusion().ignited_by_lava().push_reaction(PushReaction::Destroy).instrument("bass"),
    Block::of("minecraft:jungle_door", BlockClass::DoorBlock).properties(&[&HORIZONTAL_FACING, &DOUBLE_BLOCK_HALF, &DOOR_HINGE, &OPEN, &POWERED], &["north", "lower", "left", "false", "false"]).strength(3.0, 3.0).no_occlusion().ignited_by_lava().push_reaction(PushReaction::Destroy).instrument("bass"),
    Block::of("minecraft:acacia_door", BlockClass::DoorBlock).properties(&[&HORIZONTAL_FACING, &DOUBLE_BLOCK_HALF, &DOOR_HINGE, &OPEN, &POWERED], &["north", "lower", "left", "false", "false"]).strength(3.0, 3.0).no_occlusion().ignited_by_lava().push_reaction(PushReaction::Destroy).instrument("bass"),
    Block::of("minecraft:cherry_door", BlockClass::DoorBlock).properties(&[&HORIZONTAL_FACING, &DOUBLE_BLOCK_HALF, &DOOR_HINGE, &OPEN, &POWERED], &["north", "lower", "left", "false", "false"]).strength(3.0, 3.0).no_occlusion().ignited_by_lava().push_reaction(PushReaction::Destroy).instrument("bass"),
    Block::of("minecraft:dark_oak_door", BlockClass::DoorBlock).properties(&[&HORIZONTAL_FACING, &DOUBLE_BLOCK_HALF, &DOOR_HINGE, &OPEN, &POWERED], &["north", "lower", "left", "false", "false"]).strength(3.0, 3.0).no_occlusion().ignited_by_lava().push_reaction(PushReaction::Destroy).instrument("bass"),
    Block::of("minecraft:pale_oak_door", BlockClass::DoorBlock).properties(&[&HORIZONTAL_FACING, &DOUBLE_BLOCK_HALF, &DOOR_HINGE, &OPEN, &POWERED], &["north", "lower", "left", "false", "false"]).strength(3.0, 3.0).no_occlusion().ignited_by_lava().push_reaction(PushReaction::Destroy).instrument("bass"),
    Block::of("minecraft:mangrove_door", BlockClass::DoorBlock).properties(&[&HORIZONTAL_FACING, &DOUBLE_BLOCK_HALF, &DOOR_HINGE, &OPEN, &POWERED], &["north", "lower", "left", "false", "false"]).strength(3.0, 3.0).no_occlusion().ignited_by_lava().push_reaction(PushReaction::Destroy).instrument("bass"),
    Block::of("minecraft:bamboo_door", BlockClass::DoorBlock).properties(&[&HORIZONTAL_FACING, &DOUBLE_BLOCK_HALF, &DOOR_HINGE, &OPEN, &POWERED], &["north", "lower", "left", "false", "false"]).strength(3.0, 3.0).no_occlusion().ignited_by_lava().push_reaction(PushReaction::Destroy).instrument("bass"),
    Block::of("minecraft:end_rod", BlockClass::EndRodBlock).properties(&[&FACING], &["up"]).no_occlusion().force_solid_off().light_level(Emission::Constant(14)),
    Block::of("minecraft:chorus_plant", BlockClass::ChorusPlantBlock).properties(&[&DOWN, &EAST, &NORTH, &SOUTH, &UP, &WEST], &["false", "false", "false", "false", "false", "false"]).strength(0.4, 0.4).no_occlusion().force_solid_off().push_reaction(PushReaction::Destroy),
    Block::of("minecraft:chorus_flower", BlockClass::ChorusFlowerBlock).properties(&[&AGE_5], &["0"]).strength(0.4, 0.4).no_occlusion().random_ticks().force_solid_off().push_reaction(PushReaction::Destroy).is_redstone_conductor(Rule::Never),
    Block::of("minecraft:purpur_block", BlockClass::Block).strength(1.5, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:purpur_pillar", BlockClass::RotatedPillarBlock).properties(&[&AXIS], &["y"]).strength(1.5, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:purpur_stairs", BlockClass::StairBlock).properties(&[&HORIZONTAL_FACING, &HALF, &STAIRS_SHAPE, &WATERLOGGED], &["north", "bottom", "straight", "false"]).strength(1.5, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:end_stone_bricks", BlockClass::Block).strength(3.0, 9.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:torchflower_crop", BlockClass::TorchflowerCropBlock).properties(&[&AGE_1], &["0"]).no_collision().random_ticks().push_reaction(PushReaction::Destroy).item("minecraft:torchflower_seeds"),
    Block::of("minecraft:pitcher_crop", BlockClass::PitcherCropBlock).properties(&[&AGE_4, &DOUBLE_BLOCK_HALF], &["0", "lower"]).no_collision().random_ticks().push_reaction(PushReaction::Destroy).item("minecraft:pitcher_pod"),
    Block::of("minecraft:pitcher_plant", BlockClass::DoublePlantBlock).properties(&[&DOUBLE_BLOCK_HALF], &["lower"]).no_collision().ignited_by_lava().offset_type(OffsetType::Xz).push_reaction(PushReaction::Destroy),
    Block::of("minecraft:beetroots", BlockClass::BeetrootBlock).properties(&[&AGE_3], &["0"]).no_collision().random_ticks().push_reaction(PushReaction::Destroy).item("minecraft:beetroot_seeds"),
    Block::of("minecraft:dirt_path", BlockClass::DirtPathBlock).strength(0.65, 0.65).is_suffocating(Rule::Always).is_view_blocking(Rule::Always),
    Block::of("minecraft:end_gateway", BlockClass::EndGatewayBlock).strength(-1.0, 3600000.0).no_collision().push_reaction(PushReaction::Block).item("minecraft:air").light_level(Emission::Constant(15)),
    Block::of("minecraft:repeating_command_block", BlockClass::CommandBlock).properties(&[&CONDITIONAL, &FACING], &["false", "north"]).strength(-1.0, 3600000.0).requires_correct_tool_for_drops(),
    Block::of("minecraft:chain_command_block", BlockClass::CommandBlock).properties(&[&CONDITIONAL, &FACING], &["false", "north"]).strength(-1.0, 3600000.0).requires_correct_tool_for_drops(),
    Block::of("minecraft:frosted_ice", BlockClass::FrostedIceBlock).properties(&[&AGE_3], &["0"]).strength(0.5, 0.5).friction(0.98).no_occlusion().item("minecraft:air").is_redstone_conductor(Rule::Never),
    Block::of("minecraft:magma_block", BlockClass::MagmaBlock).strength(0.5, 0.5).requires_correct_tool_for_drops().instrument("basedrum").light_level(Emission::Constant(3)).emissive_rendering(Rule::Always),
    Block::of("minecraft:nether_wart_block", BlockClass::Block).strength(1.0, 1.0),
    Block::of("minecraft:red_nether_bricks", BlockClass::Block).strength(2.0, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:bone_block", BlockClass::RotatedPillarBlock).properties(&[&AXIS], &["y"]).strength(2.0, 2.0).requires_correct_tool_for_drops().instrument("xylophone"),
    Block::of("minecraft:structure_void", BlockClass::StructureVoidBlock).no_collision().no_terrain_particles().replaceable().push_reaction(PushReaction::Destroy),
    Block::of("minecraft:observer", BlockClass::ObserverBlock).properties(&[&FACING, &POWERED], &["south", "false"]).strength(3.0, 3.0).requires_correct_tool_for_drops().instrument("basedrum").is_redstone_conductor(Rule::Never),
    Block::of("minecraft:shulker_box", BlockClass::ShulkerBoxBlock).properties(&[&FACING], &["up"]).strength(2.0, 2.0).no_occlusion().force_solid_on().dynamic_shape().push_reaction(PushReaction::Destroy).is_suffocating(Rule::Always).is_view_blocking(Rule::Always),
    Block::of("minecraft:white_shulker_box", BlockClass::ShulkerBoxBlock).properties(&[&FACING], &["up"]).strength(2.0, 2.0).no_occlusion().force_solid_on().dynamic_shape().push_reaction(PushReaction::Destroy).is_suffocating(Rule::Always).is_view_blocking(Rule::Always),
    Block::of("minecraft:orange_shulker_box", BlockClass::ShulkerBoxBlock).properties(&[&FACING], &["up"]).strength(2.0, 2.0).no_occlusion().force_solid_on().dynamic_shape().push_reaction(PushReaction::Destroy).is_suffocating(Rule::Always).is_view_blocking(Rule::Always),
    Block::of("minecraft:magenta_shulker_box", BlockClass::ShulkerBoxBlock).properties(&[&FACING], &["up"]).strength(2.0, 2.0).no_occlusion().force_solid_on().dynamic_shape().push_reaction(PushReaction::Destroy).is_suffocating(Rule::Always).is_view_blocking(Rule::Always),
    Block::of("minecraft:light_blue_shulker_box", BlockClass::ShulkerBoxBlock).properties(&[&FACING], &["up"]).strength(2.0, 2.0).no_occlusion().force_solid_on().dynamic_shape().push_reaction(PushReaction::Destroy).is_suffocating(Rule::Always).is_view_blocking(Rule::Always),
    Block::of("minecraft:yellow_shulker_box", BlockClass::ShulkerBoxBlock).properties(&[&FACING], &["up"]).strength(2.0, 2.0).no_occlusion().force_solid_on().dynamic_shape().push_reaction(PushReaction::Destroy).is_suffocating(Rule::Always).is_view_blocking(Rule::Always),
    Block::of("minecraft:lime_shulker_box", BlockClass::ShulkerBoxBlock).properties(&[&FACING], &["up"]).strength(2.0, 2.0).no_occlusion().force_solid_on().dynamic_shape().push_reaction(PushReaction::Destroy).is_suffocating(Rule::Always).is_view_blocking(Rule::Always),
    Block::of("minecraft:pink_shulker_box", BlockClass::ShulkerBoxBlock).properties(&[&FACING], &["up"]).strength(2.0, 2.0).no_occlusion().force_solid_on().dynamic_shape().push_reaction(PushReaction::Destroy).is_suffocating(Rule::Always).is_view_blocking(Rule::Always),
    Block::of("minecraft:gray_shulker_box", BlockClass::ShulkerBoxBlock).properties(&[&FACING], &["up"]).strength(2.0, 2.0).no_occlusion().force_solid_on().dynamic_shape().push_reaction(PushReaction::Destroy).is_suffocating(Rule::Always).is_view_blocking(Rule::Always),
    Block::of("minecraft:light_gray_shulker_box", BlockClass::ShulkerBoxBlock).properties(&[&FACING], &["up"]).strength(2.0, 2.0).no_occlusion().force_solid_on().dynamic_shape().push_reaction(PushReaction::Destroy).is_suffocating(Rule::Always).is_view_blocking(Rule::Always),
    Block::of("minecraft:cyan_shulker_box", BlockClass::ShulkerBoxBlock).properties(&[&FACING], &["up"]).strength(2.0, 2.0).no_occlusion().force_solid_on().dynamic_shape().push_reaction(PushReaction::Destroy).is_suffocating(Rule::Always).is_view_blocking(Rule::Always),
    Block::of("minecraft:purple_shulker_box", BlockClass::ShulkerBoxBlock).properties(&[&FACING], &["up"]).strength(2.0, 2.0).no_occlusion().force_solid_on().dynamic_shape().push_reaction(PushReaction::Destroy).is_suffocating(Rule::Always).is_view_blocking(Rule::Always),
    Block::of("minecraft:blue_shulker_box", BlockClass::ShulkerBoxBlock).properties(&[&FACING], &["up"]).strength(2.0, 2.0).no_occlusion().force_solid_on().dynamic_shape().push_reaction(PushReaction::Destroy).is_suffocating(Rule::Always).is_view_blocking(Rule::Always),
    Block::of("minecraft:brown_shulker_box", BlockClass::ShulkerBoxBlock).properties(&[&FACING], &["up"]).strength(2.0, 2.0).no_occlusion().force_solid_on().dynamic_shape().push_reaction(PushReaction::Destroy).is_suffocating(Rule::Always).is_view_blocking(Rule::Always),
    Block::of("minecraft:green_shulker_box", BlockClass::ShulkerBoxBlock).properties(&[&FACING], &["up"]).strength(2.0, 2.0).no_occlusion().force_solid_on().dynamic_shape().push_reaction(PushReaction::Destroy).is_suffocating(Rule::Always).is_view_blocking(Rule::Always),
    Block::of("minecraft:red_shulker_box", BlockClass::ShulkerBoxBlock).properties(&[&FACING], &["up"]).strength(2.0, 2.0).no_occlusion().force_solid_on().dynamic_shape().push_reaction(PushReaction::Destroy).is_suffocating(Rule::Always).is_view_blocking(Rule::Always),
    Block::of("minecraft:black_shulker_box", BlockClass::ShulkerBoxBlock).properties(&[&FACING], &["up"]).strength(2.0, 2.0).no_occlusion().force_solid_on().dynamic_shape().push_reaction(PushReaction::Destroy).is_suffocating(Rule::Always).is_view_blocking(Rule::Always),
    Block::of("minecraft:white_glazed_terracotta", BlockClass::GlazedTerracottaBlock).properties(&[&HORIZONTAL_FACING], &["north"]).strength(1.4, 1.4).requires_correct_tool_for_drops().push_reaction(PushReaction::PushOnly).instrument("basedrum"),
    Block::of("minecraft:orange_glazed_terracotta", BlockClass::GlazedTerracottaBlock).properties(&[&HORIZONTAL_FACING], &["north"]).strength(1.4, 1.4).requires_correct_tool_for_drops().push_reaction(PushReaction::PushOnly).instrument("basedrum"),
    Block::of("minecraft:magenta_glazed_terracotta", BlockClass::GlazedTerracottaBlock).properties(&[&HORIZONTAL_FACING], &["north"]).strength(1.4, 1.4).requires_correct_tool_for_drops().push_reaction(PushReaction::PushOnly).instrument("basedrum"),
    Block::of("minecraft:light_blue_glazed_terracotta", BlockClass::GlazedTerracottaBlock).properties(&[&HORIZONTAL_FACING], &["north"]).strength(1.4, 1.4).requires_correct_tool_for_drops().push_reaction(PushReaction::PushOnly).instrument("basedrum"),
    Block::of("minecraft:yellow_glazed_terracotta", BlockClass::GlazedTerracottaBlock).properties(&[&HORIZONTAL_FACING], &["north"]).strength(1.4, 1.4).requires_correct_tool_for_drops().push_reaction(PushReaction::PushOnly).instrument("basedrum"),
    Block::of("minecraft:lime_glazed_terracotta", BlockClass::GlazedTerracottaBlock).properties(&[&HORIZONTAL_FACING], &["north"]).strength(1.4, 1.4).requires_correct_tool_for_drops().push_reaction(PushReaction::PushOnly).instrument("basedrum"),
    Block::of("minecraft:pink_glazed_terracotta", BlockClass::GlazedTerracottaBlock).properties(&[&HORIZONTAL_FACING], &["north"]).strength(1.4, 1.4).requires_correct_tool_for_drops().push_reaction(PushReaction::PushOnly).instrument("basedrum"),
    Block::of("minecraft:gray_glazed_terracotta", BlockClass::GlazedTerracottaBlock).properties(&[&HORIZONTAL_FACING], &["north"]).strength(1.4, 1.4).requires_correct_tool_for_drops().push_reaction(PushReaction::PushOnly).instrument("basedrum"),
    Block::of("minecraft:light_gray_glazed_terracotta", BlockClass::GlazedTerracottaBlock).properties(&[&HORIZONTAL_FACING], &["north"]).strength(1.4, 1.4).requires_correct_tool_for_drops().push_reaction(PushReaction::PushOnly).instrument("basedrum"),
    Block::of("minecraft:cyan_glazed_terracotta", BlockClass::GlazedTerracottaBlock).properties(&[&HORIZONTAL_FACING], &["north"]).strength(1.4, 1.4).requires_correct_tool_for_drops().push_reaction(PushReaction::PushOnly).instrument("basedrum"),
    Block::of("minecraft:purple_glazed_terracotta", BlockClass::GlazedTerracottaBlock).properties(&[&HORIZONTAL_FACING], &["north"]).strength(1.4, 1.4).requires_correct_tool_for_drops().push_reaction(PushReaction::PushOnly).instrument("basedrum"),
    Block::of("minecraft:blue_glazed_terracotta", BlockClass::GlazedTerracottaBlock).properties(&[&HORIZONTAL_FACING], &["north"]).strength(1.4, 1.4).requires_correct_tool_for_drops().push_reaction(PushReaction::PushOnly).instrument("basedrum"),
    Block::of("minecraft:brown_glazed_terracotta", BlockClass::GlazedTerracottaBlock).properties(&[&HORIZONTAL_FACING], &["north"]).strength(1.4, 1.4).requires_correct_tool_for_drops().push_reaction(PushReaction::PushOnly).instrument("basedrum"),
    Block::of("minecraft:green_glazed_terracotta", BlockClass::GlazedTerracottaBlock).properties(&[&HORIZONTAL_FACING], &["north"]).strength(1.4, 1.4).requires_correct_tool_for_drops().push_reaction(PushReaction::PushOnly).instrument("basedrum"),
    Block::of("minecraft:red_glazed_terracotta", BlockClass::GlazedTerracottaBlock).properties(&[&HORIZONTAL_FACING], &["north"]).strength(1.4, 1.4).requires_correct_tool_for_drops().push_reaction(PushReaction::PushOnly).instrument("basedrum"),
    Block::of("minecraft:black_glazed_terracotta", BlockClass::GlazedTerracottaBlock).properties(&[&HORIZONTAL_FACING], &["north"]).strength(1.4, 1.4).requires_correct_tool_for_drops().push_reaction(PushReaction::PushOnly).instrument("basedrum"),
    Block::of("minecraft:white_concrete", BlockClass::Block).strength(1.8, 1.8).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:orange_concrete", BlockClass::Block).strength(1.8, 1.8).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:magenta_concrete", BlockClass::Block).strength(1.8, 1.8).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:light_blue_concrete", BlockClass::Block).strength(1.8, 1.8).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:yellow_concrete", BlockClass::Block).strength(1.8, 1.8).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:lime_concrete", BlockClass::Block).strength(1.8, 1.8).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:pink_concrete", BlockClass::Block).strength(1.8, 1.8).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:gray_concrete", BlockClass::Block).strength(1.8, 1.8).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:light_gray_concrete", BlockClass::Block).strength(1.8, 1.8).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:cyan_concrete", BlockClass::Block).strength(1.8, 1.8).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:purple_concrete", BlockClass::Block).strength(1.8, 1.8).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:blue_concrete", BlockClass::Block).strength(1.8, 1.8).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:brown_concrete", BlockClass::Block).strength(1.8, 1.8).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:green_concrete", BlockClass::Block).strength(1.8, 1.8).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:red_concrete", BlockClass::Block).strength(1.8, 1.8).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:black_concrete", BlockClass::Block).strength(1.8, 1.8).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:white_concrete_powder", BlockClass::ConcretePowderBlock).strength(0.5, 0.5).instrument("snare"),
    Block::of("minecraft:orange_concrete_powder", BlockClass::ConcretePowderBlock).strength(0.5, 0.5).instrument("snare"),
    Block::of("minecraft:magenta_concrete_powder", BlockClass::ConcretePowderBlock).strength(0.5, 0.5).instrument("snare"),
    Block::of("minecraft:light_blue_concrete_powder", BlockClass::ConcretePowderBlock).strength(0.5, 0.5).instrument("snare"),
    Block::of("minecraft:yellow_concrete_powder", BlockClass::ConcretePowderBlock).strength(0.5, 0.5).instrument("snare"),
    Block::of("minecraft:lime_concrete_powder", BlockClass::ConcretePowderBlock).strength(0.5, 0.5).instrument("snare"),
    Block::of("minecraft:pink_concrete_powder", BlockClass::ConcretePowderBlock).strength(0.5, 0.5).instrument("snare"),
    Block::of("minecraft:gray_concrete_powder", BlockClass::ConcretePowderBlock).strength(0.5, 0.5).instrument("snare"),
    Block::of("minecraft:light_gray_concrete_powder", BlockClass::ConcretePowderBlock).strength(0.5, 0.5).instrument("snare"),
    Block::of("minecraft:cyan_concrete_powder", BlockClass::ConcretePowderBlock).strength(0.5, 0.5).instrument("snare"),
    Block::of("minecraft:purple_concrete_powder", BlockClass::ConcretePowderBlock).strength(0.5, 0.5).instrument("snare"),
    Block::of("minecraft:blue_concrete_powder", BlockClass::ConcretePowderBlock).strength(0.5, 0.5).instrument("snare"),
    Block::of("minecraft:brown_concrete_powder", BlockClass::ConcretePowderBlock).strength(0.5, 0.5).instrument("snare"),
    Block::of("minecraft:green_concrete_powder", BlockClass::ConcretePowderBlock).strength(0.5, 0.5).instrument("snare"),
    Block::of("minecraft:red_concrete_powder", BlockClass::ConcretePowderBlock).strength(0.5, 0.5).instrument("snare"),
    Block::of("minecraft:black_concrete_powder", BlockClass::ConcretePowderBlock).strength(0.5, 0.5).instrument("snare"),
    Block::of("minecraft:kelp", BlockClass::KelpBlock).properties(&[&AGE_25], &["0"]).no_collision().random_ticks().push_reaction(PushReaction::Destroy),
    Block::of("minecraft:kelp_plant", BlockClass::KelpPlantBlock).no_collision().push_reaction(PushReaction::Destroy).item("minecraft:air"),
    Block::of("minecraft:dried_kelp_block", BlockClass::Block).strength(0.5, 2.5),
    Block::of("minecraft:turtle_egg", BlockClass::TurtleEggBlock).properties(&[&EGGS, &HATCH], &["1", "0"]).strength(0.5, 0.5).no_occlusion().random_ticks().force_solid_on().push_reaction(PushReaction::Destroy),
    Block::of("minecraft:sniffer_egg", BlockClass::SnifferEggBlock).properties(&[&HATCH], &["0"]).strength(0.5, 0.5).no_occlusion(),
    Block::of("minecraft:dried_ghast", BlockClass::DriedGhastBlock).properties(&[&HORIZONTAL_FACING, &DRIED_GHAST_HYDRATION_LEVELS, &WATERLOGGED], &["north", "0", "false"]).no_occlusion().random_ticks().force_solid_on(),
    Block::of("minecraft:dead_tube_coral_block", BlockClass::Block).strength(1.5, 6.0).requires_correct_tool_for_drops().force_solid_on().instrument("basedrum"),
    Block::of("minecraft:dead_brain_coral_block", BlockClass::Block).strength(1.5, 6.0).requires_correct_tool_for_drops().force_solid_on().instrument("basedrum"),
    Block::of("minecraft:dead_bubble_coral_block", BlockClass::Block).strength(1.5, 6.0).requires_correct_tool_for_drops().force_solid_on().instrument("basedrum"),
    Block::of("minecraft:dead_fire_coral_block", BlockClass::Block).strength(1.5, 6.0).requires_correct_tool_for_drops().force_solid_on().instrument("basedrum"),
    Block::of("minecraft:dead_horn_coral_block", BlockClass::Block).strength(1.5, 6.0).requires_correct_tool_for_drops().force_solid_on().instrument("basedrum"),
    Block::of("minecraft:tube_coral_block", BlockClass::CoralBlock).strength(1.5, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:brain_coral_block", BlockClass::CoralBlock).strength(1.5, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:bubble_coral_block", BlockClass::CoralBlock).strength(1.5, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:fire_coral_block", BlockClass::CoralBlock).strength(1.5, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:horn_coral_block", BlockClass::CoralBlock).strength(1.5, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:dead_tube_coral", BlockClass::BaseCoralPlantBlock).properties(&[&WATERLOGGED], &["true"]).no_collision().requires_correct_tool_for_drops().force_solid_on().instrument("basedrum"),
    Block::of("minecraft:dead_brain_coral", BlockClass::BaseCoralPlantBlock).properties(&[&WATERLOGGED], &["true"]).no_collision().requires_correct_tool_for_drops().force_solid_on().instrument("basedrum"),
    Block::of("minecraft:dead_bubble_coral", BlockClass::BaseCoralPlantBlock).properties(&[&WATERLOGGED], &["true"]).no_collision().requires_correct_tool_for_drops().force_solid_on().instrument("basedrum"),
    Block::of("minecraft:dead_fire_coral", BlockClass::BaseCoralPlantBlock).properties(&[&WATERLOGGED], &["true"]).no_collision().requires_correct_tool_for_drops().force_solid_on().instrument("basedrum"),
    Block::of("minecraft:dead_horn_coral", BlockClass::BaseCoralPlantBlock).properties(&[&WATERLOGGED], &["true"]).no_collision().requires_correct_tool_for_drops().force_solid_on().instrument("basedrum"),
    Block::of("minecraft:tube_coral", BlockClass::CoralPlantBlock).properties(&[&WATERLOGGED], &["true"]).no_collision().push_reaction(PushReaction::Destroy),
    Block::of("minecraft:brain_coral", BlockClass::CoralPlantBlock).properties(&[&WATERLOGGED], &["true"]).no_collision().push_reaction(PushReaction::Destroy),
    Block::of("minecraft:bubble_coral", BlockClass::CoralPlantBlock).properties(&[&WATERLOGGED], &["true"]).no_collision().push_reaction(PushReaction::Destroy),
    Block::of("minecraft:fire_coral", BlockClass::CoralPlantBlock).properties(&[&WATERLOGGED], &["true"]).no_collision().push_reaction(PushReaction::Destroy),
    Block::of("minecraft:horn_coral", BlockClass::CoralPlantBlock).properties(&[&WATERLOGGED], &["true"]).no_collision().push_reaction(PushReaction::Destroy),
    Block::of("minecraft:dead_tube_coral_fan", BlockClass::BaseCoralFanBlock).properties(&[&WATERLOGGED], &["true"]).no_collision().requires_correct_tool_for_drops().force_solid_on().instrument("basedrum"),
    Block::of("minecraft:dead_brain_coral_fan", BlockClass::BaseCoralFanBlock).properties(&[&WATERLOGGED], &["true"]).no_collision().requires_correct_tool_for_drops().force_solid_on().instrument("basedrum"),
    Block::of("minecraft:dead_bubble_coral_fan", BlockClass::BaseCoralFanBlock).properties(&[&WATERLOGGED], &["true"]).no_collision().requires_correct_tool_for_drops().force_solid_on().instrument("basedrum"),
    Block::of("minecraft:dead_fire_coral_fan", BlockClass::BaseCoralFanBlock).properties(&[&WATERLOGGED], &["true"]).no_collision().requires_correct_tool_for_drops().force_solid_on().instrument("basedrum"),
    Block::of("minecraft:dead_horn_coral_fan", BlockClass::BaseCoralFanBlock).properties(&[&WATERLOGGED], &["true"]).no_collision().requires_correct_tool_for_drops().force_solid_on().instrument("basedrum"),
    Block::of("minecraft:tube_coral_fan", BlockClass::CoralFanBlock).properties(&[&WATERLOGGED], &["true"]).no_collision().push_reaction(PushReaction::Destroy),
    Block::of("minecraft:brain_coral_fan", BlockClass::CoralFanBlock).properties(&[&WATERLOGGED], &["true"]).no_collision().push_reaction(PushReaction::Destroy),
    Block::of("minecraft:bubble_coral_fan", BlockClass::CoralFanBlock).properties(&[&WATERLOGGED], &["true"]).no_collision().push_reaction(PushReaction::Destroy),
    Block::of("minecraft:fire_coral_fan", BlockClass::CoralFanBlock).properties(&[&WATERLOGGED], &["true"]).no_collision().push_reaction(PushReaction::Destroy),
    Block::of("minecraft:horn_coral_fan", BlockClass::CoralFanBlock).properties(&[&WATERLOGGED], &["true"]).no_collision().push_reaction(PushReaction::Destroy),
    Block::of("minecraft:dead_tube_coral_wall_fan", BlockClass::BaseCoralWallFanBlock).properties(&[&HORIZONTAL_FACING, &WATERLOGGED], &["north", "true"]).no_collision().requires_correct_tool_for_drops().force_solid_on().instrument("basedrum").item("minecraft:dead_tube_coral_fan"),
    Block::of("minecraft:dead_brain_coral_wall_fan", BlockClass::BaseCoralWallFanBlock).properties(&[&HORIZONTAL_FACING, &WATERLOGGED], &["north", "true"]).no_collision().requires_correct_tool_for_drops().force_solid_on().instrument("basedrum").item("minecraft:dead_brain_coral_fan"),
    Block::of("minecraft:dead_bubble_coral_wall_fan", BlockClass::BaseCoralWallFanBlock).properties(&[&HORIZONTAL_FACING, &WATERLOGGED], &["north", "true"]).no_collision().requires_correct_tool_for_drops().force_solid_on().instrument("basedrum").item("minecraft:dead_bubble_coral_fan"),
    Block::of("minecraft:dead_fire_coral_wall_fan", BlockClass::BaseCoralWallFanBlock).properties(&[&HORIZONTAL_FACING, &WATERLOGGED], &["north", "true"]).no_collision().requires_correct_tool_for_drops().force_solid_on().instrument("basedrum").item("minecraft:dead_fire_coral_fan"),
    Block::of("minecraft:dead_horn_coral_wall_fan", BlockClass::BaseCoralWallFanBlock).properties(&[&HORIZONTAL_FACING, &WATERLOGGED], &["north", "true"]).no_collision().requires_correct_tool_for_drops().force_solid_on().instrument("basedrum").item("minecraft:dead_horn_coral_fan"),
    Block::of("minecraft:tube_coral_wall_fan", BlockClass::CoralWallFanBlock).properties(&[&HORIZONTAL_FACING, &WATERLOGGED], &["north", "true"]).no_collision().push_reaction(PushReaction::Destroy).item("minecraft:tube_coral_fan"),
    Block::of("minecraft:brain_coral_wall_fan", BlockClass::CoralWallFanBlock).properties(&[&HORIZONTAL_FACING, &WATERLOGGED], &["north", "true"]).no_collision().push_reaction(PushReaction::Destroy).item("minecraft:brain_coral_fan"),
    Block::of("minecraft:bubble_coral_wall_fan", BlockClass::CoralWallFanBlock).properties(&[&HORIZONTAL_FACING, &WATERLOGGED], &["north", "true"]).no_collision().push_reaction(PushReaction::Destroy).item("minecraft:bubble_coral_fan"),
    Block::of("minecraft:fire_coral_wall_fan", BlockClass::CoralWallFanBlock).properties(&[&HORIZONTAL_FACING, &WATERLOGGED], &["north", "true"]).no_collision().push_reaction(PushReaction::Destroy).item("minecraft:fire_coral_fan"),
    Block::of("minecraft:horn_coral_wall_fan", BlockClass::CoralWallFanBlock).properties(&[&HORIZONTAL_FACING, &WATERLOGGED], &["north", "true"]).no_collision().push_reaction(PushReaction::Destroy).item("minecraft:horn_coral_fan"),
    Block::of("minecraft:sea_pickle", BlockClass::SeaPickleBlock).properties(&[&PICKLES, &WATERLOGGED], &["1", "true"]).no_occlusion().push_reaction(PushReaction::Destroy).light_level(Emission::States(&[6, 0, 9, 0, 12, 0, 15, 0])),
    Block::of("minecraft:blue_ice", BlockClass::HalfTransparentBlock).strength(2.8, 2.8).friction(0.989),
    Block::of("minecraft:conduit", BlockClass::ConduitBlock).properties(&[&WATERLOGGED], &["true"]).strength(3.0, 3.0).no_occlusion().force_solid_on().instrument("hat").light_level(Emission::Constant(15)),
    Block::of("minecraft:bamboo_sapling", BlockClass::BambooSaplingBlock).strength(1.0, 1.0).no_collision().random_ticks().ignited_by_lava().force_solid_on().offset_type(OffsetType::Xz).push_reaction(PushReaction::Destroy).item("minecraft:air"),
    Block::of("minecraft:bamboo", BlockClass::BambooStalkBlock).properties(&[&AGE_1, &BAMBOO_LEAVES, &STAGE], &["0", "none", "0"]).strength(1.0, 1.0).no_occlusion().random_ticks().ignited_by_lava().force_solid_on().dynamic_shape().offset_type(OffsetType::Xz).push_reaction(PushReaction::Destroy).is_redstone_conductor(Rule::Never),
    Block::of("minecraft:potted_bamboo", BlockClass::FlowerPotBlock).no_occlusion().push_reaction(PushReaction::Destroy).item("minecraft:air"),
    Block::of("minecraft:void_air", BlockClass::AirBlock).no_collision().air().replaceable().item("minecraft:air"),
    Block::of("minecraft:cave_air", BlockClass::AirBlock).no_collision().air().replaceable().item("minecraft:air"),
    Block::of("minecraft:bubble_column", BlockClass::BubbleColumnBlock).properties(&[&DRAG], &["true"]).no_collision().liquid().replaceable().push_reaction(PushReaction::Destroy).item("minecraft:air"),
    Block::of("minecraft:polished_granite_stairs", BlockClass::StairBlock).properties(&[&HORIZONTAL_FACING, &HALF, &STAIRS_SHAPE, &WATERLOGGED], &["north", "bottom", "straight", "false"]).strength(1.5, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:smooth_red_sandstone_stairs", BlockClass::StairBlock).properties(&[&HORIZONTAL_FACING, &HALF, &STAIRS_SHAPE, &WATERLOGGED], &["north", "bottom", "straight", "false"]).strength(2.0, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:mossy_stone_brick_stairs", BlockClass::StairBlock).properties(&[&HORIZONTAL_FACING, &HALF, &STAIRS_SHAPE, &WATERLOGGED], &["north", "bottom", "straight", "false"]).strength(1.5, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:polished_diorite_stairs", BlockClass::StairBlock).properties(&[&HORIZONTAL_FACING, &HALF, &STAIRS_SHAPE, &WATERLOGGED], &["north", "bottom", "straight", "false"]).strength(1.5, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:mossy_cobblestone_stairs", BlockClass::StairBlock).properties(&[&HORIZONTAL_FACING, &HALF, &STAIRS_SHAPE, &WATERLOGGED], &["north", "bottom", "straight", "false"]).strength(2.0, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:end_stone_brick_stairs", BlockClass::StairBlock).properties(&[&HORIZONTAL_FACING, &HALF, &STAIRS_SHAPE, &WATERLOGGED], &["north", "bottom", "straight", "false"]).strength(3.0, 9.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:stone_stairs", BlockClass::StairBlock).properties(&[&HORIZONTAL_FACING, &HALF, &STAIRS_SHAPE, &WATERLOGGED], &["north", "bottom", "straight", "false"]).strength(1.5, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:smooth_sandstone_stairs", BlockClass::StairBlock).properties(&[&HORIZONTAL_FACING, &HALF, &STAIRS_SHAPE, &WATERLOGGED], &["north", "bottom", "straight", "false"]).strength(2.0, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:smooth_quartz_stairs", BlockClass::StairBlock).properties(&[&HORIZONTAL_FACING, &HALF, &STAIRS_SHAPE, &WATERLOGGED], &["north", "bottom", "straight", "false"]).strength(2.0, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:granite_stairs", BlockClass::StairBlock).properties(&[&HORIZONTAL_FACING, &HALF, &STAIRS_SHAPE, &WATERLOGGED], &["north", "bottom", "straight", "false"]).strength(1.5, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:andesite_stairs", BlockClass::StairBlock).properties(&[&HORIZONTAL_FACING, &HALF, &STAIRS_SHAPE, &WATERLOGGED], &["north", "bottom", "straight", "false"]).strength(1.5, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:red_nether_brick_stairs", BlockClass::StairBlock).properties(&[&HORIZONTAL_FACING, &HALF, &STAIRS_SHAPE, &WATERLOGGED], &["north", "bottom", "straight", "false"]).strength(2.0, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:polished_andesite_stairs", BlockClass::StairBlock).properties(&[&HORIZONTAL_FACING, &HALF, &STAIRS_SHAPE, &WATERLOGGED], &["north", "bottom", "straight", "false"]).strength(1.5, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:diorite_stairs", BlockClass::StairBlock).properties(&[&HORIZONTAL_FACING, &HALF, &STAIRS_SHAPE, &WATERLOGGED], &["north", "bottom", "straight", "false"]).strength(1.5, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:polished_granite_slab", BlockClass::SlabBlock).properties(&[&SLAB_TYPE, &WATERLOGGED], &["bottom", "false"]).strength(1.5, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:smooth_red_sandstone_slab", BlockClass::SlabBlock).properties(&[&SLAB_TYPE, &WATERLOGGED], &["bottom", "false"]).strength(2.0, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:mossy_stone_brick_slab", BlockClass::SlabBlock).properties(&[&SLAB_TYPE, &WATERLOGGED], &["bottom", "false"]).strength(1.5, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:polished_diorite_slab", BlockClass::SlabBlock).properties(&[&SLAB_TYPE, &WATERLOGGED], &["bottom", "false"]).strength(1.5, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:mossy_cobblestone_slab", BlockClass::SlabBlock).properties(&[&SLAB_TYPE, &WATERLOGGED], &["bottom", "false"]).strength(2.0, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:end_stone_brick_slab", BlockClass::SlabBlock).properties(&[&SLAB_TYPE, &WATERLOGGED], &["bottom", "false"]).strength(3.0, 9.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:smooth_sandstone_slab", BlockClass::SlabBlock).properties(&[&SLAB_TYPE, &WATERLOGGED], &["bottom", "false"]).strength(2.0, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:smooth_quartz_slab", BlockClass::SlabBlock).properties(&[&SLAB_TYPE, &WATERLOGGED], &["bottom", "false"]).strength(2.0, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:granite_slab", BlockClass::SlabBlock).properties(&[&SLAB_TYPE, &WATERLOGGED], &["bottom", "false"]).strength(1.5, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:andesite_slab", BlockClass::SlabBlock).properties(&[&SLAB_TYPE, &WATERLOGGED], &["bottom", "false"]).strength(1.5, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:red_nether_brick_slab", BlockClass::SlabBlock).properties(&[&SLAB_TYPE, &WATERLOGGED], &["bottom", "false"]).strength(2.0, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:polished_andesite_slab", BlockClass::SlabBlock).properties(&[&SLAB_TYPE, &WATERLOGGED], &["bottom", "false"]).strength(1.5, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:diorite_slab", BlockClass::SlabBlock).properties(&[&SLAB_TYPE, &WATERLOGGED], &["bottom", "false"]).strength(1.5, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:brick_wall", BlockClass::WallBlock).properties(&[&EAST_WALL, &NORTH_WALL, &SOUTH_WALL, &UP, &WATERLOGGED, &WEST_WALL], &["none", "none", "none", "true", "false", "none"]).strength(2.0, 6.0).requires_correct_tool_for_drops().force_solid_on().instrument("basedrum"),
    Block::of("minecraft:prismarine_wall", BlockClass::WallBlock).properties(&[&EAST_WALL, &NORTH_WALL, &SOUTH_WALL, &UP, &WATERLOGGED, &WEST_WALL], &["none", "none", "none", "true", "false", "none"]).strength(1.5, 6.0).requires_correct_tool_for_drops().force_solid_on().instrument("basedrum"),
    Block::of("minecraft:red_sandstone_wall", BlockClass::WallBlock).properties(&[&EAST_WALL, &NORTH_WALL, &SOUTH_WALL, &UP, &WATERLOGGED, &WEST_WALL], &["none", "none", "none", "true", "false", "none"]).strength(0.8, 0.8).requires_correct_tool_for_drops().force_solid_on().instrument("basedrum"),
    Block::of("minecraft:mossy_stone_brick_wall", BlockClass::WallBlock).properties(&[&EAST_WALL, &NORTH_WALL, &SOUTH_WALL, &UP, &WATERLOGGED, &WEST_WALL], &["none", "none", "none", "true", "false", "none"]).strength(1.5, 6.0).requires_correct_tool_for_drops().force_solid_on().instrument("basedrum"),
    Block::of("minecraft:granite_wall", BlockClass::WallBlock).properties(&[&EAST_WALL, &NORTH_WALL, &SOUTH_WALL, &UP, &WATERLOGGED, &WEST_WALL], &["none", "none", "none", "true", "false", "none"]).strength(1.5, 6.0).requires_correct_tool_for_drops().force_solid_on().instrument("basedrum"),
    Block::of("minecraft:stone_brick_wall", BlockClass::WallBlock).properties(&[&EAST_WALL, &NORTH_WALL, &SOUTH_WALL, &UP, &WATERLOGGED, &WEST_WALL], &["none", "none", "none", "true", "false", "none"]).strength(1.5, 6.0).requires_correct_tool_for_drops().force_solid_on().instrument("basedrum"),
    Block::of("minecraft:mud_brick_wall", BlockClass::WallBlock).properties(&[&EAST_WALL, &NORTH_WALL, &SOUTH_WALL, &UP, &WATERLOGGED, &WEST_WALL], &["none", "none", "none", "true", "false", "none"]).strength(1.5, 3.0).requires_correct_tool_for_drops().force_solid_on().instrument("basedrum"),
    Block::of("minecraft:nether_brick_wall", BlockClass::WallBlock).properties(&[&EAST_WALL, &NORTH_WALL, &SOUTH_WALL, &UP, &WATERLOGGED, &WEST_WALL], &["none", "none", "none", "true", "false", "none"]).strength(2.0, 6.0).requires_correct_tool_for_drops().force_solid_on().instrument("basedrum"),
    Block::of("minecraft:andesite_wall", BlockClass::WallBlock).properties(&[&EAST_WALL, &NORTH_WALL, &SOUTH_WALL, &UP, &WATERLOGGED, &WEST_WALL], &["none", "none", "none", "true", "false", "none"]).strength(1.5, 6.0).requires_correct_tool_for_drops().force_solid_on().instrument("basedrum"),
    Block::of("minecraft:red_nether_brick_wall", BlockClass::WallBlock).properties(&[&EAST_WALL, &NORTH_WALL, &SOUTH_WALL, &UP, &WATERLOGGED, &WEST_WALL], &["none", "none", "none", "true", "false", "none"]).strength(2.0, 6.0).requires_correct_tool_for_drops().force_solid_on().instrument("basedrum"),
    Block::of("minecraft:sandstone_wall", BlockClass::WallBlock).properties(&[&EAST_WALL, &NORTH_WALL, &SOUTH_WALL, &UP, &WATERLOGGED, &WEST_WALL], &["none", "none", "none", "true", "false", "none"]).strength(0.8, 0.8).requires_correct_tool_for_drops().force_solid_on().instrument("basedrum"),
    Block::of("minecraft:end_stone_brick_wall", BlockClass::WallBlock).properties(&[&EAST_WALL, &NORTH_WALL, &SOUTH_WALL, &UP, &WATERLOGGED, &WEST_WALL], &["none", "none", "none", "true", "false", "none"]).strength(3.0, 9.0).requires_correct_tool_for_drops().force_solid_on().instrument("basedrum"),
    Block::of("minecraft:diorite_wall", BlockClass::WallBlock).properties(&[&EAST_WALL, &NORTH_WALL, &SOUTH_WALL, &UP, &WATERLOGGED, &WEST_WALL], &["none", "none", "none", "true", "false", "none"]).strength(1.5, 6.0).requires_correct_tool_for_drops().force_solid_on().instrument("basedrum"),
    Block::of("minecraft:scaffolding", BlockClass::ScaffoldingBlock).properties(&[&BOTTOM, &STABILITY_DISTANCE, &WATERLOGGED], &["false", "7", "false"]).no_collision().dynamic_shape().push_reaction(PushReaction::Destroy).is_redstone_conductor(Rule::Never),
    Block::of("minecraft:loom", BlockClass::LoomBlock).properties(&[&HORIZONTAL_FACING], &["north"]).strength(2.5, 2.5).ignited_by_lava().instrument("bass"),
    Block::of("minecraft:barrel", BlockClass::BarrelBlock).properties(&[&FACING, &OPEN], &["north", "false"]).strength(2.5, 2.5).ignited_by_lava().instrument("bass"),
    Block::of("minecraft:smoker", BlockClass::SmokerBlock).properties(&[&HORIZONTAL_FACING, &LIT], &["north", "false"]).strength(3.5, 3.5).requires_correct_tool_for_drops().instrument("basedrum").light_level(Emission::States(&[13, 0, 13, 0, 13, 0, 13, 0])),
    Block::of("minecraft:blast_furnace", BlockClass::BlastFurnaceBlock).properties(&[&HORIZONTAL_FACING, &LIT], &["north", "false"]).strength(3.5, 3.5).requires_correct_tool_for_drops().instrument("basedrum").light_level(Emission::States(&[13, 0, 13, 0, 13, 0, 13, 0])),
    Block::of("minecraft:cartography_table", BlockClass::CartographyTableBlock).strength(2.5, 2.5).ignited_by_lava().instrument("bass"),
    Block::of("minecraft:fletching_table", BlockClass::Block).strength(2.5, 2.5).ignited_by_lava().instrument("bass"),
    Block::of("minecraft:grindstone", BlockClass::GrindstoneBlock).properties(&[&ATTACH_FACE, &HORIZONTAL_FACING], &["wall", "north"]).strength(2.0, 6.0).requires_correct_tool_for_drops().push_reaction(PushReaction::Block),
    Block::of("minecraft:lectern", BlockClass::LecternBlock).properties(&[&HORIZONTAL_FACING, &HAS_BOOK, &POWERED], &["north", "false", "false"]).strength(2.5, 2.5).ignited_by_lava().instrument("bass"),
    Block::of("minecraft:smithing_table", BlockClass::SmithingTableBlock).strength(2.5, 2.5).ignited_by_lava().instrument("bass"),
    Block::of("minecraft:stonecutter", BlockClass::StonecutterBlock).properties(&[&HORIZONTAL_FACING], &["north"]).strength(3.5, 3.5).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:bell", BlockClass::BellBlock).properties(&[&BELL_ATTACHMENT, &HORIZONTAL_FACING, &POWERED], &["floor", "north", "false"]).strength(5.0, 5.0).force_solid_on().push_reaction(PushReaction::Destroy),
    Block::of("minecraft:lantern", BlockClass::LanternBlock).properties(&[&HANGING, &WATERLOGGED], &["false", "false"]).strength(3.5, 3.5).no_occlusion().force_solid_on().push_reaction(PushReaction::Destroy).light_level(Emission::Constant(15)),
    Block::of("minecraft:soul_lantern", BlockClass::LanternBlock).properties(&[&HANGING, &WATERLOGGED], &["false", "false"]).strength(3.5, 3.5).no_occlusion().force_solid_on().push_reaction(PushReaction::Destroy).light_level(Emission::Constant(10)),
    Block::of("minecraft:copper_lantern", BlockClass::WeatheringLanternBlock).properties(&[&HANGING, &WATERLOGGED], &["false", "false"]).strength(3.5, 3.5).no_occlusion().force_solid_on().push_reaction(PushReaction::Destroy).light_level(Emission::Constant(15)),
    Block::of("minecraft:exposed_copper_lantern", BlockClass::WeatheringLanternBlock).properties(&[&HANGING, &WATERLOGGED], &["false", "false"]).strength(3.5, 3.5).no_occlusion().force_solid_on().push_reaction(PushReaction::Destroy).light_level(Emission::Constant(15)),
    Block::of("minecraft:weathered_copper_lantern", BlockClass::WeatheringLanternBlock).properties(&[&HANGING, &WATERLOGGED], &["false", "false"]).strength(3.5, 3.5).no_occlusion().force_solid_on().push_reaction(PushReaction::Destroy).light_level(Emission::Constant(15)),
    Block::of("minecraft:oxidized_copper_lantern", BlockClass::WeatheringLanternBlock).properties(&[&HANGING, &WATERLOGGED], &["false", "false"]).strength(3.5, 3.5).no_occlusion().force_solid_on().push_reaction(PushReaction::Destroy).light_level(Emission::Constant(15)),
    Block::of("minecraft:waxed_copper_lantern", BlockClass::LanternBlock).properties(&[&HANGING, &WATERLOGGED], &["false", "false"]).strength(3.5, 3.5).no_occlusion().force_solid_on().push_reaction(PushReaction::Destroy).light_level(Emission::Constant(15)),
    Block::of("minecraft:waxed_exposed_copper_lantern", BlockClass::LanternBlock).properties(&[&HANGING, &WATERLOGGED], &["false", "false"]).strength(3.5, 3.5).no_occlusion().force_solid_on().push_reaction(PushReaction::Destroy).light_level(Emission::Constant(15)),
    Block::of("minecraft:waxed_weathered_copper_lantern", BlockClass::LanternBlock).properties(&[&HANGING, &WATERLOGGED], &["false", "false"]).strength(3.5, 3.5).no_occlusion().force_solid_on().push_reaction(PushReaction::Destroy).light_level(Emission::Constant(15)),
    Block::of("minecraft:waxed_oxidized_copper_lantern", BlockClass::LanternBlock).properties(&[&HANGING, &WATERLOGGED], &["false", "false"]).strength(3.5, 3.5).no_occlusion().force_solid_on().push_reaction(PushReaction::Destroy).light_level(Emission::Constant(15)),
    Block::of("minecraft:campfire", BlockClass::CampfireBlock).properties(&[&HORIZONTAL_FACING, &LIT, &SIGNAL_FIRE, &WATERLOGGED], &["north", "true", "false", "false"]).strength(2.0, 2.0).no_occlusion().ignited_by_lava().instrument("bass").light_level(Emission::States(&[15, 15, 15, 15, 0, 0, 0, 0, 15, 15, 15, 15, 0, 0, 0, 0, 15, 15, 15, 15, 0, 0, 0, 0, 15, 15, 15, 15, 0, 0, 0, 0])),
    Block::of("minecraft:soul_campfire", BlockClass::CampfireBlock).properties(&[&HORIZONTAL_FACING, &LIT, &SIGNAL_FIRE, &WATERLOGGED], &["north", "true", "false", "false"]).strength(2.0, 2.0).no_occlusion().ignited_by_lava().instrument("bass").light_level(Emission::States(&[10, 10, 10, 10, 0, 0, 0, 0, 10, 10, 10, 10, 0, 0, 0, 0, 10, 10, 10, 10, 0, 0, 0, 0, 10, 10, 10, 10, 0, 0, 0, 0])),
    Block::of("minecraft:sweet_berry_bush", BlockClass::SweetBerryBushBlock).properties(&[&AGE_3], &["0"]).no_collision().random_ticks().push_reaction(PushReaction::Destroy).item("minecraft:sweet_berries"),
    Block::of("minecraft:warped_stem", BlockClass::RotatedPillarBlock).properties(&[&AXIS], &["y"]).strength(2.0, 2.0).instrument("bass"),
    Block::of("minecraft:stripped_warped_stem", BlockClass::RotatedPillarBlock).properties(&[&AXIS], &["y"]).strength(2.0, 2.0).instrument("bass"),
    Block::of("minecraft:warped_hyphae", BlockClass::RotatedPillarBlock).properties(&[&AXIS], &["y"]).strength(2.0, 2.0).instrument("bass"),
    Block::of("minecraft:stripped_warped_hyphae", BlockClass::RotatedPillarBlock).properties(&[&AXIS], &["y"]).strength(2.0, 2.0).instrument("bass"),
    Block::of("minecraft:warped_nylium", BlockClass::NyliumBlock).strength(0.4, 0.4).requires_correct_tool_for_drops().random_ticks().instrument("basedrum"),
    Block::of("minecraft:warped_fungus", BlockClass::NetherFungusBlock).no_collision().push_reaction(PushReaction::Destroy),
    Block::of("minecraft:warped_wart_block", BlockClass::Block).strength(1.0, 1.0),
    Block::of("minecraft:warped_roots", BlockClass::NetherRootsBlock).no_collision().replaceable().offset_type(OffsetType::Xz).push_reaction(PushReaction::Destroy),
    Block::of("minecraft:nether_sprouts", BlockClass::NetherSproutsBlock).no_collision().replaceable().offset_type(OffsetType::Xz).push_reaction(PushReaction::Destroy),
    Block::of("minecraft:crimson_stem", BlockClass::RotatedPillarBlock).properties(&[&AXIS], &["y"]).strength(2.0, 2.0).instrument("bass"),
    Block::of("minecraft:stripped_crimson_stem", BlockClass::RotatedPillarBlock).properties(&[&AXIS], &["y"]).strength(2.0, 2.0).instrument("bass"),
    Block::of("minecraft:crimson_hyphae", BlockClass::RotatedPillarBlock).properties(&[&AXIS], &["y"]).strength(2.0, 2.0).instrument("bass"),
    Block::of("minecraft:stripped_crimson_hyphae", BlockClass::RotatedPillarBlock).properties(&[&AXIS], &["y"]).strength(2.0, 2.0).instrument("bass"),
    Block::of("minecraft:crimson_nylium", BlockClass::NyliumBlock).strength(0.4, 0.4).requires_correct_tool_for_drops().random_ticks().instrument("basedrum"),
    Block::of("minecraft:crimson_fungus", BlockClass::NetherFungusBlock).no_collision().push_reaction(PushReaction::Destroy),
    Block::of("minecraft:shroomlight", BlockClass::Block).strength(1.0, 1.0).light_level(Emission::Constant(15)),
    Block::of("minecraft:weeping_vines", BlockClass::WeepingVinesBlock).properties(&[&AGE_25], &["0"]).no_collision().random_ticks().push_reaction(PushReaction::Destroy),
    Block::of("minecraft:weeping_vines_plant", BlockClass::WeepingVinesPlantBlock).no_collision().push_reaction(PushReaction::Destroy).item("minecraft:air"),
    Block::of("minecraft:twisting_vines", BlockClass::TwistingVinesBlock).properties(&[&AGE_25], &["0"]).no_collision().random_ticks().push_reaction(PushReaction::Destroy),
    Block::of("minecraft:twisting_vines_plant", BlockClass::TwistingVinesPlantBlock).no_collision().push_reaction(PushReaction::Destroy).item("minecraft:air"),
    Block::of("minecraft:crimson_roots", BlockClass::NetherRootsBlock).no_collision().replaceable().offset_type(OffsetType::Xz).push_reaction(PushReaction::Destroy),
    Block::of("minecraft:crimson_planks", BlockClass::Block).strength(2.0, 3.0).instrument("bass"),
    Block::of("minecraft:warped_planks", BlockClass::Block).strength(2.0, 3.0).instrument("bass"),
    Block::of("minecraft:crimson_slab", BlockClass::SlabBlock).properties(&[&SLAB_TYPE, &WATERLOGGED], &["bottom", "false"]).strength(2.0, 3.0).instrument("bass"),
    Block::of("minecraft:warped_slab", BlockClass::SlabBlock).properties(&[&SLAB_TYPE, &WATERLOGGED], &["bottom", "false"]).strength(2.0, 3.0).instrument("bass"),
    Block::of("minecraft:crimson_pressure_plate", BlockClass::PressurePlateBlock).properties(&[&POWERED], &["false"]).strength(0.5, 0.5).no_collision().force_solid_on().push_reaction(PushReaction::Destroy).instrument("bass"),
    Block::of("minecraft:warped_pressure_plate", BlockClass::PressurePlateBlock).properties(&[&POWERED], &["false"]).strength(0.5, 0.5).no_collision().force_solid_on().push_reaction(PushReaction::Destroy).instrument("bass"),
    Block::of("minecraft:crimson_fence", BlockClass::FenceBlock).properties(&[&EAST, &NORTH, &SOUTH, &WATERLOGGED, &WEST], &["false", "false", "false", "false", "false"]).strength(2.0, 3.0).instrument("bass"),
    Block::of("minecraft:warped_fence", BlockClass::FenceBlock).properties(&[&EAST, &NORTH, &SOUTH, &WATERLOGGED, &WEST], &["false", "false", "false", "false", "false"]).strength(2.0, 3.0).instrument("bass"),
    Block::of("minecraft:crimson_trapdoor", BlockClass::TrapDoorBlock).properties(&[&HORIZONTAL_FACING, &HALF, &OPEN, &POWERED, &WATERLOGGED], &["north", "bottom", "false", "false", "false"]).strength(3.0, 3.0).no_occlusion().instrument("bass"),
    Block::of("minecraft:warped_trapdoor", BlockClass::TrapDoorBlock).properties(&[&HORIZONTAL_FACING, &HALF, &OPEN, &POWERED, &WATERLOGGED], &["north", "bottom", "false", "false", "false"]).strength(3.0, 3.0).no_occlusion().instrument("bass"),
    Block::of("minecraft:crimson_fence_gate", BlockClass::FenceGateBlock).properties(&[&HORIZONTAL_FACING, &IN_WALL, &OPEN, &POWERED], &["north", "false", "false", "false"]).strength(2.0, 3.0).force_solid_on().instrument("bass"),
    Block::of("minecraft:warped_fence_gate", BlockClass::FenceGateBlock).properties(&[&HORIZONTAL_FACING, &IN_WALL, &OPEN, &POWERED], &["north", "false", "false", "false"]).strength(2.0, 3.0).force_solid_on().instrument("bass"),
    Block::of("minecraft:crimson_stairs", BlockClass::StairBlock).properties(&[&HORIZONTAL_FACING, &HALF, &STAIRS_SHAPE, &WATERLOGGED], &["north", "bottom", "straight", "false"]).strength(2.0, 3.0).instrument("bass"),
    Block::of("minecraft:warped_stairs", BlockClass::StairBlock).properties(&[&HORIZONTAL_FACING, &HALF, &STAIRS_SHAPE, &WATERLOGGED], &["north", "bottom", "straight", "false"]).strength(2.0, 3.0).instrument("bass"),
    Block::of("minecraft:crimson_button", BlockClass::ButtonBlock).properties(&[&ATTACH_FACE, &HORIZONTAL_FACING, &POWERED], &["wall", "north", "false"]).strength(0.5, 0.5).no_collision().push_reaction(PushReaction::Destroy),
    Block::of("minecraft:warped_button", BlockClass::ButtonBlock).properties(&[&ATTACH_FACE, &HORIZONTAL_FACING, &POWERED], &["wall", "north", "false"]).strength(0.5, 0.5).no_collision().push_reaction(PushReaction::Destroy),
    Block::of("minecraft:crimson_door", BlockClass::DoorBlock).properties(&[&HORIZONTAL_FACING, &DOUBLE_BLOCK_HALF, &DOOR_HINGE, &OPEN, &POWERED], &["north", "lower", "left", "false", "false"]).strength(3.0, 3.0).no_occlusion().push_reaction(PushReaction::Destroy).instrument("bass"),
    Block::of("minecraft:warped_door", BlockClass::DoorBlock).properties(&[&HORIZONTAL_FACING, &DOUBLE_BLOCK_HALF, &DOOR_HINGE, &OPEN, &POWERED], &["north", "lower", "left", "false", "false"]).strength(3.0, 3.0).no_occlusion().push_reaction(PushReaction::Destroy).instrument("bass"),
    Block::of("minecraft:crimson_sign", BlockClass::StandingSignBlock).properties(&[&ROTATION_16, &WATERLOGGED], &["8", "false"]).strength(1.0, 1.0).no_collision().force_solid_on().instrument("bass"),
    Block::of("minecraft:warped_sign", BlockClass::StandingSignBlock).properties(&[&ROTATION_16, &WATERLOGGED], &["8", "false"]).strength(1.0, 1.0).no_collision().force_solid_on().instrument("bass"),
    Block::of("minecraft:crimson_wall_sign", BlockClass::WallSignBlock).properties(&[&HORIZONTAL_FACING, &WATERLOGGED], &["north", "false"]).strength(1.0, 1.0).no_collision().force_solid_on().instrument("bass").item("minecraft:crimson_sign"),
    Block::of("minecraft:warped_wall_sign", BlockClass::WallSignBlock).properties(&[&HORIZONTAL_FACING, &WATERLOGGED], &["north", "false"]).strength(1.0, 1.0).no_collision().force_solid_on().instrument("bass").item("minecraft:warped_sign"),
    Block::of("minecraft:structure_block", BlockClass::StructureBlock).properties(&[&STRUCTUREBLOCK_MODE], &["load"]).strength(-1.0, 3600000.0).requires_correct_tool_for_drops(),
    Block::of("minecraft:jigsaw", BlockClass::JigsawBlock).properties(&[&ORIENTATION], &["north_up"]).strength(-1.0, 3600000.0).requires_correct_tool_for_drops(),
    Block::of("minecraft:test_block", BlockClass::TestBlock).properties(&[&TEST_BLOCK_MODE], &["start"]).strength(-1.0, 3600000.0),
    Block::of("minecraft:test_instance_block", BlockClass::TestInstanceBlock).strength(-1.0, 3600000.0).no_occlusion().is_view_blocking(Rule::Never),
    Block::of("minecraft:composter", BlockClass::ComposterBlock).properties(&[&LEVEL_COMPOSTER], &["0"]).strength(0.6, 0.6).ignited_by_lava().instrument("bass"),
    Block::of("minecraft:target", BlockClass::TargetBlock).properties(&[&POWER], &["0"]).strength(0.5, 0.5),
    Block::of("minecraft:bee_nest", BlockClass::BeehiveBlock).properties(&[&HORIZONTAL_FACING, &LEVEL_HONEY], &["north", "0"]).strength(0.3, 0.3).ignited_by_lava().instrument("bass"),
    Block::of("minecraft:beehive", BlockClass::BeehiveBlock).properties(&[&HORIZONTAL_FACING, &LEVEL_HONEY], &["north", "0"]).strength(0.6, 0.6).ignited_by_lava().instrument("bass"),
    Block::of("minecraft:honey_block", BlockClass::HoneyBlock).speed_factor(0.4).jump_factor(0.5).no_occlusion(),
    Block::of("minecraft:honeycomb_block", BlockClass::Block).strength(0.6, 0.6),
    Block::of("minecraft:netherite_block", BlockClass::Block).strength(50.0, 1200.0).requires_correct_tool_for_drops(),
    Block::of("minecraft:ancient_debris", BlockClass::Block).strength(30.0, 1200.0).requires_correct_tool_for_drops(),
    Block::of("minecraft:crying_obsidian", BlockClass::CryingObsidianBlock).strength(50.0, 1200.0).requires_correct_tool_for_drops().instrument("basedrum").light_level(Emission::Constant(10)),
    Block::of("minecraft:respawn_anchor", BlockClass::RespawnAnchorBlock).properties(&[&RESPAWN_ANCHOR_CHARGES], &["0"]).strength(50.0, 1200.0).requires_correct_tool_for_drops().instrument("basedrum").light_level(Emission::States(&[0, 3, 7, 11, 15])),
    Block::of("minecraft:potted_crimson_fungus", BlockClass::FlowerPotBlock).no_occlusion().push_reaction(PushReaction::Destroy).item("minecraft:air"),
    Block::of("minecraft:potted_warped_fungus", BlockClass::FlowerPotBlock).no_occlusion().push_reaction(PushReaction::Destroy).item("minecraft:air"),
    Block::of("minecraft:potted_crimson_roots", BlockClass::FlowerPotBlock).no_occlusion().push_reaction(PushReaction::Destroy).item("minecraft:air"),
    Block::of("minecraft:potted_warped_roots", BlockClass::FlowerPotBlock).no_occlusion().push_reaction(PushReaction::Destroy).item("minecraft:air"),
    Block::of("minecraft:lodestone", BlockClass::Block).strength(3.5, 3.5).requires_correct_tool_for_drops().push_reaction(PushReaction::Block),
    Block::of("minecraft:blackstone", BlockClass::Block).strength(1.5, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:blackstone_stairs", BlockClass::StairBlock).properties(&[&HORIZONTAL_FACING, &HALF, &STAIRS_SHAPE, &WATERLOGGED], &["north", "bottom", "straight", "false"]).strength(1.5, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:blackstone_wall", BlockClass::WallBlock).properties(&[&EAST_WALL, &NORTH_WALL, &SOUTH_WALL, &UP, &WATERLOGGED, &WEST_WALL], &["none", "none", "none", "true", "false", "none"]).strength(1.5, 6.0).requires_correct_tool_for_drops().force_solid_on().instrument("basedrum"),
    Block::of("minecraft:blackstone_slab", BlockClass::SlabBlock).properties(&[&SLAB_TYPE, &WATERLOGGED], &["bottom", "false"]).strength(2.0, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:polished_blackstone", BlockClass::Block).strength(2.0, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:polished_blackstone_bricks", BlockClass::Block).strength(1.5, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:cracked_polished_blackstone_bricks", BlockClass::Block).strength(1.5, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:chiseled_polished_blackstone", BlockClass::Block).strength(1.5, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:polished_blackstone_brick_slab", BlockClass::SlabBlock).properties(&[&SLAB_TYPE, &WATERLOGGED], &["bottom", "false"]).strength(2.0, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:polished_blackstone_brick_stairs", BlockClass::StairBlock).properties(&[&HORIZONTAL_FACING, &HALF, &STAIRS_SHAPE, &WATERLOGGED], &["north", "bottom", "straight", "false"]).strength(1.5, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:polished_blackstone_brick_wall", BlockClass::WallBlock).properties(&[&EAST_WALL, &NORTH_WALL, &SOUTH_WALL, &UP, &WATERLOGGED, &WEST_WALL], &["none", "none", "none", "true", "false", "none"]).strength(1.5, 6.0).requires_correct_tool_for_drops().force_solid_on().instrument("basedrum"),
    Block::of("minecraft:gilded_blackstone", BlockClass::Block).strength(1.5, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:polished_blackstone_stairs", BlockClass::StairBlock).properties(&[&HORIZONTAL_FACING, &HALF, &STAIRS_SHAPE, &WATERLOGGED], &["north", "bottom", "straight", "false"]).strength(2.0, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:polished_blackstone_slab", BlockClass::SlabBlock).properties(&[&SLAB_TYPE, &WATERLOGGED], &["bottom", "false"]).strength(2.0, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:polished_blackstone_pressure_plate", BlockClass::PressurePlateBlock).properties(&[&POWERED], &["false"]).strength(0.5, 0.5).no_collision().force_solid_on().push_reaction(PushReaction::Destroy).instrument("basedrum"),
    Block::of("minecraft:polished_blackstone_button", BlockClass::ButtonBlock).properties(&[&ATTACH_FACE, &HORIZONTAL_FACING, &POWERED], &["wall", "north", "false"]).strength(0.5, 0.5).no_collision().push_reaction(PushReaction::Destroy),
    Block::of("minecraft:polished_blackstone_wall", BlockClass::WallBlock).properties(&[&EAST_WALL, &NORTH_WALL, &SOUTH_WALL, &UP, &WATERLOGGED, &WEST_WALL], &["none", "none", "none", "true", "false", "none"]).strength(2.0, 6.0).requires_correct_tool_for_drops().force_solid_on().instrument("basedrum"),
    Block::of("minecraft:chiseled_nether_bricks", BlockClass::Block).strength(2.0, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:cracked_nether_bricks", BlockClass::Block).strength(2.0, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:quartz_bricks", BlockClass::Block).strength(0.8, 0.8).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:candle", BlockClass::CandleBlock).properties(&[&CANDLES, &LIT, &WATERLOGGED], &["1", "false", "false"]).strength(0.1, 0.1).no_occlusion().push_reaction(PushReaction::Destroy).light_level(Emission::States(&[3, 3, 0, 0, 6, 6, 0, 0, 9, 9, 0, 0, 12, 12, 0, 0])),
    Block::of("minecraft:white_candle", BlockClass::CandleBlock).properties(&[&CANDLES, &LIT, &WATERLOGGED], &["1", "false", "false"]).strength(0.1, 0.1).no_occlusion().push_reaction(PushReaction::Destroy).light_level(Emission::States(&[3, 3, 0, 0, 6, 6, 0, 0, 9, 9, 0, 0, 12, 12, 0, 0])),
    Block::of("minecraft:orange_candle", BlockClass::CandleBlock).properties(&[&CANDLES, &LIT, &WATERLOGGED], &["1", "false", "false"]).strength(0.1, 0.1).no_occlusion().push_reaction(PushReaction::Destroy).light_level(Emission::States(&[3, 3, 0, 0, 6, 6, 0, 0, 9, 9, 0, 0, 12, 12, 0, 0])),
    Block::of("minecraft:magenta_candle", BlockClass::CandleBlock).properties(&[&CANDLES, &LIT, &WATERLOGGED], &["1", "false", "false"]).strength(0.1, 0.1).no_occlusion().push_reaction(PushReaction::Destroy).light_level(Emission::States(&[3, 3, 0, 0, 6, 6, 0, 0, 9, 9, 0, 0, 12, 12, 0, 0])),
    Block::of("minecraft:light_blue_candle", BlockClass::CandleBlock).properties(&[&CANDLES, &LIT, &WATERLOGGED], &["1", "false", "false"]).strength(0.1, 0.1).no_occlusion().push_reaction(PushReaction::Destroy).light_level(Emission::States(&[3, 3, 0, 0, 6, 6, 0, 0, 9, 9, 0, 0, 12, 12, 0, 0])),
    Block::of("minecraft:yellow_candle", BlockClass::CandleBlock).properties(&[&CANDLES, &LIT, &WATERLOGGED], &["1", "false", "false"]).strength(0.1, 0.1).no_occlusion().push_reaction(PushReaction::Destroy).light_level(Emission::States(&[3, 3, 0, 0, 6, 6, 0, 0, 9, 9, 0, 0, 12, 12, 0, 0])),
    Block::of("minecraft:lime_candle", BlockClass::CandleBlock).properties(&[&CANDLES, &LIT, &WATERLOGGED], &["1", "false", "false"]).strength(0.1, 0.1).no_occlusion().push_reaction(PushReaction::Destroy).light_level(Emission::States(&[3, 3, 0, 0, 6, 6, 0, 0, 9, 9, 0, 0, 12, 12, 0, 0])),
    Block::of("minecraft:pink_candle", BlockClass::CandleBlock).properties(&[&CANDLES, &LIT, &WATERLOGGED], &["1", "false", "false"]).strength(0.1, 0.1).no_occlusion().push_reaction(PushReaction::Destroy).light_level(Emission::States(&[3, 3, 0, 0, 6, 6, 0, 0, 9, 9, 0, 0, 12, 12, 0, 0])),
    Block::of("minecraft:gray_candle", BlockClass::CandleBlock).properties(&[&CANDLES, &LIT, &WATERLOGGED], &["1", "false", "false"]).strength(0.1, 0.1).no_occlusion().push_reaction(PushReaction::Destroy).light_level(Emission::States(&[3, 3, 0, 0, 6, 6, 0, 0, 9, 9, 0, 0, 12, 12, 0, 0])),
    Block::of("minecraft:light_gray_candle", BlockClass::CandleBlock).properties(&[&CANDLES, &LIT, &WATERLOGGED], &["1", "false", "false"]).strength(0.1, 0.1).no_occlusion().push_reaction(PushReaction::Destroy).light_level(Emission::States(&[3, 3, 0, 0, 6, 6, 0, 0, 9, 9, 0, 0, 12, 12, 0, 0])),
    Block::of("minecraft:cyan_candle", BlockClass::CandleBlock).properties(&[&CANDLES, &LIT, &WATERLOGGED], &["1", "false", "false"]).strength(0.1, 0.1).no_occlusion().push_reaction(PushReaction::Destroy).light_level(Emission::States(&[3, 3, 0, 0, 6, 6, 0, 0, 9, 9, 0, 0, 12, 12, 0, 0])),
    Block::of("minecraft:purple_candle", BlockClass::CandleBlock).properties(&[&CANDLES, &LIT, &WATERLOGGED], &["1", "false", "false"]).strength(0.1, 0.1).no_occlusion().push_reaction(PushReaction::Destroy).light_level(Emission::States(&[3, 3, 0, 0, 6, 6, 0, 0, 9, 9, 0, 0, 12, 12, 0, 0])),
    Block::of("minecraft:blue_candle", BlockClass::CandleBlock).properties(&[&CANDLES, &LIT, &WATERLOGGED], &["1", "false", "false"]).strength(0.1, 0.1).no_occlusion().push_reaction(PushReaction::Destroy).light_level(Emission::States(&[3, 3, 0, 0, 6, 6, 0, 0, 9, 9, 0, 0, 12, 12, 0, 0])),
    Block::of("minecraft:brown_candle", BlockClass::CandleBlock).properties(&[&CANDLES, &LIT, &WATERLOGGED], &["1", "false", "false"]).strength(0.1, 0.1).no_occlusion().push_reaction(PushReaction::Destroy).light_level(Emission::States(&[3, 3, 0, 0, 6, 6, 0, 0, 9, 9, 0, 0, 12, 12, 0, 0])),
    Block::of("minecraft:green_candle", BlockClass::CandleBlock).properties(&[&CANDLES, &LIT, &WATERLOGGED], &["1", "false", "false"]).strength(0.1, 0.1).no_occlusion().push_reaction(PushReaction::Destroy).light_level(Emission::States(&[3, 3, 0, 0, 6, 6, 0, 0, 9, 9, 0, 0, 12, 12, 0, 0])),
    Block::of("minecraft:red_candle", BlockClass::CandleBlock).properties(&[&CANDLES, &LIT, &WATERLOGGED], &["1", "false", "false"]).strength(0.1, 0.1).no_occlusion().push_reaction(PushReaction::Destroy).light_level(Emission::States(&[3, 3, 0, 0, 6, 6, 0, 0, 9, 9, 0, 0, 12, 12, 0, 0])),
    Block::of("minecraft:black_candle", BlockClass::CandleBlock).properties(&[&CANDLES, &LIT, &WATERLOGGED], &["1", "false", "false"]).strength(0.1, 0.1).no_occlusion().push_reaction(PushReaction::Destroy).light_level(Emission::States(&[3, 3, 0, 0, 6, 6, 0, 0, 9, 9, 0, 0, 12, 12, 0, 0])),
    Block::of("minecraft:candle_cake", BlockClass::CandleCakeBlock).properties(&[&LIT], &["false"]).strength(0.5, 0.5).force_solid_on().push_reaction(PushReaction::Destroy).item("minecraft:air").light_level(Emission::States(&[3, 0])),
    Block::of("minecraft:white_candle_cake", BlockClass::CandleCakeBlock).properties(&[&LIT], &["false"]).strength(0.5, 0.5).force_solid_on().push_reaction(PushReaction::Destroy).item("minecraft:air").light_level(Emission::States(&[3, 0])),
    Block::of("minecraft:orange_candle_cake", BlockClass::CandleCakeBlock).properties(&[&LIT], &["false"]).strength(0.5, 0.5).force_solid_on().push_reaction(PushReaction::Destroy).item("minecraft:air").light_level(Emission::States(&[3, 0])),
    Block::of("minecraft:magenta_candle_cake", BlockClass::CandleCakeBlock).properties(&[&LIT], &["false"]).strength(0.5, 0.5).force_solid_on().push_reaction(PushReaction::Destroy).item("minecraft:air").light_level(Emission::States(&[3, 0])),
    Block::of("minecraft:light_blue_candle_cake", BlockClass::CandleCakeBlock).properties(&[&LIT], &["false"]).strength(0.5, 0.5).force_solid_on().push_reaction(PushReaction::Destroy).item("minecraft:air").light_level(Emission::States(&[3, 0])),
    Block::of("minecraft:yellow_candle_cake", BlockClass::CandleCakeBlock).properties(&[&LIT], &["false"]).strength(0.5, 0.5).force_solid_on().push_reaction(PushReaction::Destroy).item("minecraft:air").light_level(Emission::States(&[3, 0])),
    Block::of("minecraft:lime_candle_cake", BlockClass::CandleCakeBlock).properties(&[&LIT], &["false"]).strength(0.5, 0.5).force_solid_on().push_reaction(PushReaction::Destroy).item("minecraft:air").light_level(Emission::States(&[3, 0])),
    Block::of("minecraft:pink_candle_cake", BlockClass::CandleCakeBlock).properties(&[&LIT], &["false"]).strength(0.5, 0.5).force_solid_on().push_reaction(PushReaction::Destroy).item("minecraft:air").light_level(Emission::States(&[3, 0])),
    Block::of("minecraft:gray_candle_cake", BlockClass::CandleCakeBlock).properties(&[&LIT], &["false"]).strength(0.5, 0.5).force_solid_on().push_reaction(PushReaction::Destroy).item("minecraft:air").light_level(Emission::States(&[3, 0])),
    Block::of("minecraft:light_gray_candle_cake", BlockClass::CandleCakeBlock).properties(&[&LIT], &["false"]).strength(0.5, 0.5).force_solid_on().push_reaction(PushReaction::Destroy).item("minecraft:air").light_level(Emission::States(&[3, 0])),
    Block::of("minecraft:cyan_candle_cake", BlockClass::CandleCakeBlock).properties(&[&LIT], &["false"]).strength(0.5, 0.5).force_solid_on().push_reaction(PushReaction::Destroy).item("minecraft:air").light_level(Emission::States(&[3, 0])),
    Block::of("minecraft:purple_candle_cake", BlockClass::CandleCakeBlock).properties(&[&LIT], &["false"]).strength(0.5, 0.5).force_solid_on().push_reaction(PushReaction::Destroy).item("minecraft:air").light_level(Emission::States(&[3, 0])),
    Block::of("minecraft:blue_candle_cake", BlockClass::CandleCakeBlock).properties(&[&LIT], &["false"]).strength(0.5, 0.5).force_solid_on().push_reaction(PushReaction::Destroy).item("minecraft:air").light_level(Emission::States(&[3, 0])),
    Block::of("minecraft:brown_candle_cake", BlockClass::CandleCakeBlock).properties(&[&LIT], &["false"]).strength(0.5, 0.5).force_solid_on().push_reaction(PushReaction::Destroy).item("minecraft:air").light_level(Emission::States(&[3, 0])),
    Block::of("minecraft:green_candle_cake", BlockClass::CandleCakeBlock).properties(&[&LIT], &["false"]).strength(0.5, 0.5).force_solid_on().push_reaction(PushReaction::Destroy).item("minecraft:air").light_level(Emission::States(&[3, 0])),
    Block::of("minecraft:red_candle_cake", BlockClass::CandleCakeBlock).properties(&[&LIT], &["false"]).strength(0.5, 0.5).force_solid_on().push_reaction(PushReaction::Destroy).item("minecraft:air").light_level(Emission::States(&[3, 0])),
    Block::of("minecraft:black_candle_cake", BlockClass::CandleCakeBlock).properties(&[&LIT], &["false"]).strength(0.5, 0.5).force_solid_on().push_reaction(PushReaction::Destroy).item("minecraft:air").light_level(Emission::States(&[3, 0])),
    Block::of("minecraft:amethyst_block", BlockClass::AmethystBlock).strength(1.5, 1.5).requires_correct_tool_for_drops(),
    Block::of("minecraft:budding_amethyst", BlockClass::BuddingAmethystBlock).strength(1.5, 1.5).requires_correct_tool_for_drops().random_ticks().push_reaction(PushReaction::Destroy),
    Block::of("minecraft:amethyst_cluster", BlockClass::AmethystClusterBlock).properties(&[&FACING, &WATERLOGGED], &["up", "false"]).strength(1.5, 1.5).no_occlusion().force_solid_on().push_reaction(PushReaction::Destroy).light_level(Emission::Constant(5)),
    Block::of("minecraft:large_amethyst_bud", BlockClass::AmethystClusterBlock).properties(&[&FACING, &WATERLOGGED], &["up", "false"]).strength(1.5, 1.5).no_occlusion().force_solid_on().push_reaction(PushReaction::Destroy).light_level(Emission::Constant(4)),
    Block::of("minecraft:medium_amethyst_bud", BlockClass::AmethystClusterBlock).properties(&[&FACING, &WATERLOGGED], &["up", "false"]).strength(1.5, 1.5).no_occlusion().force_solid_on().push_reaction(PushReaction::Destroy).light_level(Emission::Constant(2)),
    Block::of("minecraft:small_amethyst_bud", BlockClass::AmethystClusterBlock).properties(&[&FACING, &WATERLOGGED], &["up", "false"]).strength(1.5, 1.5).no_occlusion().force_solid_on().push_reaction(PushReaction::Destroy).light_level(Emission::Constant(1)),
    Block::of("minecraft:tuff", BlockClass::Block).strength(1.5, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:tuff_slab", BlockClass::SlabBlock).properties(&[&SLAB_TYPE, &WATERLOGGED], &["bottom", "false"]).strength(1.5, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:tuff_stairs", BlockClass::StairBlock).properties(&[&HORIZONTAL_FACING, &HALF, &STAIRS_SHAPE, &WATERLOGGED], &["north", "bottom", "straight", "false"]).strength(1.5, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:tuff_wall", BlockClass::WallBlock).properties(&[&EAST_WALL, &NORTH_WALL, &SOUTH_WALL, &UP, &WATERLOGGED, &WEST_WALL], &["none", "none", "none", "true", "false", "none"]).strength(1.5, 6.0).requires_correct_tool_for_drops().force_solid_on().instrument("basedrum"),
    Block::of("minecraft:polished_tuff", BlockClass::Block).strength(1.5, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:polished_tuff_slab", BlockClass::SlabBlock).properties(&[&SLAB_TYPE, &WATERLOGGED], &["bottom", "false"]).strength(1.5, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:polished_tuff_stairs", BlockClass::StairBlock).properties(&[&HORIZONTAL_FACING, &HALF, &STAIRS_SHAPE, &WATERLOGGED], &["north", "bottom", "straight", "false"]).strength(1.5, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:polished_tuff_wall", BlockClass::WallBlock).properties(&[&EAST_WALL, &NORTH_WALL, &SOUTH_WALL, &UP, &WATERLOGGED, &WEST_WALL], &["none", "none", "none", "true", "false", "none"]).strength(1.5, 6.0).requires_correct_tool_for_drops().force_solid_on().instrument("basedrum"),
    Block::of("minecraft:chiseled_tuff", BlockClass::Block).strength(1.5, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:tuff_bricks", BlockClass::Block).strength(1.5, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:tuff_brick_slab", BlockClass::SlabBlock).properties(&[&SLAB_TYPE, &WATERLOGGED], &["bottom", "false"]).strength(1.5, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:tuff_brick_stairs", BlockClass::StairBlock).properties(&[&HORIZONTAL_FACING, &HALF, &STAIRS_SHAPE, &WATERLOGGED], &["north", "bottom", "straight", "false"]).strength(1.5, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:tuff_brick_wall", BlockClass::WallBlock).properties(&[&EAST_WALL, &NORTH_WALL, &SOUTH_WALL, &UP, &WATERLOGGED, &WEST_WALL], &["none", "none", "none", "true", "false", "none"]).strength(1.5, 6.0).requires_correct_tool_for_drops().force_solid_on().instrument("basedrum"),
    Block::of("minecraft:chiseled_tuff_bricks", BlockClass::Block).strength(1.5, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:sulfur", BlockClass::Block).strength(1.5, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:potent_sulfur", BlockClass::PotentSulfurBlock).properties(&[&POTENT_SULFUR_STATE], &["dry"]).strength(1.5, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:sulfur_slab", BlockClass::SlabBlock).properties(&[&SLAB_TYPE, &WATERLOGGED], &["bottom", "false"]).strength(1.5, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:sulfur_stairs", BlockClass::StairBlock).properties(&[&HORIZONTAL_FACING, &HALF, &STAIRS_SHAPE, &WATERLOGGED], &["north", "bottom", "straight", "false"]).strength(1.5, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:sulfur_wall", BlockClass::WallBlock).properties(&[&EAST_WALL, &NORTH_WALL, &SOUTH_WALL, &UP, &WATERLOGGED, &WEST_WALL], &["none", "none", "none", "true", "false", "none"]).strength(1.5, 6.0).requires_correct_tool_for_drops().force_solid_on().instrument("basedrum"),
    Block::of("minecraft:polished_sulfur", BlockClass::Block).strength(1.5, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:polished_sulfur_slab", BlockClass::SlabBlock).properties(&[&SLAB_TYPE, &WATERLOGGED], &["bottom", "false"]).strength(1.5, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:polished_sulfur_stairs", BlockClass::StairBlock).properties(&[&HORIZONTAL_FACING, &HALF, &STAIRS_SHAPE, &WATERLOGGED], &["north", "bottom", "straight", "false"]).strength(1.5, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:polished_sulfur_wall", BlockClass::WallBlock).properties(&[&EAST_WALL, &NORTH_WALL, &SOUTH_WALL, &UP, &WATERLOGGED, &WEST_WALL], &["none", "none", "none", "true", "false", "none"]).strength(1.5, 6.0).requires_correct_tool_for_drops().force_solid_on().instrument("basedrum"),
    Block::of("minecraft:sulfur_bricks", BlockClass::Block).strength(1.5, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:sulfur_brick_slab", BlockClass::SlabBlock).properties(&[&SLAB_TYPE, &WATERLOGGED], &["bottom", "false"]).strength(1.5, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:sulfur_brick_stairs", BlockClass::StairBlock).properties(&[&HORIZONTAL_FACING, &HALF, &STAIRS_SHAPE, &WATERLOGGED], &["north", "bottom", "straight", "false"]).strength(1.5, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:sulfur_brick_wall", BlockClass::WallBlock).properties(&[&EAST_WALL, &NORTH_WALL, &SOUTH_WALL, &UP, &WATERLOGGED, &WEST_WALL], &["none", "none", "none", "true", "false", "none"]).strength(1.5, 6.0).requires_correct_tool_for_drops().force_solid_on().instrument("basedrum"),
    Block::of("minecraft:chiseled_sulfur", BlockClass::Block).strength(1.5, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:cinnabar", BlockClass::Block).strength(1.5, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:cinnabar_slab", BlockClass::SlabBlock).properties(&[&SLAB_TYPE, &WATERLOGGED], &["bottom", "false"]).strength(1.5, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:cinnabar_stairs", BlockClass::StairBlock).properties(&[&HORIZONTAL_FACING, &HALF, &STAIRS_SHAPE, &WATERLOGGED], &["north", "bottom", "straight", "false"]).strength(1.5, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:cinnabar_wall", BlockClass::WallBlock).properties(&[&EAST_WALL, &NORTH_WALL, &SOUTH_WALL, &UP, &WATERLOGGED, &WEST_WALL], &["none", "none", "none", "true", "false", "none"]).strength(1.5, 6.0).requires_correct_tool_for_drops().force_solid_on().instrument("basedrum"),
    Block::of("minecraft:polished_cinnabar", BlockClass::Block).strength(1.5, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:polished_cinnabar_slab", BlockClass::SlabBlock).properties(&[&SLAB_TYPE, &WATERLOGGED], &["bottom", "false"]).strength(1.5, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:polished_cinnabar_stairs", BlockClass::StairBlock).properties(&[&HORIZONTAL_FACING, &HALF, &STAIRS_SHAPE, &WATERLOGGED], &["north", "bottom", "straight", "false"]).strength(1.5, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:polished_cinnabar_wall", BlockClass::WallBlock).properties(&[&EAST_WALL, &NORTH_WALL, &SOUTH_WALL, &UP, &WATERLOGGED, &WEST_WALL], &["none", "none", "none", "true", "false", "none"]).strength(1.5, 6.0).requires_correct_tool_for_drops().force_solid_on().instrument("basedrum"),
    Block::of("minecraft:cinnabar_bricks", BlockClass::Block).strength(1.5, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:cinnabar_brick_slab", BlockClass::SlabBlock).properties(&[&SLAB_TYPE, &WATERLOGGED], &["bottom", "false"]).strength(1.5, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:cinnabar_brick_stairs", BlockClass::StairBlock).properties(&[&HORIZONTAL_FACING, &HALF, &STAIRS_SHAPE, &WATERLOGGED], &["north", "bottom", "straight", "false"]).strength(1.5, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:cinnabar_brick_wall", BlockClass::WallBlock).properties(&[&EAST_WALL, &NORTH_WALL, &SOUTH_WALL, &UP, &WATERLOGGED, &WEST_WALL], &["none", "none", "none", "true", "false", "none"]).strength(1.5, 6.0).requires_correct_tool_for_drops().force_solid_on().instrument("basedrum"),
    Block::of("minecraft:chiseled_cinnabar", BlockClass::Block).strength(1.5, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:calcite", BlockClass::Block).strength(0.75, 0.75).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:tinted_glass", BlockClass::TintedGlassBlock).strength(0.3, 0.3).no_occlusion().instrument("hat").is_suffocating(Rule::Never).is_view_blocking(Rule::Never).is_redstone_conductor(Rule::Never),
    Block::of("minecraft:powder_snow", BlockClass::PowderSnowBlock).strength(0.25, 0.25).no_occlusion().dynamic_shape().item("minecraft:powder_snow_bucket").is_redstone_conductor(Rule::Never),
    Block::of("minecraft:sculk_sensor", BlockClass::SculkSensorBlock).properties(&[&POWER, &SCULK_SENSOR_PHASE, &WATERLOGGED], &["0", "inactive", "false"]).strength(1.5, 1.5).light_level(Emission::Constant(1)).emissive_rendering(Rule::States(&[false, false, true, true, false, false, false, false, true, true, false, false, false, false, true, true, false, false, false, false, true, true, false, false, false, false, true, true, false, false, false, false, true, true, false, false, false, false, true, true, false, false, false, false, true, true, false, false, false, false, true, true, false, false, false, false, true, true, false, false, false, false, true, true, false, false, false, false, true, true, false, false, false, false, true, true, false, false, false, false, true, true, false, false, false, false, true, true, false, false, false, false, true, true, false, false])),
    Block::of("minecraft:calibrated_sculk_sensor", BlockClass::CalibratedSculkSensorBlock).properties(&[&HORIZONTAL_FACING, &POWER, &SCULK_SENSOR_PHASE, &WATERLOGGED], &["north", "0", "inactive", "false"]).strength(1.5, 1.5).light_level(Emission::Constant(1)).emissive_rendering(Rule::States(&[false, false, true, true, false, false, false, false, true, true, false, false, false, false, true, true, false, false, false, false, true, true, false, false, false, false, true, true, false, false, false, false, true, true, false, false, false, false, true, true, false, false, false, false, true, true, false, false, false, false, true, true, false, false, false, false, true, true, false, false, false, false, true, true, false, false, false, false, true, true, false, false, false, false, true, true, false, false, false, false, true, true, false, false, false, false, true, true, false, false, false, false, true, true, false, false, false, false, true, true, false, false, false, false, true, true, false, false, false, false, true, true, false, false, false, false, true, true, false, false, false, false, true, true, false, false, false, false, true, true, false, false, false, false, true, true, false, false, false, false, true, true, false, false, false, false, true, true, false, false, false, false, true, true, false, false, false, false, true, true, false, false, false, false, true, true, false, false, false, false, true, true, false, false, false, false, true, true, false, false, false, false, true, true, false, false, false, false, true, true, false, false, false, false, true, true, false, false, false, false, true, true, false, false, false, false, true, true, false, false, false, false, true, true, false, false, false, false, true, true, false, false, false, false, true, true, false, false, false, false, true, true, false, false, false, false, true, true, false, false, false, false, true, true, false, false, false, false, true, true, false, false, false, false, true, true, false, false, false, false, true, true, false, false, false, false, true, true, false, false, false, false, true, true, false, false, false, false, true, true, false, false, false, false, true, true, false, false, false, false, true, true, false, false, false, false, true, true, false, false, false, false, true, true, false, false, false, false, true, true, false, false, false, false, true, true, false, false, false, false, true, true, false, false, false, false, true, true, false, false, false, false, true, true, false, false, false, false, true, true, false, false, false, false, true, true, false, false, false, false, true, true, false, false, false, false, true, true, false, false, false, false, true, true, false, false, false, false, true, true, false, false, false, false, true, true, false, false, false, false, true, true, false, false])),
    Block::of("minecraft:sculk", BlockClass::SculkBlock).strength(0.2, 0.2),
    Block::of("minecraft:sculk_vein", BlockClass::SculkVeinBlock).properties(&[&DOWN, &EAST, &NORTH, &SOUTH, &UP, &WATERLOGGED, &WEST], &["false", "false", "false", "false", "false", "false", "false"]).strength(0.2, 0.2).no_collision().force_solid_on().push_reaction(PushReaction::Destroy),
    Block::of("minecraft:sculk_catalyst", BlockClass::SculkCatalystBlock).properties(&[&BLOOM], &["false"]).strength(3.0, 3.0).light_level(Emission::Constant(6)),
    Block::of("minecraft:sculk_shrieker", BlockClass::SculkShriekerBlock).properties(&[&CAN_SUMMON, &SHRIEKING, &WATERLOGGED], &["false", "false", "false"]).strength(3.0, 3.0),
    Block::of("minecraft:copper_block", BlockClass::WeatheringCopperFullBlock).strength(3.0, 6.0).requires_correct_tool_for_drops().instrument("trumpet"),
    Block::of("minecraft:exposed_copper", BlockClass::WeatheringCopperFullBlock).strength(3.0, 6.0).requires_correct_tool_for_drops().instrument("trumpet_exposed"),
    Block::of("minecraft:weathered_copper", BlockClass::WeatheringCopperFullBlock).strength(3.0, 6.0).requires_correct_tool_for_drops().instrument("trumpet_weathered"),
    Block::of("minecraft:oxidized_copper", BlockClass::WeatheringCopperFullBlock).strength(3.0, 6.0).requires_correct_tool_for_drops().instrument("trumpet_oxidized"),
    Block::of("minecraft:waxed_copper_block", BlockClass::Block).strength(3.0, 6.0).requires_correct_tool_for_drops().instrument("trumpet"),
    Block::of("minecraft:waxed_exposed_copper", BlockClass::Block).strength(3.0, 6.0).requires_correct_tool_for_drops().instrument("trumpet_exposed"),
    Block::of("minecraft:waxed_weathered_copper", BlockClass::Block).strength(3.0, 6.0).requires_correct_tool_for_drops().instrument("trumpet_weathered"),
    Block::of("minecraft:waxed_oxidized_copper", BlockClass::Block).strength(3.0, 6.0).requires_correct_tool_for_drops().instrument("trumpet_oxidized"),
    Block::of("minecraft:copper_ore", BlockClass::DropExperienceBlock).strength(3.0, 3.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:deepslate_copper_ore", BlockClass::DropExperienceBlock).strength(4.5, 3.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:cut_copper", BlockClass::WeatheringCopperFullBlock).strength(3.0, 6.0).requires_correct_tool_for_drops().instrument("trumpet"),
    Block::of("minecraft:exposed_cut_copper", BlockClass::WeatheringCopperFullBlock).strength(3.0, 6.0).requires_correct_tool_for_drops().instrument("trumpet_exposed"),
    Block::of("minecraft:weathered_cut_copper", BlockClass::WeatheringCopperFullBlock).strength(3.0, 6.0).requires_correct_tool_for_drops().instrument("trumpet_weathered"),
    Block::of("minecraft:oxidized_cut_copper", BlockClass::WeatheringCopperFullBlock).strength(3.0, 6.0).requires_correct_tool_for_drops().instrument("trumpet_oxidized"),
    Block::of("minecraft:waxed_cut_copper", BlockClass::Block).strength(3.0, 6.0).requires_correct_tool_for_drops().instrument("trumpet"),
    Block::of("minecraft:waxed_exposed_cut_copper", BlockClass::Block).strength(3.0, 6.0).requires_correct_tool_for_drops().instrument("trumpet_exposed"),
    Block::of("minecraft:waxed_weathered_cut_copper", BlockClass::Block).strength(3.0, 6.0).requires_correct_tool_for_drops().instrument("trumpet_weathered"),
    Block::of("minecraft:waxed_oxidized_cut_copper", BlockClass::Block).strength(3.0, 6.0).requires_correct_tool_for_drops().instrument("trumpet_oxidized"),
    Block::of("minecraft:chiseled_copper", BlockClass::WeatheringCopperFullBlock).strength(3.0, 6.0).requires_correct_tool_for_drops().instrument("trumpet"),
    Block::of("minecraft:exposed_chiseled_copper", BlockClass::WeatheringCopperFullBlock).strength(3.0, 6.0).requires_correct_tool_for_drops().instrument("trumpet_exposed"),
    Block::of("minecraft:weathered_chiseled_copper", BlockClass::WeatheringCopperFullBlock).strength(3.0, 6.0).requires_correct_tool_for_drops().instrument("trumpet_weathered"),
    Block::of("minecraft:oxidized_chiseled_copper", BlockClass::WeatheringCopperFullBlock).strength(3.0, 6.0).requires_correct_tool_for_drops().instrument("trumpet_oxidized"),
    Block::of("minecraft:waxed_chiseled_copper", BlockClass::Block).strength(3.0, 6.0).requires_correct_tool_for_drops().instrument("trumpet"),
    Block::of("minecraft:waxed_exposed_chiseled_copper", BlockClass::Block).strength(3.0, 6.0).requires_correct_tool_for_drops().instrument("trumpet_exposed"),
    Block::of("minecraft:waxed_weathered_chiseled_copper", BlockClass::Block).strength(3.0, 6.0).requires_correct_tool_for_drops().instrument("trumpet_weathered"),
    Block::of("minecraft:waxed_oxidized_chiseled_copper", BlockClass::Block).strength(3.0, 6.0).requires_correct_tool_for_drops().instrument("trumpet_oxidized"),
    Block::of("minecraft:cut_copper_stairs", BlockClass::WeatheringCopperStairBlock).properties(&[&HORIZONTAL_FACING, &HALF, &STAIRS_SHAPE, &WATERLOGGED], &["north", "bottom", "straight", "false"]).strength(3.0, 6.0).requires_correct_tool_for_drops().instrument("trumpet"),
    Block::of("minecraft:exposed_cut_copper_stairs", BlockClass::WeatheringCopperStairBlock).properties(&[&HORIZONTAL_FACING, &HALF, &STAIRS_SHAPE, &WATERLOGGED], &["north", "bottom", "straight", "false"]).strength(3.0, 6.0).requires_correct_tool_for_drops().instrument("trumpet_exposed"),
    Block::of("minecraft:weathered_cut_copper_stairs", BlockClass::WeatheringCopperStairBlock).properties(&[&HORIZONTAL_FACING, &HALF, &STAIRS_SHAPE, &WATERLOGGED], &["north", "bottom", "straight", "false"]).strength(3.0, 6.0).requires_correct_tool_for_drops().instrument("trumpet_weathered"),
    Block::of("minecraft:oxidized_cut_copper_stairs", BlockClass::WeatheringCopperStairBlock).properties(&[&HORIZONTAL_FACING, &HALF, &STAIRS_SHAPE, &WATERLOGGED], &["north", "bottom", "straight", "false"]).strength(3.0, 6.0).requires_correct_tool_for_drops().instrument("trumpet_oxidized"),
    Block::of("minecraft:waxed_cut_copper_stairs", BlockClass::StairBlock).properties(&[&HORIZONTAL_FACING, &HALF, &STAIRS_SHAPE, &WATERLOGGED], &["north", "bottom", "straight", "false"]).strength(3.0, 6.0).requires_correct_tool_for_drops().instrument("trumpet"),
    Block::of("minecraft:waxed_exposed_cut_copper_stairs", BlockClass::StairBlock).properties(&[&HORIZONTAL_FACING, &HALF, &STAIRS_SHAPE, &WATERLOGGED], &["north", "bottom", "straight", "false"]).strength(3.0, 6.0).requires_correct_tool_for_drops().instrument("trumpet_exposed"),
    Block::of("minecraft:waxed_weathered_cut_copper_stairs", BlockClass::StairBlock).properties(&[&HORIZONTAL_FACING, &HALF, &STAIRS_SHAPE, &WATERLOGGED], &["north", "bottom", "straight", "false"]).strength(3.0, 6.0).requires_correct_tool_for_drops().instrument("trumpet_weathered"),
    Block::of("minecraft:waxed_oxidized_cut_copper_stairs", BlockClass::StairBlock).properties(&[&HORIZONTAL_FACING, &HALF, &STAIRS_SHAPE, &WATERLOGGED], &["north", "bottom", "straight", "false"]).strength(3.0, 6.0).requires_correct_tool_for_drops().instrument("trumpet_oxidized"),
    Block::of("minecraft:cut_copper_slab", BlockClass::WeatheringCopperSlabBlock).properties(&[&SLAB_TYPE, &WATERLOGGED], &["bottom", "false"]).strength(3.0, 6.0).requires_correct_tool_for_drops().instrument("trumpet"),
    Block::of("minecraft:exposed_cut_copper_slab", BlockClass::WeatheringCopperSlabBlock).properties(&[&SLAB_TYPE, &WATERLOGGED], &["bottom", "false"]).strength(3.0, 6.0).requires_correct_tool_for_drops().instrument("trumpet_exposed"),
    Block::of("minecraft:weathered_cut_copper_slab", BlockClass::WeatheringCopperSlabBlock).properties(&[&SLAB_TYPE, &WATERLOGGED], &["bottom", "false"]).strength(3.0, 6.0).requires_correct_tool_for_drops().instrument("trumpet_weathered"),
    Block::of("minecraft:oxidized_cut_copper_slab", BlockClass::WeatheringCopperSlabBlock).properties(&[&SLAB_TYPE, &WATERLOGGED], &["bottom", "false"]).strength(3.0, 6.0).requires_correct_tool_for_drops().instrument("trumpet_oxidized"),
    Block::of("minecraft:waxed_cut_copper_slab", BlockClass::SlabBlock).properties(&[&SLAB_TYPE, &WATERLOGGED], &["bottom", "false"]).strength(3.0, 6.0).requires_correct_tool_for_drops().instrument("trumpet"),
    Block::of("minecraft:waxed_exposed_cut_copper_slab", BlockClass::SlabBlock).properties(&[&SLAB_TYPE, &WATERLOGGED], &["bottom", "false"]).strength(3.0, 6.0).requires_correct_tool_for_drops().instrument("trumpet_exposed"),
    Block::of("minecraft:waxed_weathered_cut_copper_slab", BlockClass::SlabBlock).properties(&[&SLAB_TYPE, &WATERLOGGED], &["bottom", "false"]).strength(3.0, 6.0).requires_correct_tool_for_drops().instrument("trumpet_weathered"),
    Block::of("minecraft:waxed_oxidized_cut_copper_slab", BlockClass::SlabBlock).properties(&[&SLAB_TYPE, &WATERLOGGED], &["bottom", "false"]).strength(3.0, 6.0).requires_correct_tool_for_drops().instrument("trumpet_oxidized"),
    Block::of("minecraft:copper_door", BlockClass::WeatheringCopperDoorBlock).properties(&[&HORIZONTAL_FACING, &DOUBLE_BLOCK_HALF, &DOOR_HINGE, &OPEN, &POWERED], &["north", "lower", "left", "false", "false"]).strength(3.0, 6.0).no_occlusion().push_reaction(PushReaction::Destroy),
    Block::of("minecraft:exposed_copper_door", BlockClass::WeatheringCopperDoorBlock).properties(&[&HORIZONTAL_FACING, &DOUBLE_BLOCK_HALF, &DOOR_HINGE, &OPEN, &POWERED], &["north", "lower", "left", "false", "false"]).strength(3.0, 6.0).no_occlusion().push_reaction(PushReaction::Destroy),
    Block::of("minecraft:weathered_copper_door", BlockClass::WeatheringCopperDoorBlock).properties(&[&HORIZONTAL_FACING, &DOUBLE_BLOCK_HALF, &DOOR_HINGE, &OPEN, &POWERED], &["north", "lower", "left", "false", "false"]).strength(3.0, 6.0).no_occlusion().push_reaction(PushReaction::Destroy),
    Block::of("minecraft:oxidized_copper_door", BlockClass::WeatheringCopperDoorBlock).properties(&[&HORIZONTAL_FACING, &DOUBLE_BLOCK_HALF, &DOOR_HINGE, &OPEN, &POWERED], &["north", "lower", "left", "false", "false"]).strength(3.0, 6.0).no_occlusion().push_reaction(PushReaction::Destroy),
    Block::of("minecraft:waxed_copper_door", BlockClass::DoorBlock).properties(&[&HORIZONTAL_FACING, &DOUBLE_BLOCK_HALF, &DOOR_HINGE, &OPEN, &POWERED], &["north", "lower", "left", "false", "false"]).strength(3.0, 6.0).no_occlusion().push_reaction(PushReaction::Destroy),
    Block::of("minecraft:waxed_exposed_copper_door", BlockClass::DoorBlock).properties(&[&HORIZONTAL_FACING, &DOUBLE_BLOCK_HALF, &DOOR_HINGE, &OPEN, &POWERED], &["north", "lower", "left", "false", "false"]).strength(3.0, 6.0).no_occlusion().push_reaction(PushReaction::Destroy),
    Block::of("minecraft:waxed_weathered_copper_door", BlockClass::DoorBlock).properties(&[&HORIZONTAL_FACING, &DOUBLE_BLOCK_HALF, &DOOR_HINGE, &OPEN, &POWERED], &["north", "lower", "left", "false", "false"]).strength(3.0, 6.0).no_occlusion().push_reaction(PushReaction::Destroy),
    Block::of("minecraft:waxed_oxidized_copper_door", BlockClass::DoorBlock).properties(&[&HORIZONTAL_FACING, &DOUBLE_BLOCK_HALF, &DOOR_HINGE, &OPEN, &POWERED], &["north", "lower", "left", "false", "false"]).strength(3.0, 6.0).no_occlusion().push_reaction(PushReaction::Destroy),
    Block::of("minecraft:copper_trapdoor", BlockClass::WeatheringCopperTrapDoorBlock).properties(&[&HORIZONTAL_FACING, &HALF, &OPEN, &POWERED, &WATERLOGGED], &["north", "bottom", "false", "false", "false"]).strength(3.0, 6.0).no_occlusion().requires_correct_tool_for_drops(),
    Block::of("minecraft:exposed_copper_trapdoor", BlockClass::WeatheringCopperTrapDoorBlock).properties(&[&HORIZONTAL_FACING, &HALF, &OPEN, &POWERED, &WATERLOGGED], &["north", "bottom", "false", "false", "false"]).strength(3.0, 6.0).no_occlusion().requires_correct_tool_for_drops(),
    Block::of("minecraft:weathered_copper_trapdoor", BlockClass::WeatheringCopperTrapDoorBlock).properties(&[&HORIZONTAL_FACING, &HALF, &OPEN, &POWERED, &WATERLOGGED], &["north", "bottom", "false", "false", "false"]).strength(3.0, 6.0).no_occlusion().requires_correct_tool_for_drops(),
    Block::of("minecraft:oxidized_copper_trapdoor", BlockClass::WeatheringCopperTrapDoorBlock).properties(&[&HORIZONTAL_FACING, &HALF, &OPEN, &POWERED, &WATERLOGGED], &["north", "bottom", "false", "false", "false"]).strength(3.0, 6.0).no_occlusion().requires_correct_tool_for_drops(),
    Block::of("minecraft:waxed_copper_trapdoor", BlockClass::TrapDoorBlock).properties(&[&HORIZONTAL_FACING, &HALF, &OPEN, &POWERED, &WATERLOGGED], &["north", "bottom", "false", "false", "false"]).strength(3.0, 6.0).no_occlusion().requires_correct_tool_for_drops(),
    Block::of("minecraft:waxed_exposed_copper_trapdoor", BlockClass::TrapDoorBlock).properties(&[&HORIZONTAL_FACING, &HALF, &OPEN, &POWERED, &WATERLOGGED], &["north", "bottom", "false", "false", "false"]).strength(3.0, 6.0).no_occlusion().requires_correct_tool_for_drops(),
    Block::of("minecraft:waxed_weathered_copper_trapdoor", BlockClass::TrapDoorBlock).properties(&[&HORIZONTAL_FACING, &HALF, &OPEN, &POWERED, &WATERLOGGED], &["north", "bottom", "false", "false", "false"]).strength(3.0, 6.0).no_occlusion().requires_correct_tool_for_drops(),
    Block::of("minecraft:waxed_oxidized_copper_trapdoor", BlockClass::TrapDoorBlock).properties(&[&HORIZONTAL_FACING, &HALF, &OPEN, &POWERED, &WATERLOGGED], &["north", "bottom", "false", "false", "false"]).strength(3.0, 6.0).no_occlusion().requires_correct_tool_for_drops(),
    Block::of("minecraft:copper_grate", BlockClass::WeatheringCopperGrateBlock).properties(&[&WATERLOGGED], &["false"]).strength(3.0, 6.0).no_occlusion().requires_correct_tool_for_drops().is_suffocating(Rule::Never).is_view_blocking(Rule::Never).is_redstone_conductor(Rule::Never),
    Block::of("minecraft:exposed_copper_grate", BlockClass::WeatheringCopperGrateBlock).properties(&[&WATERLOGGED], &["false"]).strength(3.0, 6.0).no_occlusion().requires_correct_tool_for_drops().is_suffocating(Rule::Never).is_view_blocking(Rule::Never).is_redstone_conductor(Rule::Never),
    Block::of("minecraft:weathered_copper_grate", BlockClass::WeatheringCopperGrateBlock).properties(&[&WATERLOGGED], &["false"]).strength(3.0, 6.0).no_occlusion().requires_correct_tool_for_drops().is_suffocating(Rule::Never).is_view_blocking(Rule::Never).is_redstone_conductor(Rule::Never),
    Block::of("minecraft:oxidized_copper_grate", BlockClass::WeatheringCopperGrateBlock).properties(&[&WATERLOGGED], &["false"]).strength(3.0, 6.0).no_occlusion().requires_correct_tool_for_drops().is_suffocating(Rule::Never).is_view_blocking(Rule::Never).is_redstone_conductor(Rule::Never),
    Block::of("minecraft:waxed_copper_grate", BlockClass::WaterloggedTransparentBlock).properties(&[&WATERLOGGED], &["false"]).strength(3.0, 6.0).no_occlusion().requires_correct_tool_for_drops().is_suffocating(Rule::Never).is_view_blocking(Rule::Never).is_redstone_conductor(Rule::Never),
    Block::of("minecraft:waxed_exposed_copper_grate", BlockClass::WaterloggedTransparentBlock).properties(&[&WATERLOGGED], &["false"]).strength(3.0, 6.0).no_occlusion().requires_correct_tool_for_drops().is_suffocating(Rule::Never).is_view_blocking(Rule::Never).is_redstone_conductor(Rule::Never),
    Block::of("minecraft:waxed_weathered_copper_grate", BlockClass::WaterloggedTransparentBlock).properties(&[&WATERLOGGED], &["false"]).strength(3.0, 6.0).no_occlusion().requires_correct_tool_for_drops().is_suffocating(Rule::Never).is_view_blocking(Rule::Never).is_redstone_conductor(Rule::Never),
    Block::of("minecraft:waxed_oxidized_copper_grate", BlockClass::WaterloggedTransparentBlock).properties(&[&WATERLOGGED], &["false"]).strength(3.0, 6.0).no_occlusion().requires_correct_tool_for_drops().is_suffocating(Rule::Never).is_view_blocking(Rule::Never).is_redstone_conductor(Rule::Never),
    Block::of("minecraft:copper_bulb", BlockClass::WeatheringCopperBulbBlock).properties(&[&LIT, &POWERED], &["false", "false"]).strength(3.0, 6.0).requires_correct_tool_for_drops().light_level(Emission::States(&[15, 15, 0, 0])).is_redstone_conductor(Rule::Never),
    Block::of("minecraft:exposed_copper_bulb", BlockClass::WeatheringCopperBulbBlock).properties(&[&LIT, &POWERED], &["false", "false"]).strength(3.0, 6.0).requires_correct_tool_for_drops().light_level(Emission::States(&[12, 12, 0, 0])).is_redstone_conductor(Rule::Never),
    Block::of("minecraft:weathered_copper_bulb", BlockClass::WeatheringCopperBulbBlock).properties(&[&LIT, &POWERED], &["false", "false"]).strength(3.0, 6.0).requires_correct_tool_for_drops().light_level(Emission::States(&[8, 8, 0, 0])).is_redstone_conductor(Rule::Never),
    Block::of("minecraft:oxidized_copper_bulb", BlockClass::WeatheringCopperBulbBlock).properties(&[&LIT, &POWERED], &["false", "false"]).strength(3.0, 6.0).requires_correct_tool_for_drops().light_level(Emission::States(&[4, 4, 0, 0])).is_redstone_conductor(Rule::Never),
    Block::of("minecraft:waxed_copper_bulb", BlockClass::CopperBulbBlock).properties(&[&LIT, &POWERED], &["false", "false"]).strength(3.0, 6.0).requires_correct_tool_for_drops().light_level(Emission::States(&[15, 15, 0, 0])).is_redstone_conductor(Rule::Never),
    Block::of("minecraft:waxed_exposed_copper_bulb", BlockClass::CopperBulbBlock).properties(&[&LIT, &POWERED], &["false", "false"]).strength(3.0, 6.0).requires_correct_tool_for_drops().light_level(Emission::States(&[12, 12, 0, 0])).is_redstone_conductor(Rule::Never),
    Block::of("minecraft:waxed_weathered_copper_bulb", BlockClass::CopperBulbBlock).properties(&[&LIT, &POWERED], &["false", "false"]).strength(3.0, 6.0).requires_correct_tool_for_drops().light_level(Emission::States(&[8, 8, 0, 0])).is_redstone_conductor(Rule::Never),
    Block::of("minecraft:waxed_oxidized_copper_bulb", BlockClass::CopperBulbBlock).properties(&[&LIT, &POWERED], &["false", "false"]).strength(3.0, 6.0).requires_correct_tool_for_drops().light_level(Emission::States(&[4, 4, 0, 0])).is_redstone_conductor(Rule::Never),
    Block::of("minecraft:copper_chest", BlockClass::WeatheringCopperChestBlock).properties(&[&HORIZONTAL_FACING, &CHEST_TYPE, &WATERLOGGED], &["north", "single", "false"]).strength(3.0, 6.0).requires_correct_tool_for_drops(),
    Block::of("minecraft:exposed_copper_chest", BlockClass::WeatheringCopperChestBlock).properties(&[&HORIZONTAL_FACING, &CHEST_TYPE, &WATERLOGGED], &["north", "single", "false"]).strength(3.0, 6.0).requires_correct_tool_for_drops(),
    Block::of("minecraft:weathered_copper_chest", BlockClass::WeatheringCopperChestBlock).properties(&[&HORIZONTAL_FACING, &CHEST_TYPE, &WATERLOGGED], &["north", "single", "false"]).strength(3.0, 6.0).requires_correct_tool_for_drops(),
    Block::of("minecraft:oxidized_copper_chest", BlockClass::WeatheringCopperChestBlock).properties(&[&HORIZONTAL_FACING, &CHEST_TYPE, &WATERLOGGED], &["north", "single", "false"]).strength(3.0, 6.0).requires_correct_tool_for_drops(),
    Block::of("minecraft:waxed_copper_chest", BlockClass::CopperChestBlock).properties(&[&HORIZONTAL_FACING, &CHEST_TYPE, &WATERLOGGED], &["north", "single", "false"]).strength(3.0, 6.0).requires_correct_tool_for_drops(),
    Block::of("minecraft:waxed_exposed_copper_chest", BlockClass::CopperChestBlock).properties(&[&HORIZONTAL_FACING, &CHEST_TYPE, &WATERLOGGED], &["north", "single", "false"]).strength(3.0, 6.0).requires_correct_tool_for_drops(),
    Block::of("minecraft:waxed_weathered_copper_chest", BlockClass::CopperChestBlock).properties(&[&HORIZONTAL_FACING, &CHEST_TYPE, &WATERLOGGED], &["north", "single", "false"]).strength(3.0, 6.0).requires_correct_tool_for_drops(),
    Block::of("minecraft:waxed_oxidized_copper_chest", BlockClass::CopperChestBlock).properties(&[&HORIZONTAL_FACING, &CHEST_TYPE, &WATERLOGGED], &["north", "single", "false"]).strength(3.0, 6.0).requires_correct_tool_for_drops(),
    Block::of("minecraft:copper_golem_statue", BlockClass::WeatheringCopperGolemStatueBlock).properties(&[&COPPER_GOLEM_POSE, &HORIZONTAL_FACING, &WATERLOGGED], &["standing", "north", "false"]).strength(3.0, 6.0).no_occlusion().push_reaction(PushReaction::Destroy),
    Block::of("minecraft:exposed_copper_golem_statue", BlockClass::WeatheringCopperGolemStatueBlock).properties(&[&COPPER_GOLEM_POSE, &HORIZONTAL_FACING, &WATERLOGGED], &["standing", "north", "false"]).strength(3.0, 6.0).no_occlusion().push_reaction(PushReaction::Destroy),
    Block::of("minecraft:weathered_copper_golem_statue", BlockClass::WeatheringCopperGolemStatueBlock).properties(&[&COPPER_GOLEM_POSE, &HORIZONTAL_FACING, &WATERLOGGED], &["standing", "north", "false"]).strength(3.0, 6.0).no_occlusion().push_reaction(PushReaction::Destroy),
    Block::of("minecraft:oxidized_copper_golem_statue", BlockClass::WeatheringCopperGolemStatueBlock).properties(&[&COPPER_GOLEM_POSE, &HORIZONTAL_FACING, &WATERLOGGED], &["standing", "north", "false"]).strength(3.0, 6.0).no_occlusion().push_reaction(PushReaction::Destroy),
    Block::of("minecraft:waxed_copper_golem_statue", BlockClass::CopperGolemStatueBlock).properties(&[&COPPER_GOLEM_POSE, &HORIZONTAL_FACING, &WATERLOGGED], &["standing", "north", "false"]).strength(3.0, 6.0).no_occlusion().push_reaction(PushReaction::Destroy),
    Block::of("minecraft:waxed_exposed_copper_golem_statue", BlockClass::CopperGolemStatueBlock).properties(&[&COPPER_GOLEM_POSE, &HORIZONTAL_FACING, &WATERLOGGED], &["standing", "north", "false"]).strength(3.0, 6.0).no_occlusion().push_reaction(PushReaction::Destroy),
    Block::of("minecraft:waxed_weathered_copper_golem_statue", BlockClass::CopperGolemStatueBlock).properties(&[&COPPER_GOLEM_POSE, &HORIZONTAL_FACING, &WATERLOGGED], &["standing", "north", "false"]).strength(3.0, 6.0).no_occlusion().push_reaction(PushReaction::Destroy),
    Block::of("minecraft:waxed_oxidized_copper_golem_statue", BlockClass::CopperGolemStatueBlock).properties(&[&COPPER_GOLEM_POSE, &HORIZONTAL_FACING, &WATERLOGGED], &["standing", "north", "false"]).strength(3.0, 6.0).no_occlusion().push_reaction(PushReaction::Destroy),
    Block::of("minecraft:lightning_rod", BlockClass::WeatheringLightningRodBlock).properties(&[&FACING, &POWERED, &WATERLOGGED], &["up", "false", "false"]).strength(3.0, 6.0).no_occlusion().requires_correct_tool_for_drops().force_solid_on(),
    Block::of("minecraft:exposed_lightning_rod", BlockClass::WeatheringLightningRodBlock).properties(&[&FACING, &POWERED, &WATERLOGGED], &["up", "false", "false"]).strength(3.0, 6.0).no_occlusion().requires_correct_tool_for_drops().force_solid_on(),
    Block::of("minecraft:weathered_lightning_rod", BlockClass::WeatheringLightningRodBlock).properties(&[&FACING, &POWERED, &WATERLOGGED], &["up", "false", "false"]).strength(3.0, 6.0).no_occlusion().requires_correct_tool_for_drops().force_solid_on(),
    Block::of("minecraft:oxidized_lightning_rod", BlockClass::WeatheringLightningRodBlock).properties(&[&FACING, &POWERED, &WATERLOGGED], &["up", "false", "false"]).strength(3.0, 6.0).no_occlusion().requires_correct_tool_for_drops().force_solid_on(),
    Block::of("minecraft:waxed_lightning_rod", BlockClass::LightningRodBlock).properties(&[&FACING, &POWERED, &WATERLOGGED], &["up", "false", "false"]).strength(3.0, 6.0).no_occlusion().requires_correct_tool_for_drops().force_solid_on(),
    Block::of("minecraft:waxed_exposed_lightning_rod", BlockClass::LightningRodBlock).properties(&[&FACING, &POWERED, &WATERLOGGED], &["up", "false", "false"]).strength(3.0, 6.0).no_occlusion().requires_correct_tool_for_drops().force_solid_on(),
    Block::of("minecraft:waxed_weathered_lightning_rod", BlockClass::LightningRodBlock).properties(&[&FACING, &POWERED, &WATERLOGGED], &["up", "false", "false"]).strength(3.0, 6.0).no_occlusion().requires_correct_tool_for_drops().force_solid_on(),
    Block::of("minecraft:waxed_oxidized_lightning_rod", BlockClass::LightningRodBlock).properties(&[&FACING, &POWERED, &WATERLOGGED], &["up", "false", "false"]).strength(3.0, 6.0).no_occlusion().requires_correct_tool_for_drops().force_solid_on(),
    Block::of("minecraft:dripstone_block", BlockClass::Block).strength(1.5, 1.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:pointed_dripstone", BlockClass::PointedDripstoneBlock).properties(&[&SPELEOTHEM_THICKNESS, &VERTICAL_DIRECTION, &WATERLOGGED], &["tip", "up", "false"]).strength(1.5, 3.0).no_occlusion().random_ticks().force_solid_on().dynamic_shape().offset_type(OffsetType::Xz).push_reaction(PushReaction::Destroy).instrument("basedrum").is_redstone_conductor(Rule::Never),
    Block::of("minecraft:sulfur_spike", BlockClass::SulfurSpikeBlock).properties(&[&SPELEOTHEM_THICKNESS, &VERTICAL_DIRECTION, &WATERLOGGED], &["tip", "up", "false"]).strength(1.5, 3.0).no_occlusion().random_ticks().force_solid_on().dynamic_shape().offset_type(OffsetType::Xz).push_reaction(PushReaction::Destroy).instrument("basedrum").is_redstone_conductor(Rule::Never),
    Block::of("minecraft:cave_vines", BlockClass::CaveVinesBlock).properties(&[&AGE_25, &BERRIES], &["0", "false"]).no_collision().random_ticks().push_reaction(PushReaction::Destroy).item("minecraft:glow_berries").light_level(Emission::States(&[14, 0, 14, 0, 14, 0, 14, 0, 14, 0, 14, 0, 14, 0, 14, 0, 14, 0, 14, 0, 14, 0, 14, 0, 14, 0, 14, 0, 14, 0, 14, 0, 14, 0, 14, 0, 14, 0, 14, 0, 14, 0, 14, 0, 14, 0, 14, 0, 14, 0, 14, 0])),
    Block::of("minecraft:cave_vines_plant", BlockClass::CaveVinesPlantBlock).properties(&[&BERRIES], &["false"]).no_collision().push_reaction(PushReaction::Destroy).item("minecraft:air").light_level(Emission::States(&[14, 0])),
    Block::of("minecraft:spore_blossom", BlockClass::SporeBlossomBlock).no_collision().push_reaction(PushReaction::Destroy),
    Block::of("minecraft:azalea", BlockClass::AzaleaBlock).no_occlusion().force_solid_off().push_reaction(PushReaction::Destroy),
    Block::of("minecraft:flowering_azalea", BlockClass::AzaleaBlock).no_occlusion().force_solid_off().push_reaction(PushReaction::Destroy),
    Block::of("minecraft:moss_carpet", BlockClass::CarpetBlock).strength(0.1, 0.1).push_reaction(PushReaction::Destroy),
    Block::of("minecraft:pink_petals", BlockClass::FlowerBedBlock).properties(&[&HORIZONTAL_FACING, &FLOWER_AMOUNT], &["north", "1"]).no_collision().push_reaction(PushReaction::Destroy),
    Block::of("minecraft:wildflowers", BlockClass::FlowerBedBlock).properties(&[&HORIZONTAL_FACING, &FLOWER_AMOUNT], &["north", "1"]).no_collision().push_reaction(PushReaction::Destroy),
    Block::of("minecraft:leaf_litter", BlockClass::LeafLitterBlock).properties(&[&HORIZONTAL_FACING, &SEGMENT_AMOUNT], &["north", "1"]).no_collision().replaceable().push_reaction(PushReaction::Destroy),
    Block::of("minecraft:moss_block", BlockClass::BonemealableFeaturePlacerBlock).strength(0.1, 0.1).push_reaction(PushReaction::Destroy),
    Block::of("minecraft:big_dripleaf", BlockClass::BigDripleafBlock).properties(&[&HORIZONTAL_FACING, &TILT, &WATERLOGGED], &["north", "none", "false"]).strength(0.1, 0.1).force_solid_off().push_reaction(PushReaction::Destroy),
    Block::of("minecraft:big_dripleaf_stem", BlockClass::BigDripleafStemBlock).properties(&[&HORIZONTAL_FACING, &WATERLOGGED], &["north", "false"]).strength(0.1, 0.1).no_collision().push_reaction(PushReaction::Destroy).item("minecraft:big_dripleaf"),
    Block::of("minecraft:small_dripleaf", BlockClass::SmallDripleafBlock).properties(&[&HORIZONTAL_FACING, &DOUBLE_BLOCK_HALF, &WATERLOGGED], &["north", "lower", "false"]).no_collision().offset_type(OffsetType::Xyz).push_reaction(PushReaction::Destroy),
    Block::of("minecraft:hanging_roots", BlockClass::HangingRootsBlock).properties(&[&WATERLOGGED], &["false"]).no_collision().ignited_by_lava().replaceable().offset_type(OffsetType::Xz).push_reaction(PushReaction::Destroy),
    Block::of("minecraft:rooted_dirt", BlockClass::RootedDirtBlock).strength(0.5, 0.5),
    Block::of("minecraft:mud", BlockClass::MudBlock).strength(0.5, 0.5).is_suffocating(Rule::Always).is_view_blocking(Rule::Always).is_redstone_conductor(Rule::Always),
    Block::of("minecraft:deepslate", BlockClass::RotatedPillarBlock).properties(&[&AXIS], &["y"]).strength(3.0, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:cobbled_deepslate", BlockClass::Block).strength(3.5, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:cobbled_deepslate_stairs", BlockClass::StairBlock).properties(&[&HORIZONTAL_FACING, &HALF, &STAIRS_SHAPE, &WATERLOGGED], &["north", "bottom", "straight", "false"]).strength(3.5, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:cobbled_deepslate_slab", BlockClass::SlabBlock).properties(&[&SLAB_TYPE, &WATERLOGGED], &["bottom", "false"]).strength(3.5, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:cobbled_deepslate_wall", BlockClass::WallBlock).properties(&[&EAST_WALL, &NORTH_WALL, &SOUTH_WALL, &UP, &WATERLOGGED, &WEST_WALL], &["none", "none", "none", "true", "false", "none"]).strength(3.5, 6.0).requires_correct_tool_for_drops().force_solid_on().instrument("basedrum"),
    Block::of("minecraft:polished_deepslate", BlockClass::Block).strength(3.5, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:polished_deepslate_stairs", BlockClass::StairBlock).properties(&[&HORIZONTAL_FACING, &HALF, &STAIRS_SHAPE, &WATERLOGGED], &["north", "bottom", "straight", "false"]).strength(3.5, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:polished_deepslate_slab", BlockClass::SlabBlock).properties(&[&SLAB_TYPE, &WATERLOGGED], &["bottom", "false"]).strength(3.5, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:polished_deepslate_wall", BlockClass::WallBlock).properties(&[&EAST_WALL, &NORTH_WALL, &SOUTH_WALL, &UP, &WATERLOGGED, &WEST_WALL], &["none", "none", "none", "true", "false", "none"]).strength(3.5, 6.0).requires_correct_tool_for_drops().force_solid_on().instrument("basedrum"),
    Block::of("minecraft:deepslate_tiles", BlockClass::Block).strength(3.5, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:deepslate_tile_stairs", BlockClass::StairBlock).properties(&[&HORIZONTAL_FACING, &HALF, &STAIRS_SHAPE, &WATERLOGGED], &["north", "bottom", "straight", "false"]).strength(3.5, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:deepslate_tile_slab", BlockClass::SlabBlock).properties(&[&SLAB_TYPE, &WATERLOGGED], &["bottom", "false"]).strength(3.5, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:deepslate_tile_wall", BlockClass::WallBlock).properties(&[&EAST_WALL, &NORTH_WALL, &SOUTH_WALL, &UP, &WATERLOGGED, &WEST_WALL], &["none", "none", "none", "true", "false", "none"]).strength(3.5, 6.0).requires_correct_tool_for_drops().force_solid_on().instrument("basedrum"),
    Block::of("minecraft:deepslate_bricks", BlockClass::Block).strength(3.5, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:deepslate_brick_stairs", BlockClass::StairBlock).properties(&[&HORIZONTAL_FACING, &HALF, &STAIRS_SHAPE, &WATERLOGGED], &["north", "bottom", "straight", "false"]).strength(3.5, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:deepslate_brick_slab", BlockClass::SlabBlock).properties(&[&SLAB_TYPE, &WATERLOGGED], &["bottom", "false"]).strength(3.5, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:deepslate_brick_wall", BlockClass::WallBlock).properties(&[&EAST_WALL, &NORTH_WALL, &SOUTH_WALL, &UP, &WATERLOGGED, &WEST_WALL], &["none", "none", "none", "true", "false", "none"]).strength(3.5, 6.0).requires_correct_tool_for_drops().force_solid_on().instrument("basedrum"),
    Block::of("minecraft:chiseled_deepslate", BlockClass::Block).strength(3.5, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:cracked_deepslate_bricks", BlockClass::Block).strength(3.5, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:cracked_deepslate_tiles", BlockClass::Block).strength(3.5, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:infested_deepslate", BlockClass::InfestedRotatedPillarBlock).properties(&[&AXIS], &["y"]).strength(1.5, 0.75),
    Block::of("minecraft:smooth_basalt", BlockClass::Block).strength(1.25, 4.2).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:raw_iron_block", BlockClass::Block).strength(5.0, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:raw_copper_block", BlockClass::Block).strength(5.0, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:raw_gold_block", BlockClass::Block).strength(5.0, 6.0).requires_correct_tool_for_drops().instrument("basedrum"),
    Block::of("minecraft:potted_azalea_bush", BlockClass::FlowerPotBlock).no_occlusion().push_reaction(PushReaction::Destroy).item("minecraft:air"),
    Block::of("minecraft:potted_flowering_azalea_bush", BlockClass::FlowerPotBlock).no_occlusion().push_reaction(PushReaction::Destroy).item("minecraft:air"),
    Block::of("minecraft:ochre_froglight", BlockClass::RotatedPillarBlock).properties(&[&AXIS], &["y"]).strength(0.3, 0.3).light_level(Emission::Constant(15)),
    Block::of("minecraft:verdant_froglight", BlockClass::RotatedPillarBlock).properties(&[&AXIS], &["y"]).strength(0.3, 0.3).light_level(Emission::Constant(15)),
    Block::of("minecraft:pearlescent_froglight", BlockClass::RotatedPillarBlock).properties(&[&AXIS], &["y"]).strength(0.3, 0.3).light_level(Emission::Constant(15)),
    Block::of("minecraft:frogspawn", BlockClass::FrogspawnBlock).no_collision().push_reaction(PushReaction::Destroy),
    Block::of("minecraft:reinforced_deepslate", BlockClass::Block).strength(55.0, 1200.0).instrument("basedrum"),
    Block::of("minecraft:decorated_pot", BlockClass::DecoratedPotBlock).properties(&[&CRACKED, &HORIZONTAL_FACING, &WATERLOGGED], &["false", "north", "false"]).no_occlusion().push_reaction(PushReaction::Destroy),
    Block::of("minecraft:crafter", BlockClass::CrafterBlock).properties(&[&CRAFTING, &ORIENTATION, &TRIGGERED], &["false", "north_up", "false"]).strength(1.5, 3.5),
    Block::of("minecraft:trial_spawner", BlockClass::TrialSpawnerBlock).properties(&[&OMINOUS, &TRIAL_SPAWNER_STATE], &["false", "inactive"]).strength(50.0, 50.0).no_occlusion().instrument("basedrum").light_level(Emission::States(&[0, 4, 8, 8, 8, 0, 0, 4, 8, 8, 8, 0])).is_view_blocking(Rule::Never),
    Block::of("minecraft:vault", BlockClass::VaultBlock).properties(&[&HORIZONTAL_FACING, &OMINOUS, &VAULT_STATE], &["north", "false", "inactive"]).strength(50.0, 50.0).no_occlusion().instrument("basedrum").light_level(Emission::States(&[6, 12, 12, 12, 6, 12, 12, 12, 6, 12, 12, 12, 6, 12, 12, 12, 6, 12, 12, 12, 6, 12, 12, 12, 6, 12, 12, 12, 6, 12, 12, 12])).is_view_blocking(Rule::Never),
    Block::of("minecraft:heavy_core", BlockClass::HeavyCoreBlock).properties(&[&WATERLOGGED], &["false"]).strength(10.0, 1200.0).instrument("snare"),
    Block::of("minecraft:pale_moss_block", BlockClass::BonemealableFeaturePlacerBlock).strength(0.1, 0.1).ignited_by_lava().push_reaction(PushReaction::Destroy),
    Block::of("minecraft:pale_moss_carpet", BlockClass::MossyCarpetBlock).properties(&[&BOTTOM, &EAST_WALL, &NORTH_WALL, &SOUTH_WALL, &WEST_WALL], &["true", "none", "none", "none", "none"]).strength(0.1, 0.1).no_occlusion().ignited_by_lava().push_reaction(PushReaction::Destroy),
    Block::of("minecraft:pale_hanging_moss", BlockClass::HangingMossBlock).properties(&[&TIP], &["true"]).no_collision().ignited_by_lava().push_reaction(PushReaction::Destroy),
    Block::of("minecraft:open_eyeblossom", BlockClass::EyeblossomBlock).no_collision().random_ticks().offset_type(OffsetType::Xz).push_reaction(PushReaction::Destroy),
    Block::of("minecraft:closed_eyeblossom", BlockClass::EyeblossomBlock).no_collision().random_ticks().offset_type(OffsetType::Xz).push_reaction(PushReaction::Destroy),
    Block::of("minecraft:potted_open_eyeblossom", BlockClass::FlowerPotBlock).no_occlusion().random_ticks().push_reaction(PushReaction::Destroy).item("minecraft:air"),
    Block::of("minecraft:potted_closed_eyeblossom", BlockClass::FlowerPotBlock).no_occlusion().random_ticks().push_reaction(PushReaction::Destroy).item("minecraft:air"),
    Block::of("minecraft:firefly_bush", BlockClass::FireflyBushBlock).no_collision().ignited_by_lava().push_reaction(PushReaction::Destroy).light_level(Emission::Constant(2)),
];

/// Each block's place in the registry, named as `Blocks` names it.
#[rustfmt::skip]
pub mod id {
    pub const AIR: usize = 0;
    pub const STONE: usize = 1;
    pub const GRANITE: usize = 2;
    pub const POLISHED_GRANITE: usize = 3;
    pub const DIORITE: usize = 4;
    pub const POLISHED_DIORITE: usize = 5;
    pub const ANDESITE: usize = 6;
    pub const POLISHED_ANDESITE: usize = 7;
    pub const GRASS_BLOCK: usize = 8;
    pub const DIRT: usize = 9;
    pub const COARSE_DIRT: usize = 10;
    pub const PODZOL: usize = 11;
    pub const COBBLESTONE: usize = 12;
    pub const OAK_PLANKS: usize = 13;
    pub const SPRUCE_PLANKS: usize = 14;
    pub const BIRCH_PLANKS: usize = 15;
    pub const JUNGLE_PLANKS: usize = 16;
    pub const ACACIA_PLANKS: usize = 17;
    pub const CHERRY_PLANKS: usize = 18;
    pub const DARK_OAK_PLANKS: usize = 19;
    pub const PALE_OAK_WOOD: usize = 20;
    pub const PALE_OAK_PLANKS: usize = 21;
    pub const MANGROVE_PLANKS: usize = 22;
    pub const BAMBOO_PLANKS: usize = 23;
    pub const BAMBOO_MOSAIC: usize = 24;
    pub const OAK_SAPLING: usize = 25;
    pub const SPRUCE_SAPLING: usize = 26;
    pub const BIRCH_SAPLING: usize = 27;
    pub const JUNGLE_SAPLING: usize = 28;
    pub const ACACIA_SAPLING: usize = 29;
    pub const CHERRY_SAPLING: usize = 30;
    pub const DARK_OAK_SAPLING: usize = 31;
    pub const PALE_OAK_SAPLING: usize = 32;
    pub const MANGROVE_PROPAGULE: usize = 33;
    pub const BEDROCK: usize = 34;
    pub const WATER: usize = 35;
    pub const LAVA: usize = 36;
    pub const SAND: usize = 37;
    pub const SUSPICIOUS_SAND: usize = 38;
    pub const RED_SAND: usize = 39;
    pub const GRAVEL: usize = 40;
    pub const SUSPICIOUS_GRAVEL: usize = 41;
    pub const GOLD_ORE: usize = 42;
    pub const DEEPSLATE_GOLD_ORE: usize = 43;
    pub const IRON_ORE: usize = 44;
    pub const DEEPSLATE_IRON_ORE: usize = 45;
    pub const COAL_ORE: usize = 46;
    pub const DEEPSLATE_COAL_ORE: usize = 47;
    pub const NETHER_GOLD_ORE: usize = 48;
    pub const OAK_LOG: usize = 49;
    pub const SPRUCE_LOG: usize = 50;
    pub const BIRCH_LOG: usize = 51;
    pub const JUNGLE_LOG: usize = 52;
    pub const ACACIA_LOG: usize = 53;
    pub const CHERRY_LOG: usize = 54;
    pub const DARK_OAK_LOG: usize = 55;
    pub const PALE_OAK_LOG: usize = 56;
    pub const MANGROVE_LOG: usize = 57;
    pub const MANGROVE_ROOTS: usize = 58;
    pub const MUDDY_MANGROVE_ROOTS: usize = 59;
    pub const BAMBOO_BLOCK: usize = 60;
    pub const STRIPPED_SPRUCE_LOG: usize = 61;
    pub const STRIPPED_BIRCH_LOG: usize = 62;
    pub const STRIPPED_JUNGLE_LOG: usize = 63;
    pub const STRIPPED_ACACIA_LOG: usize = 64;
    pub const STRIPPED_CHERRY_LOG: usize = 65;
    pub const STRIPPED_DARK_OAK_LOG: usize = 66;
    pub const STRIPPED_PALE_OAK_LOG: usize = 67;
    pub const STRIPPED_OAK_LOG: usize = 68;
    pub const STRIPPED_MANGROVE_LOG: usize = 69;
    pub const STRIPPED_BAMBOO_BLOCK: usize = 70;
    pub const OAK_WOOD: usize = 71;
    pub const SPRUCE_WOOD: usize = 72;
    pub const BIRCH_WOOD: usize = 73;
    pub const JUNGLE_WOOD: usize = 74;
    pub const ACACIA_WOOD: usize = 75;
    pub const CHERRY_WOOD: usize = 76;
    pub const DARK_OAK_WOOD: usize = 77;
    pub const MANGROVE_WOOD: usize = 78;
    pub const STRIPPED_OAK_WOOD: usize = 79;
    pub const STRIPPED_SPRUCE_WOOD: usize = 80;
    pub const STRIPPED_BIRCH_WOOD: usize = 81;
    pub const STRIPPED_JUNGLE_WOOD: usize = 82;
    pub const STRIPPED_ACACIA_WOOD: usize = 83;
    pub const STRIPPED_CHERRY_WOOD: usize = 84;
    pub const STRIPPED_DARK_OAK_WOOD: usize = 85;
    pub const STRIPPED_PALE_OAK_WOOD: usize = 86;
    pub const STRIPPED_MANGROVE_WOOD: usize = 87;
    pub const OAK_LEAVES: usize = 88;
    pub const SPRUCE_LEAVES: usize = 89;
    pub const BIRCH_LEAVES: usize = 90;
    pub const JUNGLE_LEAVES: usize = 91;
    pub const ACACIA_LEAVES: usize = 92;
    pub const CHERRY_LEAVES: usize = 93;
    pub const DARK_OAK_LEAVES: usize = 94;
    pub const PALE_OAK_LEAVES: usize = 95;
    pub const MANGROVE_LEAVES: usize = 96;
    pub const AZALEA_LEAVES: usize = 97;
    pub const FLOWERING_AZALEA_LEAVES: usize = 98;
    pub const SPONGE: usize = 99;
    pub const WET_SPONGE: usize = 100;
    pub const GLASS: usize = 101;
    pub const LAPIS_ORE: usize = 102;
    pub const DEEPSLATE_LAPIS_ORE: usize = 103;
    pub const LAPIS_BLOCK: usize = 104;
    pub const DISPENSER: usize = 105;
    pub const SANDSTONE: usize = 106;
    pub const CHISELED_SANDSTONE: usize = 107;
    pub const CUT_SANDSTONE: usize = 108;
    pub const NOTE_BLOCK: usize = 109;
    pub const WHITE_BED: usize = 110;
    pub const ORANGE_BED: usize = 111;
    pub const MAGENTA_BED: usize = 112;
    pub const LIGHT_BLUE_BED: usize = 113;
    pub const YELLOW_BED: usize = 114;
    pub const LIME_BED: usize = 115;
    pub const PINK_BED: usize = 116;
    pub const GRAY_BED: usize = 117;
    pub const LIGHT_GRAY_BED: usize = 118;
    pub const CYAN_BED: usize = 119;
    pub const PURPLE_BED: usize = 120;
    pub const BLUE_BED: usize = 121;
    pub const BROWN_BED: usize = 122;
    pub const GREEN_BED: usize = 123;
    pub const RED_BED: usize = 124;
    pub const BLACK_BED: usize = 125;
    pub const POWERED_RAIL: usize = 126;
    pub const DETECTOR_RAIL: usize = 127;
    pub const STICKY_PISTON: usize = 128;
    pub const COBWEB: usize = 129;
    pub const SHORT_GRASS: usize = 130;
    pub const FERN: usize = 131;
    pub const DEAD_BUSH: usize = 132;
    pub const BUSH: usize = 133;
    pub const SHORT_DRY_GRASS: usize = 134;
    pub const TALL_DRY_GRASS: usize = 135;
    pub const SEAGRASS: usize = 136;
    pub const TALL_SEAGRASS: usize = 137;
    pub const PISTON: usize = 138;
    pub const PISTON_HEAD: usize = 139;
    pub const WHITE_WOOL: usize = 140;
    pub const ORANGE_WOOL: usize = 141;
    pub const MAGENTA_WOOL: usize = 142;
    pub const LIGHT_BLUE_WOOL: usize = 143;
    pub const YELLOW_WOOL: usize = 144;
    pub const LIME_WOOL: usize = 145;
    pub const PINK_WOOL: usize = 146;
    pub const GRAY_WOOL: usize = 147;
    pub const LIGHT_GRAY_WOOL: usize = 148;
    pub const CYAN_WOOL: usize = 149;
    pub const PURPLE_WOOL: usize = 150;
    pub const BLUE_WOOL: usize = 151;
    pub const BROWN_WOOL: usize = 152;
    pub const GREEN_WOOL: usize = 153;
    pub const RED_WOOL: usize = 154;
    pub const BLACK_WOOL: usize = 155;
    pub const MOVING_PISTON: usize = 156;
    pub const DANDELION: usize = 157;
    pub const GOLDEN_DANDELION: usize = 158;
    pub const TORCHFLOWER: usize = 159;
    pub const POPPY: usize = 160;
    pub const BLUE_ORCHID: usize = 161;
    pub const ALLIUM: usize = 162;
    pub const AZURE_BLUET: usize = 163;
    pub const RED_TULIP: usize = 164;
    pub const ORANGE_TULIP: usize = 165;
    pub const WHITE_TULIP: usize = 166;
    pub const PINK_TULIP: usize = 167;
    pub const OXEYE_DAISY: usize = 168;
    pub const CORNFLOWER: usize = 169;
    pub const WITHER_ROSE: usize = 170;
    pub const LILY_OF_THE_VALLEY: usize = 171;
    pub const BROWN_MUSHROOM: usize = 172;
    pub const RED_MUSHROOM: usize = 173;
    pub const GOLD_BLOCK: usize = 174;
    pub const IRON_BLOCK: usize = 175;
    pub const BRICKS: usize = 176;
    pub const TNT: usize = 177;
    pub const BOOKSHELF: usize = 178;
    pub const CHISELED_BOOKSHELF: usize = 179;
    pub const ACACIA_SHELF: usize = 180;
    pub const BAMBOO_SHELF: usize = 181;
    pub const BIRCH_SHELF: usize = 182;
    pub const CHERRY_SHELF: usize = 183;
    pub const CRIMSON_SHELF: usize = 184;
    pub const DARK_OAK_SHELF: usize = 185;
    pub const JUNGLE_SHELF: usize = 186;
    pub const MANGROVE_SHELF: usize = 187;
    pub const OAK_SHELF: usize = 188;
    pub const PALE_OAK_SHELF: usize = 189;
    pub const SPRUCE_SHELF: usize = 190;
    pub const WARPED_SHELF: usize = 191;
    pub const MOSSY_COBBLESTONE: usize = 192;
    pub const OBSIDIAN: usize = 193;
    pub const TORCH: usize = 194;
    pub const WALL_TORCH: usize = 195;
    pub const FIRE: usize = 196;
    pub const SOUL_FIRE: usize = 197;
    pub const SPAWNER: usize = 198;
    pub const CREAKING_HEART: usize = 199;
    pub const OAK_STAIRS: usize = 200;
    pub const CHEST: usize = 201;
    pub const REDSTONE_WIRE: usize = 202;
    pub const DIAMOND_ORE: usize = 203;
    pub const DEEPSLATE_DIAMOND_ORE: usize = 204;
    pub const DIAMOND_BLOCK: usize = 205;
    pub const CRAFTING_TABLE: usize = 206;
    pub const WHEAT: usize = 207;
    pub const FARMLAND: usize = 208;
    pub const FURNACE: usize = 209;
    pub const OAK_SIGN: usize = 210;
    pub const SPRUCE_SIGN: usize = 211;
    pub const BIRCH_SIGN: usize = 212;
    pub const ACACIA_SIGN: usize = 213;
    pub const CHERRY_SIGN: usize = 214;
    pub const JUNGLE_SIGN: usize = 215;
    pub const DARK_OAK_SIGN: usize = 216;
    pub const PALE_OAK_SIGN: usize = 217;
    pub const MANGROVE_SIGN: usize = 218;
    pub const BAMBOO_SIGN: usize = 219;
    pub const OAK_DOOR: usize = 220;
    pub const LADDER: usize = 221;
    pub const RAIL: usize = 222;
    pub const COBBLESTONE_STAIRS: usize = 223;
    pub const OAK_WALL_SIGN: usize = 224;
    pub const SPRUCE_WALL_SIGN: usize = 225;
    pub const BIRCH_WALL_SIGN: usize = 226;
    pub const ACACIA_WALL_SIGN: usize = 227;
    pub const CHERRY_WALL_SIGN: usize = 228;
    pub const JUNGLE_WALL_SIGN: usize = 229;
    pub const DARK_OAK_WALL_SIGN: usize = 230;
    pub const PALE_OAK_WALL_SIGN: usize = 231;
    pub const MANGROVE_WALL_SIGN: usize = 232;
    pub const BAMBOO_WALL_SIGN: usize = 233;
    pub const OAK_HANGING_SIGN: usize = 234;
    pub const SPRUCE_HANGING_SIGN: usize = 235;
    pub const BIRCH_HANGING_SIGN: usize = 236;
    pub const ACACIA_HANGING_SIGN: usize = 237;
    pub const CHERRY_HANGING_SIGN: usize = 238;
    pub const JUNGLE_HANGING_SIGN: usize = 239;
    pub const DARK_OAK_HANGING_SIGN: usize = 240;
    pub const PALE_OAK_HANGING_SIGN: usize = 241;
    pub const CRIMSON_HANGING_SIGN: usize = 242;
    pub const WARPED_HANGING_SIGN: usize = 243;
    pub const MANGROVE_HANGING_SIGN: usize = 244;
    pub const BAMBOO_HANGING_SIGN: usize = 245;
    pub const OAK_WALL_HANGING_SIGN: usize = 246;
    pub const SPRUCE_WALL_HANGING_SIGN: usize = 247;
    pub const BIRCH_WALL_HANGING_SIGN: usize = 248;
    pub const ACACIA_WALL_HANGING_SIGN: usize = 249;
    pub const CHERRY_WALL_HANGING_SIGN: usize = 250;
    pub const JUNGLE_WALL_HANGING_SIGN: usize = 251;
    pub const DARK_OAK_WALL_HANGING_SIGN: usize = 252;
    pub const PALE_OAK_WALL_HANGING_SIGN: usize = 253;
    pub const MANGROVE_WALL_HANGING_SIGN: usize = 254;
    pub const CRIMSON_WALL_HANGING_SIGN: usize = 255;
    pub const WARPED_WALL_HANGING_SIGN: usize = 256;
    pub const BAMBOO_WALL_HANGING_SIGN: usize = 257;
    pub const LEVER: usize = 258;
    pub const STONE_PRESSURE_PLATE: usize = 259;
    pub const IRON_DOOR: usize = 260;
    pub const OAK_PRESSURE_PLATE: usize = 261;
    pub const SPRUCE_PRESSURE_PLATE: usize = 262;
    pub const BIRCH_PRESSURE_PLATE: usize = 263;
    pub const JUNGLE_PRESSURE_PLATE: usize = 264;
    pub const ACACIA_PRESSURE_PLATE: usize = 265;
    pub const CHERRY_PRESSURE_PLATE: usize = 266;
    pub const DARK_OAK_PRESSURE_PLATE: usize = 267;
    pub const PALE_OAK_PRESSURE_PLATE: usize = 268;
    pub const MANGROVE_PRESSURE_PLATE: usize = 269;
    pub const BAMBOO_PRESSURE_PLATE: usize = 270;
    pub const REDSTONE_ORE: usize = 271;
    pub const DEEPSLATE_REDSTONE_ORE: usize = 272;
    pub const REDSTONE_TORCH: usize = 273;
    pub const REDSTONE_WALL_TORCH: usize = 274;
    pub const STONE_BUTTON: usize = 275;
    pub const SNOW: usize = 276;
    pub const ICE: usize = 277;
    pub const SNOW_BLOCK: usize = 278;
    pub const CACTUS: usize = 279;
    pub const CACTUS_FLOWER: usize = 280;
    pub const CLAY: usize = 281;
    pub const SUGAR_CANE: usize = 282;
    pub const JUKEBOX: usize = 283;
    pub const OAK_FENCE: usize = 284;
    pub const NETHERRACK: usize = 285;
    pub const SOUL_SAND: usize = 286;
    pub const SOUL_SOIL: usize = 287;
    pub const BASALT: usize = 288;
    pub const POLISHED_BASALT: usize = 289;
    pub const SOUL_TORCH: usize = 290;
    pub const SOUL_WALL_TORCH: usize = 291;
    pub const COPPER_TORCH: usize = 292;
    pub const COPPER_WALL_TORCH: usize = 293;
    pub const GLOWSTONE: usize = 294;
    pub const NETHER_PORTAL: usize = 295;
    pub const CARVED_PUMPKIN: usize = 296;
    pub const JACK_O_LANTERN: usize = 297;
    pub const CAKE: usize = 298;
    pub const REPEATER: usize = 299;
    pub const WHITE_STAINED_GLASS: usize = 300;
    pub const ORANGE_STAINED_GLASS: usize = 301;
    pub const MAGENTA_STAINED_GLASS: usize = 302;
    pub const LIGHT_BLUE_STAINED_GLASS: usize = 303;
    pub const YELLOW_STAINED_GLASS: usize = 304;
    pub const LIME_STAINED_GLASS: usize = 305;
    pub const PINK_STAINED_GLASS: usize = 306;
    pub const GRAY_STAINED_GLASS: usize = 307;
    pub const LIGHT_GRAY_STAINED_GLASS: usize = 308;
    pub const CYAN_STAINED_GLASS: usize = 309;
    pub const PURPLE_STAINED_GLASS: usize = 310;
    pub const BLUE_STAINED_GLASS: usize = 311;
    pub const BROWN_STAINED_GLASS: usize = 312;
    pub const GREEN_STAINED_GLASS: usize = 313;
    pub const RED_STAINED_GLASS: usize = 314;
    pub const BLACK_STAINED_GLASS: usize = 315;
    pub const OAK_TRAPDOOR: usize = 316;
    pub const SPRUCE_TRAPDOOR: usize = 317;
    pub const BIRCH_TRAPDOOR: usize = 318;
    pub const JUNGLE_TRAPDOOR: usize = 319;
    pub const ACACIA_TRAPDOOR: usize = 320;
    pub const CHERRY_TRAPDOOR: usize = 321;
    pub const DARK_OAK_TRAPDOOR: usize = 322;
    pub const PALE_OAK_TRAPDOOR: usize = 323;
    pub const MANGROVE_TRAPDOOR: usize = 324;
    pub const BAMBOO_TRAPDOOR: usize = 325;
    pub const STONE_BRICKS: usize = 326;
    pub const MOSSY_STONE_BRICKS: usize = 327;
    pub const CRACKED_STONE_BRICKS: usize = 328;
    pub const CHISELED_STONE_BRICKS: usize = 329;
    pub const PACKED_MUD: usize = 330;
    pub const MUD_BRICKS: usize = 331;
    pub const INFESTED_STONE: usize = 332;
    pub const INFESTED_COBBLESTONE: usize = 333;
    pub const INFESTED_STONE_BRICKS: usize = 334;
    pub const INFESTED_MOSSY_STONE_BRICKS: usize = 335;
    pub const INFESTED_CRACKED_STONE_BRICKS: usize = 336;
    pub const INFESTED_CHISELED_STONE_BRICKS: usize = 337;
    pub const BROWN_MUSHROOM_BLOCK: usize = 338;
    pub const RED_MUSHROOM_BLOCK: usize = 339;
    pub const MUSHROOM_STEM: usize = 340;
    pub const IRON_BARS: usize = 341;
    pub const COPPER_BARS: usize = 342;
    pub const EXPOSED_COPPER_BARS: usize = 343;
    pub const WEATHERED_COPPER_BARS: usize = 344;
    pub const OXIDIZED_COPPER_BARS: usize = 345;
    pub const WAXED_COPPER_BARS: usize = 346;
    pub const WAXED_EXPOSED_COPPER_BARS: usize = 347;
    pub const WAXED_WEATHERED_COPPER_BARS: usize = 348;
    pub const WAXED_OXIDIZED_COPPER_BARS: usize = 349;
    pub const IRON_CHAIN: usize = 350;
    pub const COPPER_CHAIN: usize = 351;
    pub const EXPOSED_COPPER_CHAIN: usize = 352;
    pub const WEATHERED_COPPER_CHAIN: usize = 353;
    pub const OXIDIZED_COPPER_CHAIN: usize = 354;
    pub const WAXED_COPPER_CHAIN: usize = 355;
    pub const WAXED_EXPOSED_COPPER_CHAIN: usize = 356;
    pub const WAXED_WEATHERED_COPPER_CHAIN: usize = 357;
    pub const WAXED_OXIDIZED_COPPER_CHAIN: usize = 358;
    pub const GLASS_PANE: usize = 359;
    pub const PUMPKIN: usize = 360;
    pub const MELON: usize = 361;
    pub const ATTACHED_PUMPKIN_STEM: usize = 362;
    pub const ATTACHED_MELON_STEM: usize = 363;
    pub const PUMPKIN_STEM: usize = 364;
    pub const MELON_STEM: usize = 365;
    pub const VINE: usize = 366;
    pub const GLOW_LICHEN: usize = 367;
    pub const RESIN_CLUMP: usize = 368;
    pub const OAK_FENCE_GATE: usize = 369;
    pub const BRICK_STAIRS: usize = 370;
    pub const STONE_BRICK_STAIRS: usize = 371;
    pub const MUD_BRICK_STAIRS: usize = 372;
    pub const MYCELIUM: usize = 373;
    pub const LILY_PAD: usize = 374;
    pub const RESIN_BLOCK: usize = 375;
    pub const RESIN_BRICKS: usize = 376;
    pub const RESIN_BRICK_STAIRS: usize = 377;
    pub const RESIN_BRICK_SLAB: usize = 378;
    pub const RESIN_BRICK_WALL: usize = 379;
    pub const CHISELED_RESIN_BRICKS: usize = 380;
    pub const NETHER_BRICKS: usize = 381;
    pub const NETHER_BRICK_FENCE: usize = 382;
    pub const NETHER_BRICK_STAIRS: usize = 383;
    pub const NETHER_WART: usize = 384;
    pub const ENCHANTING_TABLE: usize = 385;
    pub const BREWING_STAND: usize = 386;
    pub const CAULDRON: usize = 387;
    pub const WATER_CAULDRON: usize = 388;
    pub const LAVA_CAULDRON: usize = 389;
    pub const POWDER_SNOW_CAULDRON: usize = 390;
    pub const END_PORTAL: usize = 391;
    pub const END_PORTAL_FRAME: usize = 392;
    pub const END_STONE: usize = 393;
    pub const DRAGON_EGG: usize = 394;
    pub const REDSTONE_LAMP: usize = 395;
    pub const COCOA: usize = 396;
    pub const SANDSTONE_STAIRS: usize = 397;
    pub const EMERALD_ORE: usize = 398;
    pub const DEEPSLATE_EMERALD_ORE: usize = 399;
    pub const ENDER_CHEST: usize = 400;
    pub const TRIPWIRE_HOOK: usize = 401;
    pub const TRIPWIRE: usize = 402;
    pub const EMERALD_BLOCK: usize = 403;
    pub const SPRUCE_STAIRS: usize = 404;
    pub const BIRCH_STAIRS: usize = 405;
    pub const JUNGLE_STAIRS: usize = 406;
    pub const COMMAND_BLOCK: usize = 407;
    pub const BEACON: usize = 408;
    pub const COBBLESTONE_WALL: usize = 409;
    pub const MOSSY_COBBLESTONE_WALL: usize = 410;
    pub const FLOWER_POT: usize = 411;
    pub const POTTED_TORCHFLOWER: usize = 412;
    pub const POTTED_OAK_SAPLING: usize = 413;
    pub const POTTED_SPRUCE_SAPLING: usize = 414;
    pub const POTTED_BIRCH_SAPLING: usize = 415;
    pub const POTTED_JUNGLE_SAPLING: usize = 416;
    pub const POTTED_ACACIA_SAPLING: usize = 417;
    pub const POTTED_CHERRY_SAPLING: usize = 418;
    pub const POTTED_DARK_OAK_SAPLING: usize = 419;
    pub const POTTED_PALE_OAK_SAPLING: usize = 420;
    pub const POTTED_MANGROVE_PROPAGULE: usize = 421;
    pub const POTTED_FERN: usize = 422;
    pub const POTTED_DANDELION: usize = 423;
    pub const POTTED_GOLDEN_DANDELION: usize = 424;
    pub const POTTED_POPPY: usize = 425;
    pub const POTTED_BLUE_ORCHID: usize = 426;
    pub const POTTED_ALLIUM: usize = 427;
    pub const POTTED_AZURE_BLUET: usize = 428;
    pub const POTTED_RED_TULIP: usize = 429;
    pub const POTTED_ORANGE_TULIP: usize = 430;
    pub const POTTED_WHITE_TULIP: usize = 431;
    pub const POTTED_PINK_TULIP: usize = 432;
    pub const POTTED_OXEYE_DAISY: usize = 433;
    pub const POTTED_CORNFLOWER: usize = 434;
    pub const POTTED_LILY_OF_THE_VALLEY: usize = 435;
    pub const POTTED_WITHER_ROSE: usize = 436;
    pub const POTTED_RED_MUSHROOM: usize = 437;
    pub const POTTED_BROWN_MUSHROOM: usize = 438;
    pub const POTTED_DEAD_BUSH: usize = 439;
    pub const POTTED_CACTUS: usize = 440;
    pub const CARROTS: usize = 441;
    pub const POTATOES: usize = 442;
    pub const OAK_BUTTON: usize = 443;
    pub const SPRUCE_BUTTON: usize = 444;
    pub const BIRCH_BUTTON: usize = 445;
    pub const JUNGLE_BUTTON: usize = 446;
    pub const ACACIA_BUTTON: usize = 447;
    pub const CHERRY_BUTTON: usize = 448;
    pub const DARK_OAK_BUTTON: usize = 449;
    pub const PALE_OAK_BUTTON: usize = 450;
    pub const MANGROVE_BUTTON: usize = 451;
    pub const BAMBOO_BUTTON: usize = 452;
    pub const SKELETON_SKULL: usize = 453;
    pub const SKELETON_WALL_SKULL: usize = 454;
    pub const WITHER_SKELETON_SKULL: usize = 455;
    pub const WITHER_SKELETON_WALL_SKULL: usize = 456;
    pub const ZOMBIE_HEAD: usize = 457;
    pub const ZOMBIE_WALL_HEAD: usize = 458;
    pub const PLAYER_HEAD: usize = 459;
    pub const PLAYER_WALL_HEAD: usize = 460;
    pub const CREEPER_HEAD: usize = 461;
    pub const CREEPER_WALL_HEAD: usize = 462;
    pub const DRAGON_HEAD: usize = 463;
    pub const DRAGON_WALL_HEAD: usize = 464;
    pub const PIGLIN_HEAD: usize = 465;
    pub const PIGLIN_WALL_HEAD: usize = 466;
    pub const ANVIL: usize = 467;
    pub const CHIPPED_ANVIL: usize = 468;
    pub const DAMAGED_ANVIL: usize = 469;
    pub const TRAPPED_CHEST: usize = 470;
    pub const LIGHT_WEIGHTED_PRESSURE_PLATE: usize = 471;
    pub const HEAVY_WEIGHTED_PRESSURE_PLATE: usize = 472;
    pub const COMPARATOR: usize = 473;
    pub const DAYLIGHT_DETECTOR: usize = 474;
    pub const REDSTONE_BLOCK: usize = 475;
    pub const NETHER_QUARTZ_ORE: usize = 476;
    pub const HOPPER: usize = 477;
    pub const QUARTZ_BLOCK: usize = 478;
    pub const CHISELED_QUARTZ_BLOCK: usize = 479;
    pub const QUARTZ_PILLAR: usize = 480;
    pub const QUARTZ_STAIRS: usize = 481;
    pub const ACTIVATOR_RAIL: usize = 482;
    pub const DROPPER: usize = 483;
    pub const WHITE_TERRACOTTA: usize = 484;
    pub const ORANGE_TERRACOTTA: usize = 485;
    pub const MAGENTA_TERRACOTTA: usize = 486;
    pub const LIGHT_BLUE_TERRACOTTA: usize = 487;
    pub const YELLOW_TERRACOTTA: usize = 488;
    pub const LIME_TERRACOTTA: usize = 489;
    pub const PINK_TERRACOTTA: usize = 490;
    pub const GRAY_TERRACOTTA: usize = 491;
    pub const LIGHT_GRAY_TERRACOTTA: usize = 492;
    pub const CYAN_TERRACOTTA: usize = 493;
    pub const PURPLE_TERRACOTTA: usize = 494;
    pub const BLUE_TERRACOTTA: usize = 495;
    pub const BROWN_TERRACOTTA: usize = 496;
    pub const GREEN_TERRACOTTA: usize = 497;
    pub const RED_TERRACOTTA: usize = 498;
    pub const BLACK_TERRACOTTA: usize = 499;
    pub const WHITE_STAINED_GLASS_PANE: usize = 500;
    pub const ORANGE_STAINED_GLASS_PANE: usize = 501;
    pub const MAGENTA_STAINED_GLASS_PANE: usize = 502;
    pub const LIGHT_BLUE_STAINED_GLASS_PANE: usize = 503;
    pub const YELLOW_STAINED_GLASS_PANE: usize = 504;
    pub const LIME_STAINED_GLASS_PANE: usize = 505;
    pub const PINK_STAINED_GLASS_PANE: usize = 506;
    pub const GRAY_STAINED_GLASS_PANE: usize = 507;
    pub const LIGHT_GRAY_STAINED_GLASS_PANE: usize = 508;
    pub const CYAN_STAINED_GLASS_PANE: usize = 509;
    pub const PURPLE_STAINED_GLASS_PANE: usize = 510;
    pub const BLUE_STAINED_GLASS_PANE: usize = 511;
    pub const BROWN_STAINED_GLASS_PANE: usize = 512;
    pub const GREEN_STAINED_GLASS_PANE: usize = 513;
    pub const RED_STAINED_GLASS_PANE: usize = 514;
    pub const BLACK_STAINED_GLASS_PANE: usize = 515;
    pub const ACACIA_STAIRS: usize = 516;
    pub const CHERRY_STAIRS: usize = 517;
    pub const DARK_OAK_STAIRS: usize = 518;
    pub const PALE_OAK_STAIRS: usize = 519;
    pub const MANGROVE_STAIRS: usize = 520;
    pub const BAMBOO_STAIRS: usize = 521;
    pub const BAMBOO_MOSAIC_STAIRS: usize = 522;
    pub const SLIME_BLOCK: usize = 523;
    pub const BARRIER: usize = 524;
    pub const LIGHT: usize = 525;
    pub const IRON_TRAPDOOR: usize = 526;
    pub const PRISMARINE: usize = 527;
    pub const PRISMARINE_BRICKS: usize = 528;
    pub const DARK_PRISMARINE: usize = 529;
    pub const PRISMARINE_STAIRS: usize = 530;
    pub const PRISMARINE_BRICK_STAIRS: usize = 531;
    pub const DARK_PRISMARINE_STAIRS: usize = 532;
    pub const PRISMARINE_SLAB: usize = 533;
    pub const PRISMARINE_BRICK_SLAB: usize = 534;
    pub const DARK_PRISMARINE_SLAB: usize = 535;
    pub const SEA_LANTERN: usize = 536;
    pub const HAY_BLOCK: usize = 537;
    pub const WHITE_CARPET: usize = 538;
    pub const ORANGE_CARPET: usize = 539;
    pub const MAGENTA_CARPET: usize = 540;
    pub const LIGHT_BLUE_CARPET: usize = 541;
    pub const YELLOW_CARPET: usize = 542;
    pub const LIME_CARPET: usize = 543;
    pub const PINK_CARPET: usize = 544;
    pub const GRAY_CARPET: usize = 545;
    pub const LIGHT_GRAY_CARPET: usize = 546;
    pub const CYAN_CARPET: usize = 547;
    pub const PURPLE_CARPET: usize = 548;
    pub const BLUE_CARPET: usize = 549;
    pub const BROWN_CARPET: usize = 550;
    pub const GREEN_CARPET: usize = 551;
    pub const RED_CARPET: usize = 552;
    pub const BLACK_CARPET: usize = 553;
    pub const TERRACOTTA: usize = 554;
    pub const COAL_BLOCK: usize = 555;
    pub const PACKED_ICE: usize = 556;
    pub const SUNFLOWER: usize = 557;
    pub const LILAC: usize = 558;
    pub const ROSE_BUSH: usize = 559;
    pub const PEONY: usize = 560;
    pub const TALL_GRASS: usize = 561;
    pub const LARGE_FERN: usize = 562;
    pub const WHITE_BANNER: usize = 563;
    pub const ORANGE_BANNER: usize = 564;
    pub const MAGENTA_BANNER: usize = 565;
    pub const LIGHT_BLUE_BANNER: usize = 566;
    pub const YELLOW_BANNER: usize = 567;
    pub const LIME_BANNER: usize = 568;
    pub const PINK_BANNER: usize = 569;
    pub const GRAY_BANNER: usize = 570;
    pub const LIGHT_GRAY_BANNER: usize = 571;
    pub const CYAN_BANNER: usize = 572;
    pub const PURPLE_BANNER: usize = 573;
    pub const BLUE_BANNER: usize = 574;
    pub const BROWN_BANNER: usize = 575;
    pub const GREEN_BANNER: usize = 576;
    pub const RED_BANNER: usize = 577;
    pub const BLACK_BANNER: usize = 578;
    pub const WHITE_WALL_BANNER: usize = 579;
    pub const ORANGE_WALL_BANNER: usize = 580;
    pub const MAGENTA_WALL_BANNER: usize = 581;
    pub const LIGHT_BLUE_WALL_BANNER: usize = 582;
    pub const YELLOW_WALL_BANNER: usize = 583;
    pub const LIME_WALL_BANNER: usize = 584;
    pub const PINK_WALL_BANNER: usize = 585;
    pub const GRAY_WALL_BANNER: usize = 586;
    pub const LIGHT_GRAY_WALL_BANNER: usize = 587;
    pub const CYAN_WALL_BANNER: usize = 588;
    pub const PURPLE_WALL_BANNER: usize = 589;
    pub const BLUE_WALL_BANNER: usize = 590;
    pub const BROWN_WALL_BANNER: usize = 591;
    pub const GREEN_WALL_BANNER: usize = 592;
    pub const RED_WALL_BANNER: usize = 593;
    pub const BLACK_WALL_BANNER: usize = 594;
    pub const RED_SANDSTONE: usize = 595;
    pub const CHISELED_RED_SANDSTONE: usize = 596;
    pub const CUT_RED_SANDSTONE: usize = 597;
    pub const RED_SANDSTONE_STAIRS: usize = 598;
    pub const OAK_SLAB: usize = 599;
    pub const SPRUCE_SLAB: usize = 600;
    pub const BIRCH_SLAB: usize = 601;
    pub const JUNGLE_SLAB: usize = 602;
    pub const ACACIA_SLAB: usize = 603;
    pub const CHERRY_SLAB: usize = 604;
    pub const DARK_OAK_SLAB: usize = 605;
    pub const PALE_OAK_SLAB: usize = 606;
    pub const MANGROVE_SLAB: usize = 607;
    pub const BAMBOO_SLAB: usize = 608;
    pub const BAMBOO_MOSAIC_SLAB: usize = 609;
    pub const STONE_SLAB: usize = 610;
    pub const SMOOTH_STONE_SLAB: usize = 611;
    pub const SANDSTONE_SLAB: usize = 612;
    pub const CUT_SANDSTONE_SLAB: usize = 613;
    pub const PETRIFIED_OAK_SLAB: usize = 614;
    pub const COBBLESTONE_SLAB: usize = 615;
    pub const BRICK_SLAB: usize = 616;
    pub const STONE_BRICK_SLAB: usize = 617;
    pub const MUD_BRICK_SLAB: usize = 618;
    pub const NETHER_BRICK_SLAB: usize = 619;
    pub const QUARTZ_SLAB: usize = 620;
    pub const RED_SANDSTONE_SLAB: usize = 621;
    pub const CUT_RED_SANDSTONE_SLAB: usize = 622;
    pub const PURPUR_SLAB: usize = 623;
    pub const SMOOTH_STONE: usize = 624;
    pub const SMOOTH_SANDSTONE: usize = 625;
    pub const SMOOTH_QUARTZ: usize = 626;
    pub const SMOOTH_RED_SANDSTONE: usize = 627;
    pub const SPRUCE_FENCE_GATE: usize = 628;
    pub const BIRCH_FENCE_GATE: usize = 629;
    pub const JUNGLE_FENCE_GATE: usize = 630;
    pub const ACACIA_FENCE_GATE: usize = 631;
    pub const CHERRY_FENCE_GATE: usize = 632;
    pub const DARK_OAK_FENCE_GATE: usize = 633;
    pub const PALE_OAK_FENCE_GATE: usize = 634;
    pub const MANGROVE_FENCE_GATE: usize = 635;
    pub const BAMBOO_FENCE_GATE: usize = 636;
    pub const SPRUCE_FENCE: usize = 637;
    pub const BIRCH_FENCE: usize = 638;
    pub const JUNGLE_FENCE: usize = 639;
    pub const ACACIA_FENCE: usize = 640;
    pub const CHERRY_FENCE: usize = 641;
    pub const DARK_OAK_FENCE: usize = 642;
    pub const PALE_OAK_FENCE: usize = 643;
    pub const MANGROVE_FENCE: usize = 644;
    pub const BAMBOO_FENCE: usize = 645;
    pub const SPRUCE_DOOR: usize = 646;
    pub const BIRCH_DOOR: usize = 647;
    pub const JUNGLE_DOOR: usize = 648;
    pub const ACACIA_DOOR: usize = 649;
    pub const CHERRY_DOOR: usize = 650;
    pub const DARK_OAK_DOOR: usize = 651;
    pub const PALE_OAK_DOOR: usize = 652;
    pub const MANGROVE_DOOR: usize = 653;
    pub const BAMBOO_DOOR: usize = 654;
    pub const END_ROD: usize = 655;
    pub const CHORUS_PLANT: usize = 656;
    pub const CHORUS_FLOWER: usize = 657;
    pub const PURPUR_BLOCK: usize = 658;
    pub const PURPUR_PILLAR: usize = 659;
    pub const PURPUR_STAIRS: usize = 660;
    pub const END_STONE_BRICKS: usize = 661;
    pub const TORCHFLOWER_CROP: usize = 662;
    pub const PITCHER_CROP: usize = 663;
    pub const PITCHER_PLANT: usize = 664;
    pub const BEETROOTS: usize = 665;
    pub const DIRT_PATH: usize = 666;
    pub const END_GATEWAY: usize = 667;
    pub const REPEATING_COMMAND_BLOCK: usize = 668;
    pub const CHAIN_COMMAND_BLOCK: usize = 669;
    pub const FROSTED_ICE: usize = 670;
    pub const MAGMA_BLOCK: usize = 671;
    pub const NETHER_WART_BLOCK: usize = 672;
    pub const RED_NETHER_BRICKS: usize = 673;
    pub const BONE_BLOCK: usize = 674;
    pub const STRUCTURE_VOID: usize = 675;
    pub const OBSERVER: usize = 676;
    pub const SHULKER_BOX: usize = 677;
    pub const WHITE_SHULKER_BOX: usize = 678;
    pub const ORANGE_SHULKER_BOX: usize = 679;
    pub const MAGENTA_SHULKER_BOX: usize = 680;
    pub const LIGHT_BLUE_SHULKER_BOX: usize = 681;
    pub const YELLOW_SHULKER_BOX: usize = 682;
    pub const LIME_SHULKER_BOX: usize = 683;
    pub const PINK_SHULKER_BOX: usize = 684;
    pub const GRAY_SHULKER_BOX: usize = 685;
    pub const LIGHT_GRAY_SHULKER_BOX: usize = 686;
    pub const CYAN_SHULKER_BOX: usize = 687;
    pub const PURPLE_SHULKER_BOX: usize = 688;
    pub const BLUE_SHULKER_BOX: usize = 689;
    pub const BROWN_SHULKER_BOX: usize = 690;
    pub const GREEN_SHULKER_BOX: usize = 691;
    pub const RED_SHULKER_BOX: usize = 692;
    pub const BLACK_SHULKER_BOX: usize = 693;
    pub const WHITE_GLAZED_TERRACOTTA: usize = 694;
    pub const ORANGE_GLAZED_TERRACOTTA: usize = 695;
    pub const MAGENTA_GLAZED_TERRACOTTA: usize = 696;
    pub const LIGHT_BLUE_GLAZED_TERRACOTTA: usize = 697;
    pub const YELLOW_GLAZED_TERRACOTTA: usize = 698;
    pub const LIME_GLAZED_TERRACOTTA: usize = 699;
    pub const PINK_GLAZED_TERRACOTTA: usize = 700;
    pub const GRAY_GLAZED_TERRACOTTA: usize = 701;
    pub const LIGHT_GRAY_GLAZED_TERRACOTTA: usize = 702;
    pub const CYAN_GLAZED_TERRACOTTA: usize = 703;
    pub const PURPLE_GLAZED_TERRACOTTA: usize = 704;
    pub const BLUE_GLAZED_TERRACOTTA: usize = 705;
    pub const BROWN_GLAZED_TERRACOTTA: usize = 706;
    pub const GREEN_GLAZED_TERRACOTTA: usize = 707;
    pub const RED_GLAZED_TERRACOTTA: usize = 708;
    pub const BLACK_GLAZED_TERRACOTTA: usize = 709;
    pub const WHITE_CONCRETE: usize = 710;
    pub const ORANGE_CONCRETE: usize = 711;
    pub const MAGENTA_CONCRETE: usize = 712;
    pub const LIGHT_BLUE_CONCRETE: usize = 713;
    pub const YELLOW_CONCRETE: usize = 714;
    pub const LIME_CONCRETE: usize = 715;
    pub const PINK_CONCRETE: usize = 716;
    pub const GRAY_CONCRETE: usize = 717;
    pub const LIGHT_GRAY_CONCRETE: usize = 718;
    pub const CYAN_CONCRETE: usize = 719;
    pub const PURPLE_CONCRETE: usize = 720;
    pub const BLUE_CONCRETE: usize = 721;
    pub const BROWN_CONCRETE: usize = 722;
    pub const GREEN_CONCRETE: usize = 723;
    pub const RED_CONCRETE: usize = 724;
    pub const BLACK_CONCRETE: usize = 725;
    pub const WHITE_CONCRETE_POWDER: usize = 726;
    pub const ORANGE_CONCRETE_POWDER: usize = 727;
    pub const MAGENTA_CONCRETE_POWDER: usize = 728;
    pub const LIGHT_BLUE_CONCRETE_POWDER: usize = 729;
    pub const YELLOW_CONCRETE_POWDER: usize = 730;
    pub const LIME_CONCRETE_POWDER: usize = 731;
    pub const PINK_CONCRETE_POWDER: usize = 732;
    pub const GRAY_CONCRETE_POWDER: usize = 733;
    pub const LIGHT_GRAY_CONCRETE_POWDER: usize = 734;
    pub const CYAN_CONCRETE_POWDER: usize = 735;
    pub const PURPLE_CONCRETE_POWDER: usize = 736;
    pub const BLUE_CONCRETE_POWDER: usize = 737;
    pub const BROWN_CONCRETE_POWDER: usize = 738;
    pub const GREEN_CONCRETE_POWDER: usize = 739;
    pub const RED_CONCRETE_POWDER: usize = 740;
    pub const BLACK_CONCRETE_POWDER: usize = 741;
    pub const KELP: usize = 742;
    pub const KELP_PLANT: usize = 743;
    pub const DRIED_KELP_BLOCK: usize = 744;
    pub const TURTLE_EGG: usize = 745;
    pub const SNIFFER_EGG: usize = 746;
    pub const DRIED_GHAST: usize = 747;
    pub const DEAD_TUBE_CORAL_BLOCK: usize = 748;
    pub const DEAD_BRAIN_CORAL_BLOCK: usize = 749;
    pub const DEAD_BUBBLE_CORAL_BLOCK: usize = 750;
    pub const DEAD_FIRE_CORAL_BLOCK: usize = 751;
    pub const DEAD_HORN_CORAL_BLOCK: usize = 752;
    pub const TUBE_CORAL_BLOCK: usize = 753;
    pub const BRAIN_CORAL_BLOCK: usize = 754;
    pub const BUBBLE_CORAL_BLOCK: usize = 755;
    pub const FIRE_CORAL_BLOCK: usize = 756;
    pub const HORN_CORAL_BLOCK: usize = 757;
    pub const DEAD_TUBE_CORAL: usize = 758;
    pub const DEAD_BRAIN_CORAL: usize = 759;
    pub const DEAD_BUBBLE_CORAL: usize = 760;
    pub const DEAD_FIRE_CORAL: usize = 761;
    pub const DEAD_HORN_CORAL: usize = 762;
    pub const TUBE_CORAL: usize = 763;
    pub const BRAIN_CORAL: usize = 764;
    pub const BUBBLE_CORAL: usize = 765;
    pub const FIRE_CORAL: usize = 766;
    pub const HORN_CORAL: usize = 767;
    pub const DEAD_TUBE_CORAL_FAN: usize = 768;
    pub const DEAD_BRAIN_CORAL_FAN: usize = 769;
    pub const DEAD_BUBBLE_CORAL_FAN: usize = 770;
    pub const DEAD_FIRE_CORAL_FAN: usize = 771;
    pub const DEAD_HORN_CORAL_FAN: usize = 772;
    pub const TUBE_CORAL_FAN: usize = 773;
    pub const BRAIN_CORAL_FAN: usize = 774;
    pub const BUBBLE_CORAL_FAN: usize = 775;
    pub const FIRE_CORAL_FAN: usize = 776;
    pub const HORN_CORAL_FAN: usize = 777;
    pub const DEAD_TUBE_CORAL_WALL_FAN: usize = 778;
    pub const DEAD_BRAIN_CORAL_WALL_FAN: usize = 779;
    pub const DEAD_BUBBLE_CORAL_WALL_FAN: usize = 780;
    pub const DEAD_FIRE_CORAL_WALL_FAN: usize = 781;
    pub const DEAD_HORN_CORAL_WALL_FAN: usize = 782;
    pub const TUBE_CORAL_WALL_FAN: usize = 783;
    pub const BRAIN_CORAL_WALL_FAN: usize = 784;
    pub const BUBBLE_CORAL_WALL_FAN: usize = 785;
    pub const FIRE_CORAL_WALL_FAN: usize = 786;
    pub const HORN_CORAL_WALL_FAN: usize = 787;
    pub const SEA_PICKLE: usize = 788;
    pub const BLUE_ICE: usize = 789;
    pub const CONDUIT: usize = 790;
    pub const BAMBOO_SAPLING: usize = 791;
    pub const BAMBOO: usize = 792;
    pub const POTTED_BAMBOO: usize = 793;
    pub const VOID_AIR: usize = 794;
    pub const CAVE_AIR: usize = 795;
    pub const BUBBLE_COLUMN: usize = 796;
    pub const POLISHED_GRANITE_STAIRS: usize = 797;
    pub const SMOOTH_RED_SANDSTONE_STAIRS: usize = 798;
    pub const MOSSY_STONE_BRICK_STAIRS: usize = 799;
    pub const POLISHED_DIORITE_STAIRS: usize = 800;
    pub const MOSSY_COBBLESTONE_STAIRS: usize = 801;
    pub const END_STONE_BRICK_STAIRS: usize = 802;
    pub const STONE_STAIRS: usize = 803;
    pub const SMOOTH_SANDSTONE_STAIRS: usize = 804;
    pub const SMOOTH_QUARTZ_STAIRS: usize = 805;
    pub const GRANITE_STAIRS: usize = 806;
    pub const ANDESITE_STAIRS: usize = 807;
    pub const RED_NETHER_BRICK_STAIRS: usize = 808;
    pub const POLISHED_ANDESITE_STAIRS: usize = 809;
    pub const DIORITE_STAIRS: usize = 810;
    pub const POLISHED_GRANITE_SLAB: usize = 811;
    pub const SMOOTH_RED_SANDSTONE_SLAB: usize = 812;
    pub const MOSSY_STONE_BRICK_SLAB: usize = 813;
    pub const POLISHED_DIORITE_SLAB: usize = 814;
    pub const MOSSY_COBBLESTONE_SLAB: usize = 815;
    pub const END_STONE_BRICK_SLAB: usize = 816;
    pub const SMOOTH_SANDSTONE_SLAB: usize = 817;
    pub const SMOOTH_QUARTZ_SLAB: usize = 818;
    pub const GRANITE_SLAB: usize = 819;
    pub const ANDESITE_SLAB: usize = 820;
    pub const RED_NETHER_BRICK_SLAB: usize = 821;
    pub const POLISHED_ANDESITE_SLAB: usize = 822;
    pub const DIORITE_SLAB: usize = 823;
    pub const BRICK_WALL: usize = 824;
    pub const PRISMARINE_WALL: usize = 825;
    pub const RED_SANDSTONE_WALL: usize = 826;
    pub const MOSSY_STONE_BRICK_WALL: usize = 827;
    pub const GRANITE_WALL: usize = 828;
    pub const STONE_BRICK_WALL: usize = 829;
    pub const MUD_BRICK_WALL: usize = 830;
    pub const NETHER_BRICK_WALL: usize = 831;
    pub const ANDESITE_WALL: usize = 832;
    pub const RED_NETHER_BRICK_WALL: usize = 833;
    pub const SANDSTONE_WALL: usize = 834;
    pub const END_STONE_BRICK_WALL: usize = 835;
    pub const DIORITE_WALL: usize = 836;
    pub const SCAFFOLDING: usize = 837;
    pub const LOOM: usize = 838;
    pub const BARREL: usize = 839;
    pub const SMOKER: usize = 840;
    pub const BLAST_FURNACE: usize = 841;
    pub const CARTOGRAPHY_TABLE: usize = 842;
    pub const FLETCHING_TABLE: usize = 843;
    pub const GRINDSTONE: usize = 844;
    pub const LECTERN: usize = 845;
    pub const SMITHING_TABLE: usize = 846;
    pub const STONECUTTER: usize = 847;
    pub const BELL: usize = 848;
    pub const LANTERN: usize = 849;
    pub const SOUL_LANTERN: usize = 850;
    pub const COPPER_LANTERN: usize = 851;
    pub const EXPOSED_COPPER_LANTERN: usize = 852;
    pub const WEATHERED_COPPER_LANTERN: usize = 853;
    pub const OXIDIZED_COPPER_LANTERN: usize = 854;
    pub const WAXED_COPPER_LANTERN: usize = 855;
    pub const WAXED_EXPOSED_COPPER_LANTERN: usize = 856;
    pub const WAXED_WEATHERED_COPPER_LANTERN: usize = 857;
    pub const WAXED_OXIDIZED_COPPER_LANTERN: usize = 858;
    pub const CAMPFIRE: usize = 859;
    pub const SOUL_CAMPFIRE: usize = 860;
    pub const SWEET_BERRY_BUSH: usize = 861;
    pub const WARPED_STEM: usize = 862;
    pub const STRIPPED_WARPED_STEM: usize = 863;
    pub const WARPED_HYPHAE: usize = 864;
    pub const STRIPPED_WARPED_HYPHAE: usize = 865;
    pub const WARPED_NYLIUM: usize = 866;
    pub const WARPED_FUNGUS: usize = 867;
    pub const WARPED_WART_BLOCK: usize = 868;
    pub const WARPED_ROOTS: usize = 869;
    pub const NETHER_SPROUTS: usize = 870;
    pub const CRIMSON_STEM: usize = 871;
    pub const STRIPPED_CRIMSON_STEM: usize = 872;
    pub const CRIMSON_HYPHAE: usize = 873;
    pub const STRIPPED_CRIMSON_HYPHAE: usize = 874;
    pub const CRIMSON_NYLIUM: usize = 875;
    pub const CRIMSON_FUNGUS: usize = 876;
    pub const SHROOMLIGHT: usize = 877;
    pub const WEEPING_VINES: usize = 878;
    pub const WEEPING_VINES_PLANT: usize = 879;
    pub const TWISTING_VINES: usize = 880;
    pub const TWISTING_VINES_PLANT: usize = 881;
    pub const CRIMSON_ROOTS: usize = 882;
    pub const CRIMSON_PLANKS: usize = 883;
    pub const WARPED_PLANKS: usize = 884;
    pub const CRIMSON_SLAB: usize = 885;
    pub const WARPED_SLAB: usize = 886;
    pub const CRIMSON_PRESSURE_PLATE: usize = 887;
    pub const WARPED_PRESSURE_PLATE: usize = 888;
    pub const CRIMSON_FENCE: usize = 889;
    pub const WARPED_FENCE: usize = 890;
    pub const CRIMSON_TRAPDOOR: usize = 891;
    pub const WARPED_TRAPDOOR: usize = 892;
    pub const CRIMSON_FENCE_GATE: usize = 893;
    pub const WARPED_FENCE_GATE: usize = 894;
    pub const CRIMSON_STAIRS: usize = 895;
    pub const WARPED_STAIRS: usize = 896;
    pub const CRIMSON_BUTTON: usize = 897;
    pub const WARPED_BUTTON: usize = 898;
    pub const CRIMSON_DOOR: usize = 899;
    pub const WARPED_DOOR: usize = 900;
    pub const CRIMSON_SIGN: usize = 901;
    pub const WARPED_SIGN: usize = 902;
    pub const CRIMSON_WALL_SIGN: usize = 903;
    pub const WARPED_WALL_SIGN: usize = 904;
    pub const STRUCTURE_BLOCK: usize = 905;
    pub const JIGSAW: usize = 906;
    pub const TEST_BLOCK: usize = 907;
    pub const TEST_INSTANCE_BLOCK: usize = 908;
    pub const COMPOSTER: usize = 909;
    pub const TARGET: usize = 910;
    pub const BEE_NEST: usize = 911;
    pub const BEEHIVE: usize = 912;
    pub const HONEY_BLOCK: usize = 913;
    pub const HONEYCOMB_BLOCK: usize = 914;
    pub const NETHERITE_BLOCK: usize = 915;
    pub const ANCIENT_DEBRIS: usize = 916;
    pub const CRYING_OBSIDIAN: usize = 917;
    pub const RESPAWN_ANCHOR: usize = 918;
    pub const POTTED_CRIMSON_FUNGUS: usize = 919;
    pub const POTTED_WARPED_FUNGUS: usize = 920;
    pub const POTTED_CRIMSON_ROOTS: usize = 921;
    pub const POTTED_WARPED_ROOTS: usize = 922;
    pub const LODESTONE: usize = 923;
    pub const BLACKSTONE: usize = 924;
    pub const BLACKSTONE_STAIRS: usize = 925;
    pub const BLACKSTONE_WALL: usize = 926;
    pub const BLACKSTONE_SLAB: usize = 927;
    pub const POLISHED_BLACKSTONE: usize = 928;
    pub const POLISHED_BLACKSTONE_BRICKS: usize = 929;
    pub const CRACKED_POLISHED_BLACKSTONE_BRICKS: usize = 930;
    pub const CHISELED_POLISHED_BLACKSTONE: usize = 931;
    pub const POLISHED_BLACKSTONE_BRICK_SLAB: usize = 932;
    pub const POLISHED_BLACKSTONE_BRICK_STAIRS: usize = 933;
    pub const POLISHED_BLACKSTONE_BRICK_WALL: usize = 934;
    pub const GILDED_BLACKSTONE: usize = 935;
    pub const POLISHED_BLACKSTONE_STAIRS: usize = 936;
    pub const POLISHED_BLACKSTONE_SLAB: usize = 937;
    pub const POLISHED_BLACKSTONE_PRESSURE_PLATE: usize = 938;
    pub const POLISHED_BLACKSTONE_BUTTON: usize = 939;
    pub const POLISHED_BLACKSTONE_WALL: usize = 940;
    pub const CHISELED_NETHER_BRICKS: usize = 941;
    pub const CRACKED_NETHER_BRICKS: usize = 942;
    pub const QUARTZ_BRICKS: usize = 943;
    pub const CANDLE: usize = 944;
    pub const WHITE_CANDLE: usize = 945;
    pub const ORANGE_CANDLE: usize = 946;
    pub const MAGENTA_CANDLE: usize = 947;
    pub const LIGHT_BLUE_CANDLE: usize = 948;
    pub const YELLOW_CANDLE: usize = 949;
    pub const LIME_CANDLE: usize = 950;
    pub const PINK_CANDLE: usize = 951;
    pub const GRAY_CANDLE: usize = 952;
    pub const LIGHT_GRAY_CANDLE: usize = 953;
    pub const CYAN_CANDLE: usize = 954;
    pub const PURPLE_CANDLE: usize = 955;
    pub const BLUE_CANDLE: usize = 956;
    pub const BROWN_CANDLE: usize = 957;
    pub const GREEN_CANDLE: usize = 958;
    pub const RED_CANDLE: usize = 959;
    pub const BLACK_CANDLE: usize = 960;
    pub const CANDLE_CAKE: usize = 961;
    pub const WHITE_CANDLE_CAKE: usize = 962;
    pub const ORANGE_CANDLE_CAKE: usize = 963;
    pub const MAGENTA_CANDLE_CAKE: usize = 964;
    pub const LIGHT_BLUE_CANDLE_CAKE: usize = 965;
    pub const YELLOW_CANDLE_CAKE: usize = 966;
    pub const LIME_CANDLE_CAKE: usize = 967;
    pub const PINK_CANDLE_CAKE: usize = 968;
    pub const GRAY_CANDLE_CAKE: usize = 969;
    pub const LIGHT_GRAY_CANDLE_CAKE: usize = 970;
    pub const CYAN_CANDLE_CAKE: usize = 971;
    pub const PURPLE_CANDLE_CAKE: usize = 972;
    pub const BLUE_CANDLE_CAKE: usize = 973;
    pub const BROWN_CANDLE_CAKE: usize = 974;
    pub const GREEN_CANDLE_CAKE: usize = 975;
    pub const RED_CANDLE_CAKE: usize = 976;
    pub const BLACK_CANDLE_CAKE: usize = 977;
    pub const AMETHYST_BLOCK: usize = 978;
    pub const BUDDING_AMETHYST: usize = 979;
    pub const AMETHYST_CLUSTER: usize = 980;
    pub const LARGE_AMETHYST_BUD: usize = 981;
    pub const MEDIUM_AMETHYST_BUD: usize = 982;
    pub const SMALL_AMETHYST_BUD: usize = 983;
    pub const TUFF: usize = 984;
    pub const TUFF_SLAB: usize = 985;
    pub const TUFF_STAIRS: usize = 986;
    pub const TUFF_WALL: usize = 987;
    pub const POLISHED_TUFF: usize = 988;
    pub const POLISHED_TUFF_SLAB: usize = 989;
    pub const POLISHED_TUFF_STAIRS: usize = 990;
    pub const POLISHED_TUFF_WALL: usize = 991;
    pub const CHISELED_TUFF: usize = 992;
    pub const TUFF_BRICKS: usize = 993;
    pub const TUFF_BRICK_SLAB: usize = 994;
    pub const TUFF_BRICK_STAIRS: usize = 995;
    pub const TUFF_BRICK_WALL: usize = 996;
    pub const CHISELED_TUFF_BRICKS: usize = 997;
    pub const SULFUR: usize = 998;
    pub const POTENT_SULFUR: usize = 999;
    pub const SULFUR_SLAB: usize = 1000;
    pub const SULFUR_STAIRS: usize = 1001;
    pub const SULFUR_WALL: usize = 1002;
    pub const POLISHED_SULFUR: usize = 1003;
    pub const POLISHED_SULFUR_SLAB: usize = 1004;
    pub const POLISHED_SULFUR_STAIRS: usize = 1005;
    pub const POLISHED_SULFUR_WALL: usize = 1006;
    pub const SULFUR_BRICKS: usize = 1007;
    pub const SULFUR_BRICK_SLAB: usize = 1008;
    pub const SULFUR_BRICK_STAIRS: usize = 1009;
    pub const SULFUR_BRICK_WALL: usize = 1010;
    pub const CHISELED_SULFUR: usize = 1011;
    pub const CINNABAR: usize = 1012;
    pub const CINNABAR_SLAB: usize = 1013;
    pub const CINNABAR_STAIRS: usize = 1014;
    pub const CINNABAR_WALL: usize = 1015;
    pub const POLISHED_CINNABAR: usize = 1016;
    pub const POLISHED_CINNABAR_SLAB: usize = 1017;
    pub const POLISHED_CINNABAR_STAIRS: usize = 1018;
    pub const POLISHED_CINNABAR_WALL: usize = 1019;
    pub const CINNABAR_BRICKS: usize = 1020;
    pub const CINNABAR_BRICK_SLAB: usize = 1021;
    pub const CINNABAR_BRICK_STAIRS: usize = 1022;
    pub const CINNABAR_BRICK_WALL: usize = 1023;
    pub const CHISELED_CINNABAR: usize = 1024;
    pub const CALCITE: usize = 1025;
    pub const TINTED_GLASS: usize = 1026;
    pub const POWDER_SNOW: usize = 1027;
    pub const SCULK_SENSOR: usize = 1028;
    pub const CALIBRATED_SCULK_SENSOR: usize = 1029;
    pub const SCULK: usize = 1030;
    pub const SCULK_VEIN: usize = 1031;
    pub const SCULK_CATALYST: usize = 1032;
    pub const SCULK_SHRIEKER: usize = 1033;
    pub const COPPER_BLOCK: usize = 1034;
    pub const EXPOSED_COPPER: usize = 1035;
    pub const WEATHERED_COPPER: usize = 1036;
    pub const OXIDIZED_COPPER: usize = 1037;
    pub const WAXED_COPPER_BLOCK: usize = 1038;
    pub const WAXED_EXPOSED_COPPER: usize = 1039;
    pub const WAXED_WEATHERED_COPPER: usize = 1040;
    pub const WAXED_OXIDIZED_COPPER: usize = 1041;
    pub const COPPER_ORE: usize = 1042;
    pub const DEEPSLATE_COPPER_ORE: usize = 1043;
    pub const CUT_COPPER: usize = 1044;
    pub const EXPOSED_CUT_COPPER: usize = 1045;
    pub const WEATHERED_CUT_COPPER: usize = 1046;
    pub const OXIDIZED_CUT_COPPER: usize = 1047;
    pub const WAXED_CUT_COPPER: usize = 1048;
    pub const WAXED_EXPOSED_CUT_COPPER: usize = 1049;
    pub const WAXED_WEATHERED_CUT_COPPER: usize = 1050;
    pub const WAXED_OXIDIZED_CUT_COPPER: usize = 1051;
    pub const CHISELED_COPPER: usize = 1052;
    pub const EXPOSED_CHISELED_COPPER: usize = 1053;
    pub const WEATHERED_CHISELED_COPPER: usize = 1054;
    pub const OXIDIZED_CHISELED_COPPER: usize = 1055;
    pub const WAXED_CHISELED_COPPER: usize = 1056;
    pub const WAXED_EXPOSED_CHISELED_COPPER: usize = 1057;
    pub const WAXED_WEATHERED_CHISELED_COPPER: usize = 1058;
    pub const WAXED_OXIDIZED_CHISELED_COPPER: usize = 1059;
    pub const CUT_COPPER_STAIRS: usize = 1060;
    pub const EXPOSED_CUT_COPPER_STAIRS: usize = 1061;
    pub const WEATHERED_CUT_COPPER_STAIRS: usize = 1062;
    pub const OXIDIZED_CUT_COPPER_STAIRS: usize = 1063;
    pub const WAXED_CUT_COPPER_STAIRS: usize = 1064;
    pub const WAXED_EXPOSED_CUT_COPPER_STAIRS: usize = 1065;
    pub const WAXED_WEATHERED_CUT_COPPER_STAIRS: usize = 1066;
    pub const WAXED_OXIDIZED_CUT_COPPER_STAIRS: usize = 1067;
    pub const CUT_COPPER_SLAB: usize = 1068;
    pub const EXPOSED_CUT_COPPER_SLAB: usize = 1069;
    pub const WEATHERED_CUT_COPPER_SLAB: usize = 1070;
    pub const OXIDIZED_CUT_COPPER_SLAB: usize = 1071;
    pub const WAXED_CUT_COPPER_SLAB: usize = 1072;
    pub const WAXED_EXPOSED_CUT_COPPER_SLAB: usize = 1073;
    pub const WAXED_WEATHERED_CUT_COPPER_SLAB: usize = 1074;
    pub const WAXED_OXIDIZED_CUT_COPPER_SLAB: usize = 1075;
    pub const COPPER_DOOR: usize = 1076;
    pub const EXPOSED_COPPER_DOOR: usize = 1077;
    pub const WEATHERED_COPPER_DOOR: usize = 1078;
    pub const OXIDIZED_COPPER_DOOR: usize = 1079;
    pub const WAXED_COPPER_DOOR: usize = 1080;
    pub const WAXED_EXPOSED_COPPER_DOOR: usize = 1081;
    pub const WAXED_WEATHERED_COPPER_DOOR: usize = 1082;
    pub const WAXED_OXIDIZED_COPPER_DOOR: usize = 1083;
    pub const COPPER_TRAPDOOR: usize = 1084;
    pub const EXPOSED_COPPER_TRAPDOOR: usize = 1085;
    pub const WEATHERED_COPPER_TRAPDOOR: usize = 1086;
    pub const OXIDIZED_COPPER_TRAPDOOR: usize = 1087;
    pub const WAXED_COPPER_TRAPDOOR: usize = 1088;
    pub const WAXED_EXPOSED_COPPER_TRAPDOOR: usize = 1089;
    pub const WAXED_WEATHERED_COPPER_TRAPDOOR: usize = 1090;
    pub const WAXED_OXIDIZED_COPPER_TRAPDOOR: usize = 1091;
    pub const COPPER_GRATE: usize = 1092;
    pub const EXPOSED_COPPER_GRATE: usize = 1093;
    pub const WEATHERED_COPPER_GRATE: usize = 1094;
    pub const OXIDIZED_COPPER_GRATE: usize = 1095;
    pub const WAXED_COPPER_GRATE: usize = 1096;
    pub const WAXED_EXPOSED_COPPER_GRATE: usize = 1097;
    pub const WAXED_WEATHERED_COPPER_GRATE: usize = 1098;
    pub const WAXED_OXIDIZED_COPPER_GRATE: usize = 1099;
    pub const COPPER_BULB: usize = 1100;
    pub const EXPOSED_COPPER_BULB: usize = 1101;
    pub const WEATHERED_COPPER_BULB: usize = 1102;
    pub const OXIDIZED_COPPER_BULB: usize = 1103;
    pub const WAXED_COPPER_BULB: usize = 1104;
    pub const WAXED_EXPOSED_COPPER_BULB: usize = 1105;
    pub const WAXED_WEATHERED_COPPER_BULB: usize = 1106;
    pub const WAXED_OXIDIZED_COPPER_BULB: usize = 1107;
    pub const COPPER_CHEST: usize = 1108;
    pub const EXPOSED_COPPER_CHEST: usize = 1109;
    pub const WEATHERED_COPPER_CHEST: usize = 1110;
    pub const OXIDIZED_COPPER_CHEST: usize = 1111;
    pub const WAXED_COPPER_CHEST: usize = 1112;
    pub const WAXED_EXPOSED_COPPER_CHEST: usize = 1113;
    pub const WAXED_WEATHERED_COPPER_CHEST: usize = 1114;
    pub const WAXED_OXIDIZED_COPPER_CHEST: usize = 1115;
    pub const COPPER_GOLEM_STATUE: usize = 1116;
    pub const EXPOSED_COPPER_GOLEM_STATUE: usize = 1117;
    pub const WEATHERED_COPPER_GOLEM_STATUE: usize = 1118;
    pub const OXIDIZED_COPPER_GOLEM_STATUE: usize = 1119;
    pub const WAXED_COPPER_GOLEM_STATUE: usize = 1120;
    pub const WAXED_EXPOSED_COPPER_GOLEM_STATUE: usize = 1121;
    pub const WAXED_WEATHERED_COPPER_GOLEM_STATUE: usize = 1122;
    pub const WAXED_OXIDIZED_COPPER_GOLEM_STATUE: usize = 1123;
    pub const LIGHTNING_ROD: usize = 1124;
    pub const EXPOSED_LIGHTNING_ROD: usize = 1125;
    pub const WEATHERED_LIGHTNING_ROD: usize = 1126;
    pub const OXIDIZED_LIGHTNING_ROD: usize = 1127;
    pub const WAXED_LIGHTNING_ROD: usize = 1128;
    pub const WAXED_EXPOSED_LIGHTNING_ROD: usize = 1129;
    pub const WAXED_WEATHERED_LIGHTNING_ROD: usize = 1130;
    pub const WAXED_OXIDIZED_LIGHTNING_ROD: usize = 1131;
    pub const DRIPSTONE_BLOCK: usize = 1132;
    pub const POINTED_DRIPSTONE: usize = 1133;
    pub const SULFUR_SPIKE: usize = 1134;
    pub const CAVE_VINES: usize = 1135;
    pub const CAVE_VINES_PLANT: usize = 1136;
    pub const SPORE_BLOSSOM: usize = 1137;
    pub const AZALEA: usize = 1138;
    pub const FLOWERING_AZALEA: usize = 1139;
    pub const MOSS_CARPET: usize = 1140;
    pub const PINK_PETALS: usize = 1141;
    pub const WILDFLOWERS: usize = 1142;
    pub const LEAF_LITTER: usize = 1143;
    pub const MOSS_BLOCK: usize = 1144;
    pub const BIG_DRIPLEAF: usize = 1145;
    pub const BIG_DRIPLEAF_STEM: usize = 1146;
    pub const SMALL_DRIPLEAF: usize = 1147;
    pub const HANGING_ROOTS: usize = 1148;
    pub const ROOTED_DIRT: usize = 1149;
    pub const MUD: usize = 1150;
    pub const DEEPSLATE: usize = 1151;
    pub const COBBLED_DEEPSLATE: usize = 1152;
    pub const COBBLED_DEEPSLATE_STAIRS: usize = 1153;
    pub const COBBLED_DEEPSLATE_SLAB: usize = 1154;
    pub const COBBLED_DEEPSLATE_WALL: usize = 1155;
    pub const POLISHED_DEEPSLATE: usize = 1156;
    pub const POLISHED_DEEPSLATE_STAIRS: usize = 1157;
    pub const POLISHED_DEEPSLATE_SLAB: usize = 1158;
    pub const POLISHED_DEEPSLATE_WALL: usize = 1159;
    pub const DEEPSLATE_TILES: usize = 1160;
    pub const DEEPSLATE_TILE_STAIRS: usize = 1161;
    pub const DEEPSLATE_TILE_SLAB: usize = 1162;
    pub const DEEPSLATE_TILE_WALL: usize = 1163;
    pub const DEEPSLATE_BRICKS: usize = 1164;
    pub const DEEPSLATE_BRICK_STAIRS: usize = 1165;
    pub const DEEPSLATE_BRICK_SLAB: usize = 1166;
    pub const DEEPSLATE_BRICK_WALL: usize = 1167;
    pub const CHISELED_DEEPSLATE: usize = 1168;
    pub const CRACKED_DEEPSLATE_BRICKS: usize = 1169;
    pub const CRACKED_DEEPSLATE_TILES: usize = 1170;
    pub const INFESTED_DEEPSLATE: usize = 1171;
    pub const SMOOTH_BASALT: usize = 1172;
    pub const RAW_IRON_BLOCK: usize = 1173;
    pub const RAW_COPPER_BLOCK: usize = 1174;
    pub const RAW_GOLD_BLOCK: usize = 1175;
    pub const POTTED_AZALEA_BUSH: usize = 1176;
    pub const POTTED_FLOWERING_AZALEA_BUSH: usize = 1177;
    pub const OCHRE_FROGLIGHT: usize = 1178;
    pub const VERDANT_FROGLIGHT: usize = 1179;
    pub const PEARLESCENT_FROGLIGHT: usize = 1180;
    pub const FROGSPAWN: usize = 1181;
    pub const REINFORCED_DEEPSLATE: usize = 1182;
    pub const DECORATED_POT: usize = 1183;
    pub const CRAFTER: usize = 1184;
    pub const TRIAL_SPAWNER: usize = 1185;
    pub const VAULT: usize = 1186;
    pub const HEAVY_CORE: usize = 1187;
    pub const PALE_MOSS_BLOCK: usize = 1188;
    pub const PALE_MOSS_CARPET: usize = 1189;
    pub const PALE_HANGING_MOSS: usize = 1190;
    pub const OPEN_EYEBLOSSOM: usize = 1191;
    pub const CLOSED_EYEBLOSSOM: usize = 1192;
    pub const POTTED_OPEN_EYEBLOSSOM: usize = 1193;
    pub const POTTED_CLOSED_EYEBLOSSOM: usize = 1194;
    pub const FIREFLY_BUSH: usize = 1195;
}
