# T32h — Strider

**Amaç:** Strider (yetişkin ve bebek, `StriderModel.setupAnim`): yürüme, saç salınımı, eyer katmanı, soğuk doku (`isSuffocating`).
**Bağımlılık:** T32d. **Boy:** M.

## Oku
- Java: `client/model/monster/strider/{StriderModel,AdultStriderModel,BabyStriderModel}.java`, `client/renderer/entity/StriderRenderer.java`, `client/renderer/entity/state/StriderRenderState.java`, `world/entity/monster/Strider.java` (istemci: `DATA_SUFFOCATING`, `DATA_SADDLE_ID`/eyer ekipmanı, `isRidden`).
- Rust: `crates/voxel/src/hoglins.rs` ve `bears.rs` (kalıp), `voxel-render/src/mob_model.rs` (`setup_hoglin`), `docs/client-hoglin-parity-conformance.md`; eyer için `crates/voxel/src/farm_animals.rs` (domuz/at eyeri).

## Yapılacaklar
1. `setup_strider` (bacak/saç salınımı `ageInTicks` ile, binici varken hız sönümü).
2. Soğuk (boğulan) doku ve eyer katmanı (`minecraft:strider#saddle`).

## Oracle
`hoglin_model` / `living-entity-animations` kalıbı: gerçek model + `setupAnim`.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`. Canlı (`shot`): strider (soğuk, eyerli) `summon` ile.

## Kapsam dışı
Strider davranışı ve binme kontrolü (T44), sesler.

## Belgeler
`docs/index.md` satırı, yeni `docs/client-strider-parity-conformance.md`.

## Devir notu
(boş)
