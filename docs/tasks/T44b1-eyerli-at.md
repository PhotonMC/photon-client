# T44b1 — Eyerli at sürüşü

**Amaç:** yerel oyuncunun oturduğu eyerli at ailesinin istemci tikini (`LivingEntity.travelRidden`, `AbstractHorse.tickRidden`), atlama yükünü (`LocalPlayer.jumpableVehicle`, `START_RIDING_JUMP`) ve binici tikini gerçek Java'ya karşı bit bit eşitlemek.
**Bağımlılık:** T44a, T31. **Boy:** L.

## Oku
- Java: `world/entity/LivingEntity.java` (`aiStep`, `travel*`), `world/entity/animal/equine/AbstractHorse.java`, `client/player/LocalPlayer.java` (`aiStep` atlama dalı), `world/entity/EntityAttachments.java`.
- Rust: `photon-world/src/{mount_ride,vehicle_ride,horse}.rs`, `local_player.rs` (`tick_riding`), `connection/play.rs` (`tick`).

## Oracle
`movement/ride-horse` (`MovementOracle.localPlayerTicks`, `ride.type` at, eyer ve evcil yapılır); `movement/entity-pick-table` koltuk noktaları.

## Kabul
`tools/photon.py check --capture` → `CHECK OK`. Canlı: eyerli at `summon` + `ride`.

## Belgeler
`docs/records/client-mount-ride-parity-conformance.md`, `docs/index.md`.

## Devir notu
- `mount_ride.rs`: mob için `LivingEntity` hareketi (`MountBody`: hava/su/lav, basamak, sekme, blok etkileri; ortak `living_inside_effect`), at için `getRiddenInput`/`tickRidden`/`executeRidersJump`, kalkma sayacı (`horse.rs`); `vehicle_ride.rs`: ortak `VehicleView`, koltuk (`EntityAttachment.PASSENGER` tablosu oracle'dan). 33 satır bit bit eşit.
- Oyuncu: atlama yükü ve `PlayerCommand` yükü, yolcu kapıları. Oturum: araç oyuncudan önce tiklenir, `set_rider_controls`, `on_player_jump`.
- Açık: [T44c](T44c-diger-bineklar.md), varlık itmesi, ölçek/bebek koltuğu, sesler, `LocalPlayer`/`MountBody` ortaklaştırması.
