# T88 — Kitap açma ve düzenleme (`OpenBook`, `EditBook`)

**Amaç:** Yazılı kitabı elde açma (`ClientboundOpenBookPacket` → `BookViewScreen`; görünüm T56'da
var) ve kitap-tüy düzenleme (`BookEditScreen`, `BookSignScreen`, `ServerboundEditBookPacket`).
**Bağımlılık:** T56. **Boy:** M.

## Oku
- Java: `client/gui/screens/inventory/BookEditScreen.java`, `BookSignScreen.java`, `BookViewScreen.java`,
  `client/gui/components/MultiLineEditBox.java`, `ClientPacketListener.handleOpenBook`,
  `network/protocol/game/ServerboundEditBookPacket.java`.
- Rust: `crates/photon-render/src/gui/screen/book.rs`, `crates/photon-net/src/connection/play/incoming.rs`
  (paket yönlendirme; `grep -n` ile çapa), eşyayı kullanma yolu (`use_item`).

## Yapılacaklar
1. `OpenBook` paketi → eldeki yazılı kitabın görünümü.
2. Düzenleme ekranı: sayfa metni, sayfa ekleme, imleç/seçim (`MultiLineEditBox`), imzalama ekranı.
3. `EditBook` paketi (yuva, sayfalar, başlık).
4. G tarifi: `book_edit_screen`, `book_sign_screen`.

## Oracle
G; paket baytları `wire` ailesinde.

## Kabul
`CHECK OK`.

## Kapsam dışı
IME (K-M5a).

## Belgeler
`docs/gui-oracle.md`.

## Devir notu
