# T36f — Mutlu ghast

**Amaç:** `HappyGhastModel` (+ `HappyGhastHarnessModel` koşum katmanı), `HappyGhastRenderer` ve ilgili istemci durumları (bebek ve yetişkin, koşum eşyası, `isRidden`, dokunaç salınımı). Bu oturumda nautilusun eyer ve zırh katmanları da (T35e açığı) aynı altyapıyla kapandı. İp katmanı (bağlı mob durumu yok) açık.
**Bağımlılık:** T36, T41. **Boy:** M.

## Oku
- Java: `client/model/animal/ghast/HappyGhast*.java`, `client/renderer/entity/HappyGhastRenderer.java`, `client/renderer/entity/state/HappyGhastRenderState.java`, `world/entity/animal/happyghast/HappyGhast.java`.
- Rust: `crates/voxel-render/src/mob_model.rs` (`setup_ghast`), `crates/voxel/src/monsters.rs`.

## Oracle
`airborne_model` kalıbı; gerçek varlık tiki için `living-entity-animations`.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`.

## Kapsam dışı
Sesler, parçacıklar, davranışlar, binme denetimi.

## Devir notu
- Yapıldı: `setup_happy_ghast` ve `setup_happy_ghast_harness`, `crates/voxel/src/happy_ghasts.rs` (koşum `simple_layer` ile), nautilusun eyer ve zırh katmanları (`armor` `remote_water_animals` ve `remote_nautiluses`'a taşındı; `SIMPLE` katman türleri `happy_ghast_body`, `nautilus_body`, `nautilus_saddle`). Kayıt: `docs/records/client-happy-ghast-parity-conformance.md`.
- Sapma: ip katmanı yok (bağlı mob bilgisi `SetEntityLink` istemcide tutulmuyor); koşum ve eyer dokuları oracle'sız. Canlı gözlem yapılmadı.
