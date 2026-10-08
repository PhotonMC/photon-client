# T78 — `LevelEvent` ve `BlockEvent` işleyicileri

**Amaç:** `LevelEventHandler.levelEvent` (kapı/trapdoor sesleri, fırın, tuğla kırılması, dispenser, portal, havai fişek…: ses ve parçacık), `BlockEventPacket` (nota bloğu, sandık kapağı, piston, çan), `ClientLevel.blockEvent`; 2001 dışında `LevelEvent` şimdi yok sayılıyor.
**Bağımlılık:** T71, T64b. **Boy:** L.

## Oku
Java: `client/renderer/LevelEventHandler.java`, `network/protocol/game/{ClientboundLevelEventPacket,ClientboundBlockEventPacket}.java`, `world/level/block/NoteBlock.java`, `world/level/block/entity/ChestBlockEntity.java` (`triggerEvent`). Rust: `crates/photon-world/src/lib.rs` (`apply_level_event`), `crates/photon-net/src/connection/play.rs`.

## Oracle
`sound` ailesinin tetik oracle'ı (gerçek `LevelEventHandler` sahte `SoundManager`/`ParticleEngine` ile): olay numarası → ses/parçacık dizisi. Önce Java kaynağı satır satır okunur, oracle senaryosu yazılır, sonra Rust ([davranış testi belgesi](../behavior-testing.md)).

## Kabul
`tools/photon.py check --capture` → `CHECK OK`. Canlı gözlem yalnız görsel bir sonuç varsa, `tools/photon.py shot` ile bir kez.

## Kapsam dışı
Başka aileler (kendi görevlerinde); performans optimizasyonu.

## Belgeler
İlgili contract/parity kaydı (`docs/` altında yeni dosya), `docs/index.md`, `docs/remaining-work.md` §8 işareti.

## Devir notu
Yapılan: olaylar ağdan kuyruğa girer, motor seviye rastgelesiyle işler (`particles/level_events.rs`):
ses olayları 1000–1052, 1009, 1501–1503, 2000/2008/2009/2010, 2001 (ses ve kırıntı). Her sesten sonra
tohum için `nextLong` çekilir. `LocalSoundSource`'a düşman ve tarafsız eklendi.
Oracle: `particle-engine`'de dört satır; sesler eşit, atış özeti Q04 Gauss farkıyla tutmuyor.
Bölündü: kalan olaylar → T78b, `BlockEvent` → T78c.
