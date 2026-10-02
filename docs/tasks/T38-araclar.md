# T38 — Araçlar: tekneler ve minecart ailesi

**Amaç:** Tekne ve sandıklı tekne (ahşap türleri, bambu salı), minecart ve türevleri (sandık, ocak, huni, TNT, spawner, komut bloğu): model, yönelme, su alma/sallanma, ray eğimi (`MinecartBehavior` istemci tiki).
**Bağımlılık:** T24. **Boy:** M. **Türler:** *_boat, *_chest_boat, bamboo_raft, bamboo_chest_raft, minecart, chest_minecart, furnace_minecart, hopper_minecart, tnt_minecart, spawner_minecart, command_block_minecart.

## Oku
- Java: `client/model/object/boat/BoatModel.java`, `client/renderer/entity/BoatRenderer.java`, `client/model/object/cart/MinecartModel.java`, `client/renderer/entity/AbstractMinecartRenderer.java`, `client/renderer/entity/MinecartRenderer.java`.
- Rust: `crates/voxel-render/src/mob_model.rs` (`MobModel::bake`, `setup_quadruped`; yeni aile için yeni `setup_*` + durum yapısı), `crates/voxel/src/mobs.rs` (`KINDS`, `layer_name`, çeşit çözümü), `crates/voxel-render/src/entity_textures.rs` (`rect`), `crates/voxel-data/src/entity_model.rs` (katman tablosu T23).

## Yapılacaklar
1. Tekne çizimi (`BoatRenderState`, kürek açısı `rowingTime`, balçık/su yarığı `WaterPatch`); ahşap türü doku satırı.
2. Minecart: yönelme ve ray eğimi (`NewMinecartBehavior` istemci iç-konumlaması); içerik bloğu (`getDisplayBlockState`).
3. Oyuncunun bindiği araç kontrolü T44'te.

## Oracle
`EntityOracle.java` `quadrupedModel` kalıbı: gerçek model sınıfı `setupAnim` ile poz verilir, köşeler `model_layers` tablosuyla birlikte karşılaştırılır; `tools/behavior/cases.py` entity ailesi, `behavior_adapter/main.rs` `entity_result`.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`. Canlı (`shot`): ilgili türler `summon` ile çağrılır, ekran görüntüsü bir kez okunur.

## Kapsam dışı
Sürüş (T44), ray fiziği sunucu tarafında, sesler.

## Belgeler
`docs/remaining-work.md` M4 madde 3; yeni `docs/client-*-parity-conformance.md` kaydı ve `docs/index.md` satırı.

## Not
Tekne dalı + minecart dalı; türevler tablo satırı. Aile içi türler ayrı dilim değil, tablo satırıdır (AGENTS.md "varyant değil dal"); yeni probe açmadan önce üç soruyu yanıtla.

## Devir notu
(boş)
