use super::*;

impl GameMode {
    pub fn is_destroying(&self) -> bool {
        self.is_destroying
    }

    /// The blocks being broken since this was last asked, with how far (`Tutorial.onDestroyBlock`):
    /// 0 for a start, 1 for a finished or creative break, -1 for an abandoned one.
    pub fn take_destroyed(&mut self) -> Vec<(photon_core::ids::BlockStateId, f32)> {
        std::mem::take(&mut self.destroyed)
    }

    pub fn destroy_progress(&self) -> f32 {
        self.destroy_progress
    }

    pub fn destroy_ticks(&self) -> f32 {
        self.destroy_ticks
    }

    pub fn destroy_delay(&self) -> i32 {
        self.destroy_delay
    }

    pub fn miss_time(&self) -> i32 {
        self.miss_time
    }

    /// `getDestroyStage`: the crack from 0 to 9, or -1 before any progress.
    pub fn destroy_stage(&self) -> i32 {
        if self.destroy_progress > 0.0 {
            (self.destroy_progress * 10.0) as i32
        } else {
            -1
        }
    }

    /// `Minecraft.startAttack`; whether the attack ended the key's hold.
    pub(super) fn start_attack(
        &mut self,
        ctx: &mut Context<'_>,
        aim: Aim,
        out: &mut Vec<Outgoing>,
    ) -> bool {
        if self.miss_time > 0 {
            return false;
        }
        if ctx.mode == ClientPlayerGameMode::Spectator {
            // `spectate` of the entity aimed at, or `spectatorNoAction`.
            out.push(Outgoing::SpectatorAction {
                entity_id: match aim {
                    Aim::Entity { id, .. } => Some(id),
                    _ => None,
                },
            });
            return true;
        }
        if let Aim::Entity { id, .. } = aim {
            // `MultiPlayerGameMode.attack`: the server decides what it does.
            out.push(Outgoing::Attack { entity_id: id });
            ctx.player.reset_attack_strength();
            ctx.swing(out);
            return false;
        }
        let mut end_attack = false;
        let aimed_block = match aim {
            Aim::Block { x, y, z, face, .. } if !is_air(ctx.state_at((x, y, z))) => {
                Some((x, y, z, face))
            }
            _ => None,
        };
        if let Some((x, y, z, face)) = aimed_block {
            self.start_destroy_block(ctx, (x, y, z), face, out);
            if is_air(ctx.state_at((x, y, z))) {
                end_attack = true;
            }
        } else {
            if ctx.mode != ClientPlayerGameMode::Creative {
                self.miss_time = 10;
            }
            ctx.player.reset_attack_strength();
        }
        ctx.swing(out);
        end_attack
    }

    /// `Minecraft.continueAttack`.
    pub(super) fn continue_attack(
        &mut self,
        ctx: &mut Context<'_>,
        down: bool,
        aim: Aim,
        out: &mut Vec<Outgoing>,
    ) {
        if !down {
            self.miss_time = 0;
        }
        if self.miss_time > 0 {
            return;
        }
        match aim {
            Aim::Block { x, y, z, face, .. } if down => {
                if !is_air(ctx.state_at((x, y, z)))
                    && self.continue_destroy_block(ctx, (x, y, z), face, out)
                {
                    ctx.world.add_breaking_block_effect(
                        (x, y, z),
                        photon_core::direction::Direction::from_3d_data_value(i32::from(face)),
                    );
                    ctx.swing(out);
                }
            }
            _ => self.stop_destroy_block(ctx, out),
        }
    }

    pub(super) fn same_destroy_target(&self, ctx: &Context<'_>, pos: (i32, i32, i32)) -> bool {
        let same_item = match (&ctx.miner.main_hand, &self.destroying_item) {
            (None, None) => true,
            (Some(held), Some(destroying)) => held.is_same_item_same_components(destroying),
            _ => false,
        };
        pos == self.destroy_pos && same_item
    }

    pub(super) fn crack(&self, ctx: &mut Context<'_>) {
        let (x, y, z) = self.destroy_pos;
        let id = ctx.player.entity_id();
        ctx.world
            .destroy_block_progress(id, x, y, z, self.destroy_stage());
    }

