# T44c — Öteki binekler: eyerli domuz, strider, deve, minecart

**Amaç:** eyerli domuz (`Pig.getRiddenInput`, havuç sopası), strider (`Strider.getRiddenInput`, lavda), deve (`Camel`: koşu, oturma), minecart sürüşü ve binme/inme (`startRiding`, `getDismountLocationForPassenger`, `mount.onboard`).
**Bağımlılık:** T44b1. **Boy:** L (türe göre böl).

## Oku
- Java: `world/entity/animal/pig/Pig.java`, `world/entity/monster/Strider.java`, `world/entity/animal/camel/Camel.java`, `world/entity/vehicle/minecart/AbstractMinecart.java`.
- Rust: `photon-world/src/mount_ride.rs` (örnek), `vehicle_ride.rs`.

## Oracle
`movement/ride-horse` örneğiyle türe göre `ride` satırları.

## Kabul
`tools/photon.py check --capture` → `CHECK OK`.

## Belgeler
Yeni `docs/client-*-parity-conformance.md` kaydı ve `docs/index.md`.

## Devir notu
(boş)
