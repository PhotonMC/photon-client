# T44c3 — Deve ve minecart

**Amaç:** eyerli deve (`Camel`: koşu `dash`, oturma, `getRiddenInput`, `canSprint`), minecart yolcusu ve binme/inme (`startRiding`, `getDismountLocationForPassenger`, `mount.onboard`).
**Bağımlılık:** T44c2. **Boy:** M.

## Oku
- Java: `world/entity/animal/camel/Camel.java`, `world/entity/vehicle/minecart/AbstractMinecart.java`.
- Rust: `voxel-world/src/{mount_ride,vehicle_ride,horse}.rs`.

## Oracle
`movement/ride-horse` örneğiyle `ride` satırları.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`.

## Belgeler
`docs/client-mount-ride-parity-conformance.md`.

## Devir notu
(boş)