    /// `startDestroyBlock`.
    pub(super) fn start_destroy_block(
        &mut self,
        ctx: &mut Context<'_>,
        pos: (i32, i32, i32),
        face: u8,
        out: &mut Vec<Outgoing>,
    ) -> bool {
        if ctx.block_action_restricted() || !ctx.within_border(pos) {
            return false;
        }
        if ctx.instabuild() {
            self.destroyed.push((ctx.state_at(pos), 1.0));
            self.predict(ctx, out, |_, ctx, sequence| {
                destroy_block(ctx, pos);
                Self::action(PlayerAction::StartDestroyBlock, pos, face, sequence)
            });
            self.destroy_delay = 5;
        } else if !self.is_destroying || !self.same_destroy_target(ctx, pos) {
            if self.is_destroying {
                out.push(Self::action(
                    PlayerAction::AbortDestroyBlock,
                    self.destroy_pos,
                    face,
                    0,
                ));
            }
            let state = ctx.state_at(pos);
            self.destroyed.push((state, 0.0));
            self.predict(ctx, out, |game_mode, ctx, sequence| {
                let not_air = !is_air(state);
                if not_air && ctx.miner.destroy_progress(ctx.world, ctx.player, state) >= 1.0 {
                    destroy_block(ctx, pos);
                } else {
                    game_mode.is_destroying = true;
                    game_mode.destroy_pos = pos;
                    game_mode.destroying_item = ctx.miner.main_hand.clone();
                    game_mode.destroy_progress = 0.0;
                    game_mode.destroy_ticks = 0.0;
                    game_mode.crack(ctx);
                }
                Self::action(PlayerAction::StartDestroyBlock, pos, face, sequence)
            });
        }
        true
    }

    /// `stopDestroyBlock`: letting go abandons the block.
    pub(super) fn stop_destroy_block(&mut self, ctx: &mut Context<'_>, out: &mut Vec<Outgoing>) {
        if !self.is_destroying {
            return;
        }
        self.destroyed.push((ctx.state_at(self.destroy_pos), -1.0));
        out.push(Self::action(
            PlayerAction::AbortDestroyBlock,
            self.destroy_pos,
            DIRECTION_DOWN,
            0,
        ));
        self.is_destroying = false;
        self.destroy_progress = 0.0;
        let (x, y, z) = self.destroy_pos;
        let id = ctx.player.entity_id();
        ctx.world.destroy_block_progress(id, x, y, z, -1);
        ctx.player.reset_attack_strength();
    }

    /// `continueDestroyBlock`.
    pub(super) fn continue_destroy_block(
        &mut self,
        ctx: &mut Context<'_>,
        pos: (i32, i32, i32),
        face: u8,
        out: &mut Vec<Outgoing>,
    ) -> bool {
        if self.destroy_delay > 0 {
            self.destroy_delay -= 1;
            return true;
        }
        if ctx.instabuild() && ctx.within_border(pos) {
            self.destroy_delay = 5;
            self.destroyed.push((ctx.state_at(pos), 1.0));
            self.predict(ctx, out, |_, ctx, sequence| {
                destroy_block(ctx, pos);
                Self::action(PlayerAction::StartDestroyBlock, pos, face, sequence)
            });
            return true;
        }
        if !self.same_destroy_target(ctx, pos) {
            return self.start_destroy_block(ctx, pos, face, out);
        }
        let state = ctx.state_at(pos);
        if is_air(state) {
            self.is_destroying = false;
            return false;
        }
        self.destroy_progress += ctx.miner.destroy_progress(ctx.world, ctx.player, state);
        if self.destroy_ticks % 4.0 == 0.0 {
            ctx.world.play_hit_sound(pos, state);
        }
        self.destroy_ticks += 1.0;
        self.destroyed
            .push((state, self.destroy_progress.clamp(0.0, 1.0)));
        if self.destroy_progress >= 1.0 {
            self.is_destroying = false;
            self.predict(ctx, out, |_, ctx, sequence| {
                destroy_block(ctx, pos);
                Self::action(PlayerAction::StopDestroyBlock, pos, face, sequence)
            });
            self.destroy_progress = 0.0;
            self.destroy_ticks = 0.0;
            self.destroy_delay = 5;
        }
        self.crack(ctx);
        true
    }
}

/// `destroyBlock`: the block becomes the fluid it held, as a prediction.
pub(super) fn destroy_block(ctx: &mut Context<'_>, pos: (i32, i32, i32)) -> bool {
    if ctx.block_action_restricted() {
        return false;
    }
    let old = ctx.state_at(pos);
    // `Item.canDestroyBlock`: some tools cannot break blocks in creative.
    let tool = ctx
        .miner
        .main_hand
        .as_ref()
        .and_then(|stack| crate::item::definition(stack.item_registry_id()))
        .and_then(|item| item.tool);
    if tool.is_some_and(|tool| !tool.can_destroy_blocks_in_creative) && ctx.instabuild() {
        return false;
    }
    if is_game_master_block(old) || is_air(old) {
        return false;
    }
    let fluid = photon_world::block::state_or_air(old).fluid.legacy_block();
    let position: Vec3 = ctx.player.position();
    let changed = ctx
        .world
        .set_block_with_updates(pos, fluid, 11, 512, position);
    if changed {
        // `Block.playerWillDestroy`: the break sound (and the particles).
        ctx.world.apply_level_event(2001, pos, old.0);
    }
    changed
}
