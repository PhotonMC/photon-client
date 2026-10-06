# T44b — Binek hayvanlar ve minecart sürüşü

**Amaç:** teknenin dışındaki araçların denetimi: at/eşek/katır (`travelRidden`, `getRiddenInput`, zıplama gücü `PlayerCommand` START/STOP_RIDING_JUMP, `MoveVehicle`), eyerli domuz, deve, strider, minecart; ikinci yolcu, binme/inme (`startRiding`, `getDismountLocationForPassenger`, `mount.onboard`).
**Bağımlılık:** T44a, T31. **Boy:** L (sığmazsa türe göre böl: at ailesi, minecart, öteki).

## Oku
- Java: `world/entity/LivingEntity.java` (`travelRidden`), `world/entity/animal/horse/AbstractHorse.java`, `client/player/LocalPlayer.java` (`getJumpRidingScale`, `sendRidingJump`), `AbstractMinecart`.
- Rust: `voxel-world/src/boat_ride.rs` (örnek), `voxel-world/src/local_player.rs`, `crates/voxel-net/src/connection/play.rs`.

## Oracle
`MovementOracle.localPlayerTicks` `ride` satırları (`movement/ride-boat` örneği), türe göre.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`.

## Kapsam dışı
At eğitimi, iksir etkileri, ses.

## Belgeler
Yeni `docs/client-*-parity-conformance.md` kaydı ve `docs/index.md`.

## Devir notu
(boş)
