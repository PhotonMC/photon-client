# T35b — Kalamar ve ışıklı kalamar

**Amaç:** `SquidModel`/`BabySquidModel`, `SquidRenderer.setupRotations`, `Squid`/`GlowSquid` istemci tiki (dokunaç vuruşu, gövde dönüşleri), mob'ların suda olma durumu ve hareket azalması.
**Bağımlılık:** T35. **Boy:** M–L.

## Oku
- Java: `world/entity/animal/squid/{Squid,GlowSquid}.java`, `client/model/animal/squid/*.java`, `client/renderer/entity/{Squid,GlowSquid}Renderer.java`, `LivingEntity.aiStep`, `Entity.updateFluidInteraction`.
- Rust: `crates/voxel-world/src/{squid,fluid,entity}.rs`, `crates/voxel/src/mobs/aquatic/squids.rs`.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`.

## Devir notu
- Yapıldı: `SquidState` (tik), mob suda olma durumu (`mob_in_water`, `tracks_water`: yalnız kalamar), `LivingEntity.aiStep`'in hareket azalması ve eşiği (bütün mob'lar), `GlowSquid` karanlık tikleri, `setup_squid`, `SquidTurns` (çizim dönüşü), `squids.rs`. Kayıt: `docs/records/client-squid-parity-conformance.md`.
- Oracle: `entity/squid-model` 16, `squid-rotations` 8, `glow-squid-light` 24, `living-entity-animations`'a 10 kalamar satırı. Oracle'ın varlık başlangıç yönü artık sabit (paketteki gibi).
- Sapma: akıntıyla itilme (öteki su canlıları için) yok; mürekkep, ses, gölge yok; canlı gözlem yapılmadı.
