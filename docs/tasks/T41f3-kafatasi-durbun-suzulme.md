# T41f3 — Oyuncunun gövde dönüşü: süzülme eğimi ve riptide

**Amaç:** `AvatarRenderer.setupRotations` ve `LivingEntityRenderer.setupRotations`'ı gerçek yığınla karşılaştır; süzülen (`fallFlyingScale`, `flyingYRot`) ve riptide dönen oyuncunun gövdesi.
**Bağımlılık:** T41f2. **Boy:** S.

## Oku
- Java: `client/renderer/entity/player/AvatarRenderer.java` (`setupRotations`, `extractFlightData`), `client/renderer/entity/LivingEntityRenderer.java`, `client/renderer/entity/state/AvatarRenderState.java`.
- Rust: `voxel-render/src/player_renderer.rs` (`placement_pose`), `crates/voxel/src/{arm_poses,overlays}.rs`.

## Oracle
`entity/avatar-pose` (gerçek `setupRotations` + `submit`'in sonu).

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`.

## Belgeler
`docs/records/client-held-item-parity-conformance.md`.

## Devir notu
- `Placement::{fall_flying, auto_spin}`; `entity/avatar-pose` 26 satır bit bit eşit (ölüm, yüzme, sürünme, süzülme ve yönü, riptide). Oyunda `fall_flying_ticks`/`previous_motion` sayaçları ve `arm_poses::fall_flying`; süzülme durumunun çıkarımı Java'ya karşı karşılaştırılmadı, canlı gözlem yok.
- Açık → [T41f4](T41f4-kafatasi-durbun.md).
