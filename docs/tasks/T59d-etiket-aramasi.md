# T59d — Yaratıcı aramada `#etiket`

**Amaç:** `CreativeModeInventoryScreen.refreshSearchResults`'ın `#` ile başlayan araması: eşya etiketlerinin
adlarıyla eşleşen eşyalar (sunucunun gönderdiği etiket tablosu).
**Bağımlılık:** T59. **Boy:** S.

## Oku
Java: `client/gui/screens/inventory/CreativeModeInventoryScreen.java` (`updateVisibleTags`,
`refreshSearchResults`). Rust: `crates/photon-game/src/menus/creative/search.rs`.

## Oracle
Yaratıcı arama oracle satırları (G tarifi).

## Kabul
`tools/photon.py check --capture` → yalnız bilinen Q04 farkı dışında yeşil.

## Belgeler
`docs/records/generic-container-conformance.md`, `docs/gui-oracle.md` tablosu, `docs/remaining-work.md` §3.

## Devir notu
- `photon-game::menus::creative::{tag_search, visible_tags}`: `IdSearchTree` ve
  `updateVisibleTags`; ekran (`CreativeScreen::new`) açılışta sunucunun eşya etiketlerini alır,
  ipucuna koyu mor `#etiket` satırlarını ekler.
- Oracle `block/creative-tag-search` (gui değil: vanilla etiketleri bağlı olan block ailesi);
  Java'nın kendi `IdSearchTree`'si ve özel `updateVisibleTags`'i, 9 satır birebir.
- Sapma: birden çok etiket satırının sırası Java'da kimlik hash'ine bağlı; Rust sıralı.
- Açık: etiket satırlı ipucu çizimi oracle'da yok; yaratıcı ad araması da hâlâ oracle'sız.
