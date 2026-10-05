# T37h — Ender ejderhası

**Amaç:** `EnderDragonModel` (parçalı gövde, kanat ve boyun dizileri), `EnderDragonRenderer` (uçuş geçmişi `DragonFlightHistory`, ölüm ışını, kristal ışını), `EnderDragon` istemci durumu ve `EnderDragonPart` seçilebilirliği (T26 notu).
**Bağımlılık:** T37. **Boy:** L.

## Oku
- Java: `world/entity/boss/enderdragon/{EnderDragon,EnderDragonPart}.java`, `client/model/monster/dragon/EnderDragonModel.java`, `client/renderer/entity/{EnderDragonRenderer,state/EnderDragonRenderState}.java`.
- Rust: `crates/voxel-world/src/pick.rs` (seçilebilirlik), `crates/voxel-render/src/mob_model.rs`.

## Kapsam dışı
Savaş yapıları, ejderha yumurtası, ses.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`.

## Devir notu
- Yapıldı: canlı ejderha. `setup_ender_dragon` (geçmişe bağlı boyun/kuyruk zinciri), `voxel-world/ender_dragon.rs` (`FlightHistory`, çırpış, evre veri 16, `dragonDeathTime`), `EnderDragon.aiStep`'in istemci hâli (hareket sönmez, yürüyüş yok), ölüm olayında `handleKillingBlow`, `ender_dragons.rs` çizici (geçmişe bağlı dönüş ve eğim, gözler, yumurtaya uzaklık için `MOTION_BLOCKING_NO_LEAVES`). Kayıt: `docs/client-ender-dragon-parity-conformance.md`.
- Kalan: ölüm çizimi [T37h2](T37h2-ejderha-olumu.md), kristal ışını [T37h3](T37h3-ejderha-kristal-isini.md), parçalar [T37h4](T37h4-ejderha-parcalari.md); bu görev `bölündü`.
- Sapma: ölürken normal çizim; ses/parçacık yok; canlı gözlem yapılmadı.
