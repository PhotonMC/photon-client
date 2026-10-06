# T44a — Oyuncunun tekneyi sürmesi

**Amaç:** yerel oyuncunun oturduğu teknenin istemci tikini (`AbstractBoat.tick`: durum, yüzdürme, kontrol, hareket) ve oyuncunun yolcu tikini (`LocalPlayer.rideTick`), `PlayerInput`/`MovePlayerRot`/`MoveVehicle`/`PaddleBoat` paketlerini gerçek Java'ya karşı bit bit eşitlemek.
**Bağımlılık:** T38. **Boy:** L.

## Oku
- Java: `world/entity/vehicle/boat/AbstractBoat.java`, `client/player/LocalPlayer.java` (`tick`, `rideTick`), `world/entity/Entity.java` (`move`, `baseTick`), `ClientLevel.tickNonPassenger`.
- Rust: `voxel-world/src/boat_ride.rs`, `voxel-world/src/local_player.rs` (`tick_riding`), `crates/voxel-net/src/connection/play.rs` (`tick`).

## Oracle
`movement/ride-boat` (`MovementOracle.localPlayerTicks` gerçek bir tekneyi `ride` ile bindirir).

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`. Canlı: tekne `summon` + `ride` ile, ileri ve dönüş.

## Belgeler
`docs/client-boat-ride-parity-conformance.md`, `docs/index.md`.

## Devir notu
- `boat_ride.rs`: `getStatus`/`floatBoat`/`controlBoat`/`move`/blok etkileri (bal, ağ, toz kar, kabarcık sütunu) gerçek tekneye karşı 40 satır bit bit eşit; oyuncu tarafı `LocalPlayer::tick_riding` (yolcu kapıları: yerde değil, çömelme/yüzme/sprint yok, akışkan kutusu, yürüme durur).
- Oturum: araç oyuncudan önce tiklenir (`PaddleBoat` önce), `turn_player` dönüşü teknenin dörtte biriyle sınırlar, binişte oyuncu teknenin yönüne bakar. `ClientLevel.hasChunk` her zaman doğru: oyuncu yolundaki yüklü yığın sorgusu düzeltildi.
- Açık: [T44b](T44b-binek-hayvanlar-minecart.md) (diğer araçlar), varlık itme/çarpışması, `yRotO`, binme/inme.
