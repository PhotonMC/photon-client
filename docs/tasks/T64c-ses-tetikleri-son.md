# T64c — Oyuncunun hasar ve ölüm sesi

**Amaç:** yerel oyuncunun `handleDamageEvent` (hasar türünün `effects` değerine göre `Player.getHurtSound`) ve `handleEntityEvent(3)` (ölüm sesi) seslerini gerçek Java'ya karşı karşılaştırmak. Kalanlar [T64d](T64d-ses-tetikleri-ortam-muzik.md)'de.
**Bağımlılık:** T64b. **Boy:** M.

## Oku
- Java: `world/entity/LivingEntity.java` (`handleDamageEvent`, `handleEntityEvent`), `world/damagesource/{DamageEffects,DamageType}.java`, `network/protocol/game/ClientboundDamageEventPacket.java`.
- Rust: `crates/voxel-world/src/local_player/sounds.rs`, `crates/voxel-net/src/connection/play.rs`.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`.

## Belgeler
`docs/client-block-sounds-conformance.md`.

## Devir notu
- Yapıldı: hasar türü → etki → ses tablosu oracle'dan (`sound/damage-effects`, `tools/generate/damage_effects.py`, `voxel-data::damage_effect`; her koşuda karşılaştırılır), `ClientboundDamageEventPacket` türünün çözümü (`parse_damage_event_packet`; ad, `minecraft:damage_type` kayıt sırasından), `LocalPlayer::{handle_damage_event, handle_death_event}` (yürüme hızı 1,5, `invulnerableTime`, `hurtTime`, ses). Oracle: `movement/local-player-ticks` `hurt-and-die` satırı (gerçek `handleDamageEvent`).
- Sapma: perde varlığın tohumsuz rastgelesidir, karşılaştırmada `0`. Pakette yazılı (kayıtta adı olmayan) hasar türü desteklenmez, varsayılan `hurt` sayılır. Başkalarının hasar sesleri sunucudan `Sound` paketiyle gelir.
