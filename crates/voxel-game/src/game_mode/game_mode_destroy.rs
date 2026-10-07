use super::*;

impl GameMode {
    pub fn is_destroying(&self) -> bool {
        self.is_destroying
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

    pub fn right_click_delay(&self) -> i32 {
        self.right_click_delay
    }

    /// One tick of the attack and use keys, in `Minecraft.tick`'s order:
    /// the right click delay counts down, then `handleKeybinds` spends the
    /// attack clicks, the use clicks, a held use key and a held attack key,
    /// then the miss wait counts down.
    pub fn keys_tick(&mut self, ctx: &mut Context<'_>, keys: Keys, aim: Aim) -> Vec<Outgoing> {
        let mut out = Vec::new();
        if self.right_click_delay > 0 {
            self.right_click_delay -= 1;
        }
        let mut instant_attack = false;
        for _ in 0..keys.attack_clicks {
            instant_attack |= self.start_attack(ctx, aim, &mut out);
        }
        for _ in 0..keys.use_clicks {
            self.start_use_item(ctx, aim, &mut out);
        }
        for _ in 0..keys.pick_clicks {
            Self::pick_block_or_entity(aim, keys.control_down, &mut out);
        }
        if keys.use_held && self.right_click_delay == 0 {
            self.start_use_item(ctx, aim, &mut out);
        }
        self.continue_attack(ctx, keys.attack_held && !instant_attack, aim, &mut out);
        if self.miss_time > 0 {
            self.miss_time -= 1;
        }
        out
    }

    /// `Minecraft.pickBlockOrEntity`: the server is asked for the aimed
    /// block's or entity's item; a miss asks for nothing.
    pub(super) fn pick_block_or_entity(aim: Aim, include_data: bool, out: &mut Vec<Outgoing>) {
        match aim {
            Aim::Block { x, y, z, .. } => out.push(Outgoing::PickItemFromBlock {
                x,
                y,
                z,
                include_data,
            }),
            Aim::Entity { id, .. } => out.push(Outgoing::PickItemFromEntity {
                entity_id: id,
                include_data,
            }),
            Aim::Miss => {}
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
        match aimed_block {
            Some((x, y, z, face)) => {
                self.start_destroy_block(ctx, (x, y, z), face, out);
                if is_air(ctx.state_at((x, y, z))) {
                    end_attack = true;
                }
            }
            None => {
                if ctx.mode != ClientPlayerGameMode::Creative {
                    self.miss_time = 10;
                }
                ctx.player.reset_attack_strength();
            }
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

    /// `startPrediction`: the action runs under a new sequence number, which
    /// the packet it returns carries.
    pub(super) fn predict(
        &mut self,
        ctx: &mut Context<'_>,
        out: &mut Vec<Outgoing>,
        action: impl FnOnce(&mut Self, &mut Context<'_>, i32) -> Outgoing,
    ) {
        let sequence = ctx.world.start_predicting();
        let packet = action(self, ctx, sequence);
        out.push(packet);
        ctx.world.stop_predicting();
    }

    pub(super) fn action(
        action: PlayerAction,
        (x, y, z): (i32, i32, i32),
        face: u8,
        sequence: i32,
    ) -> Outgoing {
        Outgoing::PlayerAction {
            action,
            x,
            y,
            z,
            face,
            sequence,
        }
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
