# T32h — Strider

**Amaç:** Strider (yetişkin ve bebek, `StriderModel.setupAnim`): yürüme, saç salınımı, eyer katmanı, soğuk doku (`isSuffocating`).
**Bağımlılık:** T32d. **Boy:** M.

## Oku
- Java: `client/model/monster/strider/{StriderModel,AdultStriderModel,BabyStriderModel}.java`, `client/renderer/entity/StriderRenderer.java`, `client/renderer/entity/state/StriderRenderState.java`, `world/entity/monster/Strider.java` (istemci: `DATA_SUFFOCATING`, `DATA_SADDLE_ID`/eyer ekipmanı, `isRidden`).
- Rust: `crates/photon/src/mobs/monsters/hoglins.rs` ve `bears.rs` (kalıp), `photon-render/src/mob_model.rs` (`setup_hoglin`), `docs/records/client-hoglin-parity-conformance.md`; eyer için `crates/photon/src/mobs/animals/farm_animals.rs` (domuz/at eyeri).

## Yapılacaklar
1. `setup_strider` (bacak/saç salınımı `ageInTicks` ile, binici varken hız sönümü).
2. Soğuk (boğulan) doku ve eyer katmanı (`minecraft:strider#saddle`).

## Oracle
`hoglin_model` / `living-entity-animations` kalıbı: gerçek model + `setupAnim`.

## Kabul
`tools/photon.py check --capture` → `CHECK OK`. Canlı (`shot`): strider (soğuk, eyerli) `summon` ile.

## Kapsam dışı
Strider davranışı ve binme kontrolü (T44), sesler.

## Belgeler
`docs/index.md` satırı, yeni `docs/records/client-strider-parity-conformance.md`.

## Devir notu
- Yapıldı: `setup_strider` (yetişkin, bebek, eyer katmanı modeli), `Armor::simple_layer` (genel basit ekipman katmanı: strider eyeri), `photon-world` boğulma verisi ve `is_vehicle`, `crates/photon/src/mobs/monsters/striders.rs`. Kayıt: `docs/records/client-strider-parity-conformance.md`.
- Oracle: `strider-model` 18, `strider-textures` 4, `living-entity-animations` 2 (yapay zekâsız), `variant-fields` +1; harness'a `mob_flags`, `strider_data` olayları.
- Sapma: yapay zekâlı strider boğulmayı kendi hesaplamıyor (sunucu verisi); titreme T32i; eyer sırası ve ses yok.
- Açık: T32i titreme (boğulan strider de eklendi).
