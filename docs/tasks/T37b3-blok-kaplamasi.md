# T37b3 — Varlık üstündeki bloğun kaplaması

**Amaç:** `BlockModelRenderState.submit`'in `overlayCoords`'u: sulfur cube'un taşıdığı blokta hasar kırmızısı ve yanık fitilin beyazı (`SulfurCubeInnerLayer`), aynı yolu kullanan öteki katmanlarda (enderman bloğu, kar golemi balkabağı) varsa kaplama. Şu an `HeldItem::block` yerdeki eşya hattından geçer; o hat (`set_ground_items`, `VERTEX_SHADER`) kaplama taşımaz.
**Bağımlılık:** T37b2. **Boy:** M.

## Oku
- Java: `client/renderer/block/BlockModelRenderState.java` (`submit`), `layers/{SulfurCubeInnerLayer,CarriedBlockLayer}.java`, `OverlayTexture.java`.
- Rust: `crates/voxel/src/held_items.rs` (`Carried::Block`), `crates/voxel-render/src/{item_entity,vulkan}.rs` (`entity_block_vertices`, entity ve yer hatları).

## Yapılacaklar
Önce hangi katmanların gerçekten kaplama verdiğini Java'da doğrula; kaplamayı taşıyan hatta (entity shader'ı `in_shade`'i kaplama kodu olarak okur) blok köşelerini çiz ya da yer hattına kaplama ekle. Çözüm geçici olmasın; oracle'a kaplama kodunu ekle.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`.

## Devir notu
(boş)
