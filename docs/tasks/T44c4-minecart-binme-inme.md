# T44c4 — Minecart yolcusu ve binme/inme

**Amaç:** minecart yolcusu (`AbstractMinecart`, koltuk ve sunucunun yürüttüğü hareket) ve binme/inme akışı (`startRiding`, `getDismountLocationForPassenger`, `mount.onboard`, `Shift` ile inme).
**Bağımlılık:** T44c3. **Boy:** M.

## Oku
- Java: `world/entity/vehicle/minecart/AbstractMinecart.java`, `Entity.java` (`startRiding`, `removeVehicle`, `getDismountLocationForPassenger`).
- Rust: `voxel-world/src/{vehicle_ride,mount_ride,local_player}.rs`.

## Oracle
`movement/ride-camel` örneğiyle `ride` satırları; `kind` olarak minecart.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`.

## Belgeler
`docs/client-mount-ride-parity-conformance.md`.

## Devir notu
(boş)
