# T46 — Mob öznitelikleri (`UpdateAttributes`)

**Amaç:** Oyuncu dışı varlıkların özniteliklerini (can üstü, ölçek, hız) istemcide tutmak. Bugün demir golemin çatlak düzeyi can üstünü 100 sayıyor (`golems.rs`), ölçek hep bir.
**Bağımlılık:** T34c. **Boy:** S.

## Oku
- Java: `world/entity/ai/attributes/{AttributeMap,AttributeInstance}.java`, `network/protocol/game/ClientboundUpdateAttributesPacket.java`, `world/entity/LivingEntity.java` (`getMaxHealth`, `getScale`).
- Rust: `crates/voxel-world/src/entity.rs` (`ClientAttributeState`, `ClientAttributeModifierState`), `crates/voxel-world/src/lib.rs` (`apply_local_player_attributes`), `crates/voxel-net/src/connection/` (öznitelik paketi), `crates/voxel/src/golems.rs` (`IRON_GOLEM_MAX_HEALTH`).

## Yapılacaklar
1. `UpdateAttributes` paketini mob için de uygula; taban değer + değiştiriciler (`AttributeInstance` hesabı, oyuncu için varolan koddan).
2. `max_health` ve `scale` okuyucuları; demir golem çatlağı ve `placement(.., scale)` bunları kullanır.

## Oracle
`movement/living-entity-*` kalıbı: gerçek varlık, öznitelik paketi, `getMaxHealth`/`getScale` satırları.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`.

## Kapsam dışı
Hız özniteliğinin hareket etkisi (P10), etki (`MobEffect`) kaynaklı değiştiriciler.

## Belgeler
`docs/index.md` satırı; `client-golem-enderman-parity-conformance.md` "can üst sınırı 100" sapması kalkar.

## Devir notu
