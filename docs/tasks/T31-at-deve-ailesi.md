# T31 — At, eşek, katır ve ölü atlar

**Amaç:** `AbstractEquineModel` ailesi (at, eşek, katır, iskelet atı, zombi atı): şahlanma, otlama, kuyruk, renk/işaret. Deve ve lama T31b'ye ayrıldı.
**Bağımlılık:** T24. **Boy:** L. **Türler:** horse, donkey, mule, skeleton_horse, zombie_horse.

## Oku
- Java: `client/model/animal/equine/AbstractEquineModel.java`, `client/model/animal/equine/HorseModel.java`, `client/model/animal/llama/LlamaModel.java`, `client/model/animal/camel/CamelModel.java`, `client/renderer/entity/AbstractHorseRenderer.java`, `client/renderer/entity/HorseRenderer.java`, `client/renderer/entity/LlamaRenderer.java`, `client/renderer/entity/CamelRenderer.java`.
- Rust: `crates/voxel-render/src/mob_model.rs` (`MobModel::bake`, `setup_quadruped`; yeni aile için yeni `setup_*` + durum yapısı), `crates/voxel/src/mobs.rs` (`KINDS`, `layer_name`, çeşit çözümü), `crates/voxel-render/src/entity_textures.rs` (`rect`), `crates/voxel-data/src/entity_model.rs` (katman tablosu T23).

## Yapılacaklar
1. `AbstractEquineModel.setupAnim` (baş eğme, kuyruk, yeme/şahlanma durumları `HorseRenderState`).
2. At rengi/işaretleri (`Variant`, `Markings` verisi → katmanlı doku) ve bebek ölçeği.
3. Deve: `KeyframeAnimation` ister (T43b); önce at/lama, deve T43b'den sonra.

## Oracle
`EntityOracle.java` `quadrupedModel` kalıbı: gerçek model sınıfı `setupAnim` ile poz verilir, köşeler `model_layers` tablosuyla birlikte karşılaştırılır; `tools/behavior/cases.py` entity ailesi, `behavior_adapter/main.rs` `entity_result`.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`. Canlı (`shot`): ilgili türler `summon` ile çağrılır, ekran görüntüsü bir kez okunur.

## Kapsam dışı
Eyer, zırh (T41), binme (T44), deve keyframe animasyonları (T43b).

## Belgeler
`docs/remaining-work.md` M4 madde 3; yeni `docs/client-*-parity-conformance.md` kaydı ve `docs/index.md` satırı.

## Not
At dalı açar; eşek/katır/iskelet/zombi atı ölçek+doku satırı; lama ayrı dal. Aile içi türler ayrı dilim değil, tablo satırıdır (AGENTS.md "varyant değil dal"); yeni probe açmadan önce üç soruyu yanıtla.

## Devir notu
(boş)
- Yapıldı (bölündü): `setup_equine` (yedi model, baby donkey iç içe), `voxel-world/src/horse.rs` (otlama/şahlanma/ağız animasyonları, kuyruk, ambiyans yoklamasının rastgele çekimleri), çeşit/işaret/sandık verisi, işaret katmanı; `farm_animals.rs`. Kayıt: `docs/records/client-horse-parity-conformance.md`.
- Oracle: `equine-model` 80, `living-entity-animations` 7 at satırı; `variant-fields` üç satır. Canlı: sekiz kadar at, eşek, katır ve ölü at `shot` ile görüldü.
- Sapma: suda olma izlenmiyor, işaret katmanı varlık hattıyla, eyer/zırh yok.
- Açık: deve ve lama (T31b).

