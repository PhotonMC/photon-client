# T41b — Zırh katmanı (`HumanoidArmorLayer`, `EquipmentLayerRenderer`)

**Amaç:** Giyilen zırhı çiz: göğüs/bacak/ayak/baş zırhı için iç ve dış zırh modelleri, `EquipmentClientInfo` katmanları (doku, boyalı deri rengi), süs (trim) ve büyü parıltısı.
**Bağımlılık:** T41. **Boy:** L (sığmazsa boya/süs/parıltıyı ayır).

## Oku
- Java: `client/renderer/entity/layers/{HumanoidArmorLayer,EquipmentLayerRenderer}.java`, `client/renderer/entity/ArmorModelSet.java`, `client/resources/model/{EquipmentClientInfo,EquipmentAssetManager}.java`, `world/item/equipment/{Equippable,trim/ArmorTrim}.java`, `client/model/geom/LayerDefinitions.java` (zırh katmanı tanımları), `world/item/component/DyedItemColor.java`.
- Rust: `crates/voxel-render/src/mob_model.rs`, `player_model.rs`, `crates/voxel/src/{mobs,overlays,held_items}.rs`, `crates/voxel-world/src/entity.rs` (`equipment`, T41).

## Yapılacaklar
1. Zırh modeli katmanları (`humanoid_armor_*#inner/outer`, bebek) T23 tablosunda var mı bak; yoksa oracle'dan tablo.
2. `equipment/*.json` varlıkları (jar'dan, yerel): katman listesi ve doku yolu; boyalı deri (`DyedItemColor`), süs atlası ve parıltı sonra.
3. `HumanoidArmorLayer` sırası (göğüs, bacak, ayak, baş), iç/dış model seçimi, bebek zırhı.

## Oracle
`EntityOracle.java` `held_item_pose` kalıbı: gerçek katman modeli aynı `setupAnim` ile pozlanır, dörtgenler karşılaştırılır; katman tablosu ve doku yolları her koşuda karşılaştırılır.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`. Canlı: zırhlı zombi/iskelet/oyuncu yan yana.

## Kapsam dışı
Kafa eşyası/pelerin/elytra (T41c), at zırhı (T31).

## Belgeler
`docs/index.md` satırı, yeni `docs/client-*-parity-conformance.md`.

## Devir notu
