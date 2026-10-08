# T88b — Kitap imzalama ve `OpenBook`

**Amaç:** Kitap-tüyün imzalama ekranı (`BookSignScreen`: ortalanmış başlık kutusu, Finalize/Cancel,
`ServerboundEditBookPacket` başlıkla) ve sunucunun açtığı yazılı kitap (`ClientboundOpenBookPacket`
→ eldeki kitabın `BookViewScreen`'i; görünüm T56'da).
**Bağımlılık:** T88. **Boy:** M.

## Oku
- Java: `client/gui/screens/inventory/BookSignScreen.java`, `BookViewScreen.java`,
  `client/gui/components/EditBox.java` (`setCentered`, `setBordered`), `ClientPacketListener.handleOpenBook`.
- Rust: `crates/photon-render/src/gui/screen/book_edit.rs` (`BookEditAction::Sign`),
  `crates/photon/src/controls/book.rs`, `crates/photon-render/src/gui/widgets/edit_box.rs`,
  `crates/photon-render/src/gui/screen/book.rs`.

## Yapılacaklar
1. `EditBox`'a ortalanmış ve çerçevesiz çizim; `BookSignScreen` (Cancel düzenleme ekranına döner,
   başlık korunur; Enter ile imza).
2. `edit_book(hand, pages, Some(title))`.
3. `OpenBook` (kimlik 58) → eldeki yazılı kitabın görünümü (menüsüz `BookScreen` kipi).
4. G tarifi: `book_sign_screen`; `book_view` eldeki kitapla.

## Oracle
G.

## Kabul
`CHECK OK`.

## Kapsam dışı
IME (K-M5a).

## Belgeler
`docs/gui-oracle.md`.

## Devir notu
- Yapıldı: `BookSignScreen` (`screen/book_sign.rs`; `EditBox`'a gölgesiz çizim, seçim vurgusu Java'nın `textHighlight`'ı oldu), Sign → imzalama, Cancel başlığı tutup sayfalara döner (`rebuildWidgets` odağı yeniden verir), Sign and Close `EditBook`'u başlıkla ve sayfaları olduğu gibi gönderir. `OpenBook` (58) → `BookScreen::held` (sayfa yerel, kapanınca paket yok; `Screen::is_container`). G: `gui/book-sign-screen` (24), `gui/book-view` (15) yeşil.
- Sapma: eldeki kitapta `run_command`/`open_url` tıklamaları yok (kürsüde de yok); sayfa sesi yok.
- Açık: canlıda denenmedi.
