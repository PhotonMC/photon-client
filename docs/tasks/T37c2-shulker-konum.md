# T37c2 — Shulker konumu ve vuruş kutusu

**Amaç:** `Shulker.setPos` (blok merkezine yakalama: `floor(x)+0,5`, `floor(y+0,5)`, `floor(z)+0,5`; blok değişince açıklık verisi 0 ve ışınlanma), `getRenderPosition` (6 tiklik kare ağırlıklı kaydırma, `clientOldAttachPosition`), `startRiding`/`stopRiding` ve açıklığa/yüze bağlı `makeBoundingBox` (`getProgressAabb`).
**Bağımlılık:** T37c. **Boy:** M.

## Oku
- Java: `world/entity/monster/Shulker.java` (`setPos`, `getRenderPosition`, `makeBoundingBox`, `getProgressDeltaAabb`), `world/entity/InterpolationHandler.java`, `client/renderer/entity/ShulkerRenderer.java` (`getRenderOffset`, `shouldRender`).
- Rust: `crates/photon-world/src/entity.rs` (`tick_position_interpolation`: konum yazılarından sonra yakalama; `set_position`, `apply_entity`), `crates/photon-world/src/pick.rs` (`dimensions`).

## Kabul
`tools/photon.py check --capture` → `CHECK OK`.

## Devir notu
- Yapıldı: `Shulker.setPos` yakalama (`snapTo`/`set_position`), açıklık sıfırlama, 6 tiklik kayma ve `getRenderPosition` çizim kayması, arayüzsüz (null) interpolasyon, açıklığa/yüze bağlı kutu (çarpışma ve hedefleme), biniş/iniş kancaları. Kayıt: `docs/records/client-shulker-parity-conformance.md`.
- Oracle: `living-entity-animations`'a 9 satır; olay `passengers`, `sync` (yaşayan varlık oracle'ı), shulker ayrıntısına konum/kutu/kayma; Java yolcuyu aracı üzerinden tikler.
- Java'dan sapma ve açık: `onPeekAmountChange` itmesi, yolcu listesi yinelenince gövde sıfırlaması, `shouldRender` görüş genişletmesi (kayıtta).
