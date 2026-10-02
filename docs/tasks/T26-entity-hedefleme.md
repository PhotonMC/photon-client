# T26 — Entity hedefleme, `Attack`/`Interact`

**Amaç:** Crosshair'in baktığı entity'yi bul (blokla birlikte en yakın), sol tık `Attack`, sağ tık `Interact`, el sallama; `PickItemFromEntity` ve sohbet seçicisindeki bakılan varlık UUID önerisi (T15 notu).
**Bağımlılık:** T21. **Boy:** M.

## Oku
- Java: `.data/minecraft/source/net/minecraft/client/renderer/GameRenderer.java` (`pick`, `filterHitResult`: `grep -n "pick\|ProjectileUtil"`), `.data/minecraft/source/net/minecraft/world/entity/projectile/ProjectileUtil.java` (`getEntityHitResult`), `.data/minecraft/source/net/minecraft/client/Minecraft.java` (`startAttack`, `startUseItem`, `continueAttack`), `.data/minecraft/source/net/minecraft/client/multiplayer/MultiPlayerGameMode.java` (`attack`, `interact`, `interactAt`), `.data/minecraft/source/net/minecraft/network/protocol/game/{ServerboundAttackPacket,ServerboundInteractPacket,ServerboundSwingPacket}.java`.
- Rust: mevcut blok hedefleme `crates/voxel/src/controls.rs` (`break`/`place` yolu), `crates/voxel-game/src/attack.rs` (saldırı gücü/bekleme), docs `use-item-conformance.md` (kısa), `crates/voxel-world/src/lib.rs` entity AABB'leri (`ClientEntity.bounding_box`), `voxel_core::aabb` ray (`clip`).

## Yapılacaklar
1. Hedefleme: `entity_pick(eye, direction, reach)` — AABB `clip` + `pickRadius`, blok isabetiyle mesafe karşılaştırma (Java `GameRenderer.pick` sırası: blok → entity, entity daha yakınsa).
2. Paketler: `ServerboundAttack` (entity kimliği), `ServerboundInteract`/`InteractAt` (el, konum, sneaking), `Swing`; kaçırma/boş vuruş `Swing`.
3. Sol/sağ tık dalları `controls.rs`'de entity hedefi varsa onu kullanır; tahmin yok (sunucu otorite).
4. Hasar geri bildirimi (`HurtAnimation`, `DamageEvent`) sonraki görevlerde.

## Oracle
`entity_pick` operation'ı (`entity`/`movement` ailesi, Java `ProjectileUtil.getEntityHitResult` + `GameRenderer.pick` mantığı gerçek `Entity`'lerle kurulabiliyorsa; kurulamıyorsa `AABB.clip` parity'sine dayanıp Java kaynağından satır satır birim test — nedeni devir notuna).

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`. Canlı (`shot`): `summon pig` sonrası sol tık, sunucu logunda hasar.

## Kapsam dışı
Saldırı süpürme/özel silah mantığı, kalkan, binme.

## Belgeler
`docs/remaining-work.md` M4 madde 4; T15 notu kapanır.

## Devir notu
- Yapıldı: `voxel-world/pick.rs` (`ClientEntity::is_pickable/bounding_box/pick_radius`, `ClientWorld::entity_hit_result` = `ProjectileUtil.getEntityHitResult`), `voxel-game/pick.rs` (`LocalPlayer.pick`: blok/entity karşılaştırma, menzil süzme; menzil `Miner`'dan öznitelik değeri), `Aim::Entity`, `Outgoing::{Attack, Interact, SpectatorAction}` (+ izleyici boş vuruşta `spectator_action`, eskiden eksikti), paket kimlikleri 1/26/62, `view::pick` kaldırıldı.
- Veri: tür başına `isPickable`/`getPickRadius`/`canBePickedFromInside`/boyut(+bebek) tablosu oracle'dan (`movement/entity-pick-table`, `tools/generate/entity_picks.py`), her parity koşusunda karşılaştırılır. Kayıt: `docs/client-entity-aim-parity-conformance.md`.
- Oracle: `movement/entity-aim` 27 satır, gerçek `raycastHitResult` + `startAttack/startUseItem`, paketler bayt bayt. Canlı: `Viewer2` ile etiketli domuza sol tık, sunucuda Health 10 → 9.
- Sapma: `Player.interactOn` istemci tahmini yok (yenebilir eşyayla hayvana sağ tık ayrışabilir); SCALE/poz boyutları, marker armor stand, `ATTACK_RANGE`, `isHandsBusy` yok. Hasar geri bildirimi (`HurtAnimation`, `DamageEvent`) sonraki görevlerde.
- Açık: T26b (UUID önerisi, `PickItemFromEntity`).
