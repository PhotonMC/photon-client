# T44c2 — Strider (deve ve minecart T44c3'e ayrıldı)

**Amaç:** strider (`Strider.getRiddenInput/getRiddenSpeed`, lavın üstünde yürümek: `canStandOnFluid`, `getLiquidCollisionShape`, `floatStrider`, sıcak blok boğulması, yürüme animasyonlu koltuk), deve (`Camel`: koşu, oturma, `canSprint`), minecart sürüşü; binme/inme (`startRiding`, `getDismountLocationForPassenger`, `mount.onboard`).
**Bağımlılık:** T44c1. **Boy:** L (türe göre böl).

## Oku
- Java: `world/entity/monster/Strider.java`, `world/entity/animal/camel/Camel.java`, `world/entity/vehicle/minecart/AbstractMinecart.java`.
- Rust: `photon-world/src/mount_ride.rs`, `vehicle_ride.rs`.

## Oracle
`movement/ride-pig` örneğiyle türe göre `ride` satırları.

## Kabul
`tools/photon.py check --capture` → `CHECK OK`.

## Belgeler
`docs/records/client-mount-ride-parity-conformance.md`.

## Devir notu
- `MountKind::Strider`: lav yüzeyi çarpışması (`Level::standing_on_lava_at`), `floatStrider`, soğukluk bayrağı ve hız özelliği çarpanı, yürüyüşle oynayan koltuk; 17 satır gerçek Java'ya karşı bit bit eşit; canlıda lav üstünde sürüldü.
- Açık: [T44c3](T44c3-deve-minecart.md), çarpık mantarlı sopanın `boost`'u.
