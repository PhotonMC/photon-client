# T37b — Sulfur cube

**Amaç:** `SulfurCubeModel`/`SmallSulfurCubeModel`, `SulfurCubeInnerLayer`, `SulfurCubeRenderer.scale` (fitil şişmesi, kabuk oturması), `SulfurCube` fitil tiki ve `MAX_FUSE` verisi.
**Bağımlılık:** T37a. **Boy:** M.

## Oku
- Java: `world/entity/monster/cubemob/SulfurCube.java`, `client/model/monster/slime/{SulfurCubeModel,SmallSulfurCubeModel}.java`, `client/renderer/entity/{SulfurCubeRenderer,TntRenderer,layers/SulfurCubeInnerLayer}.java`.
- Rust: `crates/voxel/src/mobs/monsters/cube_mobs.rs`, `crates/voxel-world/src/mobs/cube_mob.rs`, `crates/voxel-render/src/entity/mob_model.rs` (`CubeMobScale`).

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`.

## Devir notu
- Yapıldı: fitil durumu (`MAX_FUSE` 18, tik, sıfırda kalır), `SulfurCubeScale` ve `tnt_swell_amount`/`tnt_is_lit` (T40b TNT'si de kullanır), iç gövde (fitil yanıksa beyaz) ve kabuk ayrı sıralı translucent, küçük model bebek bayrağıyla. Kayıt: `docs/records/client-sulfur-cube-parity-conformance.md`.
- Oracle: `cube-mob-rotations`'a 36, `living-entity-animations`'a 3 satır; `sulfur_fuse` olayı.
- Kalan: taşıdığı blok ve squish'in sıfırlanması (T37b2). Canlı gözlem yapılmadı.
