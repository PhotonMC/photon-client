# T29 — İskelet ve piglin aileleri (yay/arbalet/kılıç pozları)

**Amaç:** İskelet, stray, bogged, parched, wither iskeleti ve piglin, piglin brute, zombifiye piglin: `HumanoidModel` üzerine silah pozları (`ArmPose`: yay çekme, arbalet, kılıç sallama), piglin kulakları ve `admiring`/`dancing` pozları.
**Bağımlılık:** T28. **Boy:** M. **Türler:** skeleton, stray, bogged, parched, wither_skeleton, piglin, piglin_brute, zombified_piglin.

## Oku
- Java: `client/model/monster/skeleton/SkeletonModel.java`, `client/model/monster/piglin/PiglinModel.java`, `client/renderer/entity/AbstractSkeletonRenderer.java`, `client/renderer/entity/SkeletonRenderer.java`, `client/renderer/entity/BoggedRenderer.java`, `client/renderer/entity/PiglinRenderer.java`, `client/renderer/entity/ZombifiedPiglinRenderer.java`.
- Rust: `crates/voxel-render/src/entity/mob_model.rs` (`MobModel::bake`, `setup_quadruped`; yeni aile için yeni `setup_*` + durum yapısı), `crates/voxel/src/mobs.rs` (`KINDS`, `layer_name`, çeşit çözümü), `crates/voxel-render/src/entity/textures.rs` (`rect`), `crates/voxel-data/src/entity_model.rs` (katman tablosu T23).

## Yapılacaklar
1. `ArmPose` seçimi (elde tutulan eşya `SetEquipment`'e bağlı: eşya yokken varsayılan poz; T41 sonrası tam).
2. Piglin kulak ve poz dalları (`PiglinModel.setupAnim`).
3. Dış katmanlar (stray giysisi, bogged mantarları): yalnız katman+doku satırı.

## Oracle
`EntityOracle.java` `quadrupedModel` kalıbı: gerçek model sınıfı `setupAnim` ile poz verilir, köşeler `model_layers` tablosuyla birlikte karşılaştırılır; `tools/behavior/cases.py` entity ailesi, `behavior_adapter/main.rs` `entity_result`.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`. Canlı (`shot`): ilgili türler `summon` ile çağrılır, ekran görüntüsü bir kez okunur.

## Kapsam dışı
Eşya tutma ve zırh (T41), silah kullanma yapay zekâsı (sunucu).

## Belgeler
`docs/remaining-work.md` M4 madde 3; yeni `docs/client-*-parity-conformance.md` kaydı ve `docs/index.md` satırı.

## Not
Skeleton dalı açar; stray/bogged/parched/wither iskeleti tablo satırı. Piglin ikinci dal. Aile içi türler ayrı dilim değil, tablo satırıdır (AGENTS.md "varyant değil dal"); yeni probe açmadan önce üç soruyu yanıtla.

## Devir notu
- Yapıldı: `MobModel::setup_skeleton` (saldırgan yaysız kol pozu), `setup_piglin` (kulaklar, dans, zombifiye kollar), `mobs.rs` humanoid tablosu 12 tür (stray/bogged dış katmanları), `Piglin.DATA_BABY_ID`/`DATA_IS_DANCING` (17/19, oracle'la doğrulandı) dünyaya. Kayıt: `docs/records/client-skeleton-piglin-parity-conformance.md`.
- Oracle: `entity/skeleton-model` 63 + `entity/piglin-model` 47 satır (gerçek model sınıfları). Canlı: altı tür yan yana.
- Sapma: elde eşya yok (iskelet yay tutuyor sayılır, T41), `attackTime` sıfır (T43), mantar/titreme bayrakları yok, mob yüzme tiklenmiyor.
- Açık: yok.
