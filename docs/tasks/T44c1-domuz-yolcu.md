# T44c1 — Eyerli domuz ve yönetilmeyen yolcu

**Amaç:** havuçlu sopayla yönetilen eyerli domuz (`Pig.getRiddenInput/tickRidden/getRiddenSpeed`) ve oyuncunun yönetmediği araçta yolcu olması (`rideTick`, `Rot` paketi, `MoveVehicle` yok).
**Bağımlılık:** T44b1. **Boy:** M.

## Oku
- Java: `world/entity/animal/pig/Pig.java`, `LocalPlayer.tick`.
- Rust: `voxel-world/src/{mount_ride,vehicle_ride}.rs`.

## Oracle
`movement/ride-pig`, `movement/ride-horse` (eyersiz at).

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`.

## Belgeler
`docs/client-mount-ride-parity-conformance.md`.

## Devir notu
- `MountKind::Pig`, `Controller.holds_carrot_on_a_stick` (oturum ana el ve yuva 45'e bakar), tırmanma geri geldi (domuz tırmanır, at tırmanmaz); `VehicleView.controlled` ve `VehicleKind::{Pig, Other}`; yolcu koltuğu tür tablosundan.
- 16 domuz satırı + eyersiz at satırı gerçek Java'ya karşı bit bit eşit; canlıda sopayla domuz sürüldü.
- Açık: sopanın `boost`'u (rastgele süre), [T44c2](T44c2-strider-deve-minecart.md).
