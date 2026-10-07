# T37h2 — Ender ejderhasının ölümü

**Amaç:** `EnderDragonRenderer`'ın ölüm çizimi: `entityCutoutDissolve` (çözünme maskesi `dragon_exploding.png`, alfa `1 − ölümZamanı/200`), ölüm ışınları (`dragonRays`, tohumu 432 olan rastgele ışın demeti), ölümde `partialTicks = 0`, `dragonDeathTime` (T37h'de sayılıyor), `EXPLOSION` parçacıkları dışında.
**Bağımlılık:** T37h. **Boy:** M.

## Oku
- Java: `client/renderer/entity/EnderDragonRenderer.java` (`submitRays`), `RenderTypes.entityCutoutDissolve`/`dragonRays`, `shaders/core/entity.fsh` (`DISSOLVE`).
- Rust: `crates/voxel/src/mobs/monsters/ender_dragons.rs`, `crates/voxel-render/src/vulkan.rs` (entity gölgelendiricisi).

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`.

## Devir notu
- Yapıldı: çözünme (`PipelineKind::Dissolve`, `mob_vertices_dissolving`, maske atlasta itme sabitiyle) ve ışınlar (`dragon_rays.rs` + `Quaternion::rotate_xyz`, `PipelineKind::Rays`), `ender_dragons.rs`'te ölüm dalı. İtme sabiti aralığı 164'ten 176 bayta çıktı. Kayıt: `docs/records/client-ender-dragon-death-parity-conformance.md`.
- Sapma: çözünme ve ışın boru hattı oracle'sız; ölen ejderha canlı görülemedi (oyun açılıp ejderha çizildi, ölüm karesi yakalanamadı).
- Açık: `EXPLOSION` parçacıkları ve sesler.
