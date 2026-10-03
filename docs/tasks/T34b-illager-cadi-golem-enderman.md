# T34b — Illager ve cadı

**Amaç:** `IllagerModel` (kol çaprazı/saldırı pozu: `ArmPose`), cadı (`WitchModel`: burun, içme), demir golem (`Crackiness` çatlak katmanı), kar golemi, bakır golem, enderman (taşınan blok, `creepy`) ve zombi köylünün çizimi (`ZombieVillagerModel` + T34'te hazır `profession_passes("zombie_villager", …)`).
**Bağımlılık:** T34. **Boy:** L (sığmazsa böl: illager+cadı / golemler / enderman).

## Oku
- Java: `client/model/monster/illager/IllagerModel.java`, `client/model/monster/witch/WitchModel.java`, `client/model/animal/golem/{IronGolemModel,SnowGolemModel,CopperGolemModel}.java`, `client/model/monster/enderman/EndermanModel.java`, `client/model/monster/zombie/ZombieVillagerModel.java`, `client/renderer/entity/{Illager,Witch,IronGolem,SnowGolem,Enderman,ZombieVillager}Renderer.java`.
- Rust: `crates/voxel/src/villagers.rs` (köylü kalıbı), `voxel-render/src/{mob_model,villager}.rs`, `voxel/src/mobs.rs` (`HUMANOIDS`, zombi kolları), `voxel-world/src/metadata.rs`.

## Yapılacaklar
1. Illager kol pozları (`IllagerModel.ArmPose`) ve cadı; illagerlar tablo satırı.
2. Golemler (demir golemin çatlak katmanı), enderman (taşınan blok, `creepy`).
3. Zombi köylü: `ZombieVillagerModel` + `profession_passes`.

## Oracle
`EntityOracle.java` `villagerModel` kalıbı: gerçek model sınıfı + `setupAnim`, köşeler `posedQuads` ile.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`. Canlı: `summon`.

## Kapsam dışı
Ticaret ekranı, sesler, köylü kafa eşyası/çapraz kol eşyası (T41d ile).

## Belgeler
`docs/index.md` satırı, yeni kayıt.

## Devir notu
- Yapıldı (bölündü): dört illager (`setup_illager`, her kol pozu, `voxel/src/illagers.rs`: poz seçimi, eldeki eşya ve kafa katmanı) ve cadı (`setup_witch`, `villagers.rs`); `Raider.IS_CELEBRATING` 16, büyü baytı 17. Kayıt: `docs/client-illager-witch-parity-conformance.md`.
- Oracle: `entity/illager-model` 21, `witch-model` 5 satır; `variant-fields` iki yeni satır.
- Sapma: arbalet doldurma pozu (kullanım tiki yok), illusioner yansımaları, `WitchItemLayer` yok.
- Açık: golemler, enderman, zombi köylü (T34c).
