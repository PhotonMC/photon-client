# T41c — Kafa eşyası, pelerin, elytra, dürbün

**Amaç:** `CustomHeadLayer` (kafadaki blok/eşya, kafa kafatasları), `CapeLayer`, `WingsLayer` (elytra) ve oyuncunun dürbünle gözüne tuttuğu eşya (`PlayerItemInHandLayer.renderItemHeldToEye`).
**Bağımlılık:** T41. **Boy:** M.

## Oku
- Java: `client/renderer/entity/layers/{CustomHeadLayer,CapeLayer,WingsLayer,PlayerItemInHandLayer}.java`, `client/model/object/equipment/ElytraModel.java`, `client/renderer/entity/player/AvatarRenderer.java` (`extractCapeState`).
- Rust: `crates/photon/src/equipment/held_items.rs` (`Hands`, `HeldItem`), `photon-render/src/held_item.rs`, `player_model.rs`.

## Yapılacaklar
1. Baş yuvası: `Head` görüntü bağlamı ile eşya/blok, kafa dönüşümü (`translateToHead`).
2. Elytra modeli ve `elytraAnimationState`; pelerin dokusu PlayerInfo'dan (PLAN-M9) gelmeden yalnız elytra.

## Oracle
`held_item_pose` kalıbı: gerçek katman, gerçek model; matris ve dörtgen karşılaştırma.

## Kabul
`tools/photon.py check --capture` → `CHECK OK`.

## Kapsam dışı
Pelerin dokusu indirme (PLAN-M9), zırh (T41b).

## Belgeler
`docs/index.md` satırı, yeni kayıt.

## Devir notu
- Yapıldı (kısmen): kafa eşyası (`CustomHeadLayer`: `head_item_pose`, `Hands.head`, `Place::Head`, `Armor::head_item`; piglin dönüşümü) oyuncu, zombi/iskelet/piglin ve zırh askısında; `entity/held-item-pose`'a 23 kafa satırı. Kayıt: `docs/records/client-held-item-parity-conformance.md`.
- Sapma: kafatası modelleri (`SkullBlockRenderer`, oyuncu kafası profili), köylü kafa dönüşümü kullanılmıyor (köylü çizilmiyor).
- Açık: elytra (`WingsLayer`, `ElytraModel` + `elytraAnimationState`), pelerin (PLAN-M9), dürbün gözde (`renderItemHeldToEye`), kafatası modelleri.
