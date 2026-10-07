# T32g — Ravager

**Amaç:** Ravager (`RavagerModel.setupAnim`): saldırı, sersemleme ve kükreme animasyonları.
**Bağımlılık:** T32d. **Boy:** M.

## Oku
- Java: `client/model/monster/ravager/RavagerModel.java`, `client/renderer/entity/RavagerRenderer.java`, `client/renderer/entity/state/RavagerRenderState.java`, `world/entity/monster/Ravager.java` (istemci: `attackTick`, `stunnedTick`, `roarTick`, `aiStep`, `handleEntityEvent` 4/39/...).
- Rust: `crates/voxel/src/hoglins.rs` ve `bears.rs` (kalıp), `voxel-render/src/mob_model.rs` (`setup_hoglin`), `voxel-world/src/animal.rs` (`hoglin_event`, `tick_hoglin`), `docs/records/client-hoglin-parity-conformance.md`.

## Yapılacaklar
1. `setup_ravager` (boyun/ağız: `Mth.triangleWave`, sersemlemede yana salınım, kükremede ağız açılışı; bacaklar `0,4` genlik).
2. `Ravager` istemci sayaçları (olay ve `aiStep` ile ilerler) ve çizim durumu (`getStunnedTick - partial` vb., kükreme kesri `(20 - roarTick + partial)/20`).

## Oracle
`hoglin_model` / `living-entity-animations` kalıbı: gerçek model + `setupAnim`; sayaçlar gerçek `Ravager` ile.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`. Canlı (`shot`, `difficulty easy`): ravager `summon` ile.

## Kapsam dışı
Ravager davranışı (sunucu), sesler, binici illager.

## Belgeler
`docs/index.md` satırı, yeni `docs/records/client-ravager-parity-conformance.md`.

## Devir notu
- Yapıldı: `setup_ravager`, `voxel-world/src/ravager.rs` (saldırı/sersemleme/kükreme sayaçları, olay 4/39/69, çekimler), ortak `entity_random.rs` (panda da kullanır), `crates/voxel/src/ravagers.rs`. Kayıt: `docs/records/client-ravager-parity-conformance.md`.
- Oracle: `ravager-model` 21, `living-entity-animations` 3 satır.
- Sapma: sersemleme/kükreme oracle'da koşulamaz (bulut parçacığı + tekrarlanamayan çekim) — birim test; kükreme itmesi, parçacık, ses yok.
- Açık: T32h strider, T32i titreme.
