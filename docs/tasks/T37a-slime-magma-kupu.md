# T37a — Slime ve magma cube

**Amaç:** `SlimeModel` (iç ve dış katman), `MagmaCubeModel.setupAnim`, `AbstractCubeMobRenderer.scale`, `AbstractCubeMob.tick` squish'i, boyut verisi (`ID_SIZE`) ve boyutla ölçeklenen kutu.
**Bağımlılık:** T37. **Boy:** M.

## Oku
- Java: `world/entity/monster/cubemob/{AbstractCubeMob,Slime,MagmaCube}.java`, `client/model/monster/slime/{SlimeModel,MagmaCubeModel}.java`, `client/renderer/entity/{AbstractCubeMob,Slime,MagmaCube}Renderer.java`, `layers/SlimeOuterLayer.java`.
- Rust: `crates/voxel-world/src/{cube_mob,entity}.rs`, `crates/voxel/src/{cube_mobs,squids}.rs` (kalıp).

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`.

## Devir notu
- Yapıldı: `voxel-world/cube_mob.rs` (squish tiki, slime 0,6 / magma 0,9), veri 17 (`cube_size`; her alınışta baş dönüşü `yRot`/`yBodyRot` olur), boyutla ölçeklenen kutu (`dimensions`), `MobModel::setup_magma_cube`, `CubeMobScale` (`scale` kancası), `sort_translucent_quads` (dış katman uzaktan yakına), `crates/voxel/src/cube_mobs.rs`, gölge yarıçapı. Kayıt: `docs/records/client-cube-mob-parity-conformance.md`.
- Oracle: `entity/magma-cube-model` 6, `entity/cube-mob-rotations` 33, `living-entity-animations`'a 6 satır; `head_rotation`, `cube_size` olayları, `cube` ayrıntısı.
- Sapma: ses, iniş parçacığı, görünmezlik, çok slime arasında karışım sırası yok. Canlı gözlem yapılmadı. Sulfur cube T37b'ye kaldı.
