# T37h4 — Ender ejderhasının parçaları

**Amaç:** `EnderDragonPart` alt varlıkları (baş, boyun, gövde, kanatlar, üç kuyruk): `tickPart` konumları, vuruş kutuları ve seçilebilirlik (T26 notu), ejderhanın kendisine yönlendirilen saldırı ve etkileşim.
**Bağımlılık:** T37h. **Boy:** M.

## Oku
- Java: `world/entity/boss/enderdragon/{EnderDragon,EnderDragonPart}.java`, `world/entity/PartEntity.java`.
- Rust: `crates/photon-world/src/pick.rs` (seçilebilirlik), `crates/photon-world/src/mobs/ender_dragon.rs`.

## Kabul
`tools/photon.py check --capture` → `CHECK OK`.

## Devir notu
- Yapıldı: `DragonPart` (sekiz parça, boyut, kimlik kuralı), `tick_parts` (`tickPart`), ölürken 0.1 yükseliş (ejderha ve parçalar), `entity_hit_result`'ta parçalar (`EntityHit` artık konum ve kutu taşır, `photon-game/pick.rs` `entity_at`'a bağlı değil). Kayıt: `docs/records/client-ender-dragon-parts-parity-conformance.md`.
- Sapma: nişan oracle'sız (birim test); parça kimlik kuralı kaynaktan; aynı mesafede eşitlik sırası.
- Açık: ejderhaya `Attack`/`Interact` paketlerinin sunucuyla canlı denemesi.
