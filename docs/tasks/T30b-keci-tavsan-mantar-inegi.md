# T30b — Keçi ve mantar ineği

**Amaç:** Keçi (boynuzlar, baş indirme, bebek) ve mantar ineği (kırmızı/kahverengi, mantar katmanı: `entity_block` yolu, bebek). Tavşan T30c'ye ayrıldı (`KeyframeAnimation` ister).
**Bağımlılık:** T30, T34d. **Boy:** M.

## Oku
- Java: `client/model/animal/goat/{GoatModel,BabyGoatModel}.java`, `client/renderer/entity/GoatRenderer.java`, `client/model/animal/rabbit/{RabbitModel,AdultRabbitModel,BabyRabbitModel}.java`, `client/renderer/entity/RabbitRenderer.java`, `client/renderer/entity/MushroomCowRenderer.java`, `client/renderer/entity/layers/MushroomCowMushroomLayer.java`, `world/entity/animal/rabbit/Rabbit.java` (`jumpCompletion`, `setupAnim` girdileri), `world/entity/animal/goat/Goat.java`.
- Rust: `crates/voxel-render/src/entity/mob_model.rs` (`setup_sheep`, `setup_chicken` kalıbı), `crates/voxel/src/{mobs,sheep}.rs`, `crates/voxel-world/src/mobs/animal.rs` (tavuk/koyun durumu), `tools/behavior/cases.py` `entity_sheep_cases`.

## Yapılacaklar
1. Keçi modeli `setupAnim` kolları, baş indirme zamanlayıcısı (olay 58/59) ve boynuz verisi.
2. Mantar ineği: tür verisi (`DATA_TYPE`), mantar katmanı dönüşümleri.

## Oracle
`sheep_model`/`chicken_model` kalıbı (gerçek model sınıfı + gerçek `setupAnim`, dörtgenler); durum için gerçek sınıfın erişilebilir kolları.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`. Canlı: `summon` ile üç tür.

## Kapsam dışı
Keçi çarpması/ram sesleri, tavşan Killer Bunny, binme.

## Belgeler
`docs/index.md` satırı, yeni `docs/client-*-parity-conformance.md`.

## Devir notu
- Yapıldı: `setup_goat`, keçi zamanlayıcısı (`animal.rs`) ve boynuz/mooshroom verisi, `entity_block::mooshroom_mushroom_poses`, `voxel/src/farm_animals.rs`. Kayıt: `docs/records/client-goat-mooshroom-parity-conformance.md`.
- Oracle: `goat-model` 14, mantar ineği `quadruped-model` 14, `block-layers` 2, `living-entity-animations` 3 keçi satırı; `variant-fields` üç satır. Canlı: keçi (yetişkin, bebek) ve iki mooshroom `shot` ile görüldü.
- Sapma: tavşan yok (T30c); parlayan görünmezde dış hat yok.

