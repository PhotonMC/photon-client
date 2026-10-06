# T44c2 — Strider, deve, minecart

**Amaç:** strider (`Strider.getRiddenInput/getRiddenSpeed`, lavın üstünde yürümek: `canStandOnFluid`, `getLiquidCollisionShape`, `floatStrider`, sıcak blok boğulması, yürüme animasyonlu koltuk), deve (`Camel`: koşu, oturma, `canSprint`), minecart sürüşü; binme/inme (`startRiding`, `getDismountLocationForPassenger`, `mount.onboard`).
**Bağımlılık:** T44c1. **Boy:** L (türe göre böl).

## Oku
- Java: `world/entity/monster/Strider.java`, `world/entity/animal/camel/Camel.java`, `world/entity/vehicle/minecart/AbstractMinecart.java`.
- Rust: `voxel-world/src/mount_ride.rs`, `vehicle_ride.rs`.

## Oracle
`movement/ride-pig` örneğiyle türe göre `ride` satırları.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`.

## Belgeler
`docs/client-mount-ride-parity-conformance.md`.

## Devir notu
(boş)
