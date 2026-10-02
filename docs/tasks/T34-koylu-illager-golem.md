# T34 — Köylü, tüccar, illager, cadı, golemler ve enderman

**Amaç:** Uzun burunlu insansılar: köylü ve gezgin tüccar (`VillagerModel`, meslek/seviye katmanı, başı sallama), illager ailesi (`IllagerModel`: kol çaprazı/saldırı pozu), cadı (burun, içme), demir/kar/bakır golem, enderman (taşıdığı blok, çığlık).
**Bağımlılık:** T24. **Boy:** L. **Türler:** villager, wandering_trader, evoker, illusioner, pillager, vindicator, witch, iron_golem, snow_golem, copper_golem, enderman.

## Oku
- Java: `client/model/npc/VillagerModel.java`, `client/model/monster/illager/IllagerModel.java`, `client/model/monster/witch/WitchModel.java`, `client/model/animal/golem/IronGolemModel.java`, `client/model/animal/golem/SnowGolemModel.java`, `client/model/animal/golem/CopperGolemModel.java`, `client/model/monster/enderman/EndermanModel.java`, `client/renderer/entity/VillagerRenderer.java`, `client/renderer/entity/IllagerRenderer.java`, `client/renderer/entity/EndermanRenderer.java`.
- Rust: `crates/voxel-render/src/mob_model.rs` (`MobModel::bake`, `setup_quadruped`; yeni aile için yeni `setup_*` + durum yapısı), `crates/voxel/src/mobs.rs` (`KINDS`, `layer_name`, çeşit çözümü), `crates/voxel-render/src/entity_textures.rs` (`rect`), `crates/voxel-data/src/entity_model.rs` (katman tablosu T23).

## Yapılacaklar
1. Villager meslek/seviye katmanı (`VillagerData` verisi; T24b yürüyüşü bunu atlıyor, okuyucu eklenir).
2. Illager kol pozları (`IllagerModel.ArmPose`).
3. Enderman: taşınan blok ve `creepy`; demir golemin çatlak katmanı (`Crackiness`).

## Oracle
`EntityOracle.java` `quadrupedModel` kalıbı: gerçek model sınıfı `setupAnim` ile poz verilir, köşeler `model_layers` tablosuyla birlikte karşılaştırılır; `tools/behavior/cases.py` entity ailesi, `behavior_adapter/main.rs` `entity_result`.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`. Canlı (`shot`): ilgili türler `summon` ile çağrılır, ekran görüntüsü bir kez okunur.

## Kapsam dışı
Ticaret ekranı (`MerchantScreen`, ayrı ekran görevi), köylü sesleri.

## Belgeler
`docs/remaining-work.md` M4 madde 3; yeni `docs/client-*-parity-conformance.md` kaydı ve `docs/index.md` satırı.

## Not
Villager dalı açar; illagerlar tablo satırı (aynı model, farklı doku/poz). Aile içi türler ayrı dilim değil, tablo satırıdır (AGENTS.md "varyant değil dal"); yeni probe açmadan önce üç soruyu yanıtla.

## Devir notu
(boş)
