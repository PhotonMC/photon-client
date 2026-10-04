# T37b2 — Sulfur cube'un taşıdığı blok

**Amaç:** `SulfurCube.getBodyArmorItem` (gövde ekipmanı, `SetEquipment` BODY), `BlockItemStateProperties` ile blok durumu, `SulfurCubeInnerLayer`'ın blok dalı (`XP` 180°, `translate(-0,5, -0,518, -0,5)`, bebekte `0,5` ölçek, iç gövdenin yerine), `applySizeAndSquish`'in squish'i sıfırlaması.
**Bağımlılık:** T37b, T34d (varlık üstünde blok durumu). **Boy:** M.

## Oku
- Java: `client/renderer/entity/{SulfurCubeRenderer,layers/SulfurCubeInnerLayer}.java`, `world/entity/monster/cubemob/SulfurCube.java` (`hasBodyItem`, `canUseSlot`), `world/item/component/BlockItemStateProperties.java`.
- Rust: `crates/voxel/src/cube_mobs.rs`, `crates/voxel/src/` (T34d blok çizimi), `crates/voxel-world/src/entity.rs` (ekipman).

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`.

## Devir notu
- Yapıldı: gövde (BODY) yuvasındaki eşyadan blok durumu (`block_item_state`, bakır golemden ortak yardımcıya taşındı), `SulfurCubeScale.holds_block` ile squish sıfırlama, `sulfur_cube_block_pose` ve iç gövdenin yerine blok. Kayıt: `docs/client-sulfur-cube-parity-conformance.md`.
- Oracle: `cube-mob-rotations`'a 8 satır; gerçek `SulfurCubeInnerLayer`'ın blok matrisi ve boş olmayan `containedBlock` ile gerçek `scale`. Canlı gözlem yapılmadı.
- Java'dan sapma: bloğun kaplaması (hasar kırmızısı, fitil beyazı) yok; blok yerdeki eşya hattından geçer, o hat kaplama taşımaz (T37b3). Modeli olmayan bir blok durumu (hava dışında) boş sayılmaz.
- Kalan: arketipler.
