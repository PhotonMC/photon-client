# T41e2 — Zırh büyü parıltısı ve `equippable` yaması

**Amaç:** `armorEntityGlint` (büyülü parçanın kayan parıltısı: `hasFoil`, `enchanted_glint_armor` dokusu, doku matrisi, ışıksız karışım; yeni boru hattı). Eşya yamasındaki `equippable` [T41e3](T41e3-equippable-yamasi.md)'e ayrıldı.
**Bağımlılık:** T41e. **Boy:** M.

## Oku
- Java: `client/renderer/entity/layers/EquipmentLayerRenderer.java`, `client/renderer/rendertype/RenderTypes.java` (`armorEntityGlint`), `world/item/ItemStack.java` (`hasFoil`), `world/item/component/Equippable.java`.
- Rust: `crates/voxel/src/armor.rs`, `voxel-render/src/vulkan.rs` (entity boru hattı), `voxel-game/src/item_values.rs`.

## Oracle
`entity/equipment-layers` satırları (`hasFoil` kararı), parıltı dokusu matrisi için sayısal oracle.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`.

## Belgeler
`docs/records/client-armor-parity-conformance.md`.

## Devir notu
- Yapıldı: `ItemStack.hasFoil` (`Values::has_foil`), glint doku matrisi ve boru hattı bilgisi (`render_types::{GLINT, glint_texture_matrix}`), `PipelineKind::Glint` (Vulkan, yeni gölgelendirici), zırh ve elytra katmanının ilk katmanında parıltı (`Armor::glint`). Kayıt: `docs/records/client-armor-parity-conformance.md`. Canlıda görüldü.
- Oracle: `entity/item-foil` (gerçek `hasFoil`, her eşya + yamalar), `entity/glint-layers` (gerçek render türü ve doku matrisi); `entity` ailesine `netty-transport` eklendi (gerçek `Minecraft` sınıfı için).
- Sapma: derinlik `EQUAL` yerine büyük-eşit; parıltı gücü/hızı seçenekleri sabit varsayılan; eyer/kafa/el eşyasında parıltı yok; `equippable` yaması T41e3.
