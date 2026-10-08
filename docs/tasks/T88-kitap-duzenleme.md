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
- Yapıldı: `MultilineTextField` ve `StringSplitter.splitLines` (`widgets/multiline.rs`), `MultiLineEditBox` kaydırma ve kaydırma çubuğuyla (`widgets/text_area.rs`), `BookEditScreen` (`screen/book_edit.rs`: sayfa kutusu, sayfa düğmeleri, Page Up/Down, Tab odağı, Sign/Done). Kitap-tüy kullanılınca ekran açılır (`openItemGui`, kol sallanır), Done `EditBook` (24) gönderir, Escape kaydetmez. G: `gui/book-edit-screen` (53 satır) yeşil.
- Sapma: ok tuşlarıyla odak gezintisi (düğme odaktayken) yok; çift tıklama oyunda algılanmıyor (alan destekliyor); `updateLocalCopy` yok (sunucunun yuva güncellemesi bekleniyor); sayfa çevirme sesi yok.
- Bölündü: imzalama ekranı ve `OpenBook` → T88b. Sign düğmesi şimdilik bir şey yapmaz.
- Açık: canlıda denenmedi; `§` biçim kodlu sayfaların genişliği yalnız kodu atlayarak hesaplanır.
