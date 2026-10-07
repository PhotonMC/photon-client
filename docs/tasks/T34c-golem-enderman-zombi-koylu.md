# T34c — Demir ve kar golemi, enderman, zombi köylü

**Amaç:** demir golem (`Crackiness` çatlak katmanı), kar golemi (balkabağı başlık), bakır golem, enderman (taşınan blok, `creepy`) ve zombi köylünün çizimi (`ZombieVillagerModel` + T34'te hazır `profession_passes("zombie_villager", …)`).
**Bağımlılık:** T34b. **Boy:** L (sığmazsa böl: golemler / enderman+zombi köylü).

## Oku
- Java: `client/model/animal/golem/{IronGolemModel,SnowGolemModel,CopperGolemModel}.java`, `client/model/monster/enderman/EndermanModel.java`, `client/model/monster/zombie/ZombieVillagerModel.java`, `client/renderer/entity/{IronGolem,SnowGolem,CopperGolem,Enderman,ZombieVillager}Renderer.java`, `client/renderer/entity/layers/{IronGolemCrackinessLayer,CarriedBlockLayer}.java`.
- Rust: `crates/voxel/src/{villagers,illagers}.rs` (kalıp), `voxel-render/src/{mob_model,villager}.rs`, `voxel/src/mobs.rs` (`HUMANOIDS`, zombi kolları), `voxel-world/src/metadata.rs`.

## Yapılacaklar
1. Golemler (demir golemin çatlak katmanı, kar golemi, bakır golem).
2. Enderman (taşınan blok, `creepy`), `ZombieVillagerModel` + `profession_passes`.

## Oracle
`EntityOracle.java` `illagerModel`/`villagerModel` kalıbı: gerçek model sınıfı + `setupAnim`, köşeler `posedQuads` ile.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`. Canlı: `summon`.

## Kapsam dışı
Sesler, patlama/teleport parçacıkları (PLAN-M8).

## Belgeler
`docs/index.md` satırı, yeni kayıt.

## Devir notu
- Yapıldı (bölündü): demir golem (model, yürürken sallanma `MobPlacement.roll`, çatlak katmanı, saldırı/çiçek zamanlayıcıları olay 4/11/34), kar golemi, enderman (taşıma ve ürkütme verisi dahil model), zombi köylü (insansı tablo satırı + meslek katmanı); `voxel/src/golems.rs`. Kayıt: `docs/records/client-golem-enderman-parity-conformance.md`.
- Oracle: `iron-golem-model` 9, `snow-golem-model` 3, `enderman-model` 6, `iron-golem-render` 16, zombi köylü 6 satır, `living-entity-animations` 5 demir golem satırı.
- Sapma: balkabağı, çiçek, taşınan blok, enderman gözleri ve titremesi yok; can üst sınırı 100 sayılır.
- Açık: blok çizimi (T34d), bakır golem (T34e).

