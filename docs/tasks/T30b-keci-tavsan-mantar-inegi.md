# T30b — Keçi, tavşan, mantar ineği

**Amaç:** Keçi (boynuzlar, ram/başlık animasyonu, bebek), tavşan (zıplama `jumpCompletion`, çeşitler, bebek, Toast adı) ve mantar ineği (mantar katmanı: blok modelleri, kırmızı/kahverengi çeşit, bebek).
**Bağımlılık:** T30. **Boy:** M–L (mantar katmanı blok çizimi gerektirir; sığmazsa ayır).

## Oku
- Java: `client/model/animal/goat/{GoatModel,BabyGoatModel}.java`, `client/renderer/entity/GoatRenderer.java`, `client/model/animal/rabbit/{RabbitModel,AdultRabbitModel,BabyRabbitModel}.java`, `client/renderer/entity/RabbitRenderer.java`, `client/renderer/entity/MushroomCowRenderer.java`, `client/renderer/entity/layers/MushroomCowMushroomLayer.java`, `world/entity/animal/rabbit/Rabbit.java` (`jumpCompletion`, `setupAnim` girdileri), `world/entity/animal/goat/Goat.java`.
- Rust: `crates/voxel-render/src/mob_model.rs` (`setup_sheep`, `setup_chicken` kalıbı), `crates/voxel/src/{mobs,sheep}.rs`, `crates/voxel-world/src/animal.rs` (tavuk/koyun durumu), `tools/behavior/cases.py` `entity_sheep_cases`.

## Yapılacaklar
1. Keçi ve tavşan model `setupAnim` kolları (zıplama, baş/kulak), durum alanları ve veri okuyucuları (`Goat.DATA_HAS_LEFT_HORN`…, `Rabbit.DATA_TYPE_ID`).
2. Mantar ineği: tür verisi (kırmızı/kahverengi), mantar katmanı: 26.2 `MushroomCowMushroomLayer`'ın blok modelini model ağacıyla çizmesi.

## Oracle
`sheep_model`/`chicken_model` kalıbı (gerçek model sınıfı + gerçek `setupAnim`, dörtgenler); durum için gerçek sınıfın erişilebilir kolları.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`. Canlı: `summon` ile üç tür.

## Kapsam dışı
Keçi çarpması/ram sesleri, tavşan Killer Bunny, binme.

## Belgeler
`docs/index.md` satırı, yeni `docs/client-*-parity-conformance.md`.

## Devir notu
