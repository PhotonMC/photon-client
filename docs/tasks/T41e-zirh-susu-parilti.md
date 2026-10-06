# T41e — Zırh süsü (trim), büyü parıltısı ve bileşen yaması

**Amaç:** `EquipmentLayerRenderer`'ın kalan geçişleri: `ArmorTrim` (süs atlası, `trims/entity/<tür>/<desen>_<malzeme>`), `armorEntityGlint` (büyülü parça parıltısı) ve eşya yamasındaki `equippable` (özel varlık kimliği).
**Bağımlılık:** T41b. **Boy:** M.

## Oku
- Java: `client/renderer/entity/layers/EquipmentLayerRenderer.java` (`TrimSpriteKey`, glint), `world/item/equipment/trim/ArmorTrim.java`, `client/renderer/rendertype/RenderTypes.java` (`armorEntityGlint`, `armorTrimsSheet`), `client/renderer/texture/` süs atlası (`atlases/armor_trims.json`).
- Rust: `crates/voxel/src/armor.rs`, `voxel-render/src/equipment.rs`, `voxel-game/src/item_values.rs` (bileşen okuma), `voxel-render/src/vulkan.rs` (entity boru hattı).

## Yapılacaklar
1. Süs: `TRIM` bileşenini oku, süs atlasını (varlık dokuları arasına) kur, her giyilen parça için süs katmanını çiz.
2. Parıltı: `STANDARD` parıltı türü, kayan doku; yeni render türü.
3. `equippable` yaması: `Values::raw(EQUIPPABLE)` çözümü.

## Oracle
`entity/equipment-layers` kalıbı: süs sprite kimliği ve atlas düzeni gerçek sınıflardan.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`.

## Kapsam dışı
Kafa eşyası/pelerin (T41c).

## Belgeler
`docs/client-armor-parity-conformance.md` sapma listesi.

## Devir notu
- Yalnız süs yapıldı: `voxel-render::trim` (veri paketi desen/malzeme, `PalettedPermutations` eşlemesi, sprite adı), `Armor::{trim_images, set_trim_registries, trim_texture}`, süs atlası entity dokusuna eklenir, süs katmanı zırhın üstüne.
- Oracle: `image/trim-sprite-id` (gerçek `ArmorTrim`, 246 satır) ve `image/palette-permutation` (gerçek eşleme); canlıda görüldü. Dekal deseni derinlik eşitliği olmadan çizilir.
- Parıltı ve `equippable` yaması → [T41e2](T41e2-zirh-parilti-equippable.md).
