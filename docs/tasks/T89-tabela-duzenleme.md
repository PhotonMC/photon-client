# T89 — Tabela düzenleme (`OpenSignEditor`, `SignUpdate`)

**Amaç:** `SignEditScreen`/`HangingSignEditScreen`: tabela modelinin ekranda çizimi, dört satır
düzenleme, imleç; `ServerboundSignUpdatePacket`. Tabelanın dünyada çizimi T80d'dedir.
**Bağımlılık:** T80d. **Boy:** M.

## Oku
- Java: `client/gui/screens/inventory/AbstractSignEditScreen.java`, `SignEditScreen.java`,
  `HangingSignEditScreen.java`, `client/gui/font/TextFieldHelper.java`,
  `ClientPacketListener.handleOpenSignEditor`.
- Rust: T80d tabela çizicisi, `crates/photon-render/src/gui/screen/` kalıbı.

## Yapılacaklar
1. Paket → ekran (ön/arka yüz), satır düzenleme ve imleç yanıp sönmesi.
2. Kapatınca `SignUpdate`.
3. G tarifi: `sign_edit_screen`, `hanging_sign_edit_screen`.

## Oracle
G.

## Kabul
`CHECK OK`.

## Kapsam dışı
Renkli/parlak tabela boyası (dünya çiziminde).

## Belgeler
`docs/gui-oracle.md`.

## Devir notu
- Yapıldı: `TextFieldHelper` (`widgets/text_field.rs`), `SignEditScreen`/`HangingSignEditScreen` (`screen/sign_edit.rs`; tahta `textures/gui/{signs,hanging_signs}` atlasa eklendi), `textHighlight` için `GUI_TEXT_HIGHLIGHT` (ADDITIVE) GUI boru hattı. Ağ: `OpenSignEditor` (60) ve `SignUpdate` (61). Oyunda: paket → ekran (`controls/sign.rs`), kapanınca `SignUpdate`, tik geçerliliği (tabela yok ya da erişim + 4 dışında), yazılan satır dünyadaki tabelada anında. G: `gui/sign-edit-screen` (58 satır) yeşil.
- Oracle: seviyesiz `SignBlockEntity`'de `setText` alanı doğrudan yazan alt sınıfla geçildi. Oracle `num` biçimi Java `Float.toString`'in eşitlikte çift hane kuralına uyduruldu (Rust yukarı yuvarlıyordu).
- Sapma: Tab yalnız Done'a odak verir; Space odaktaki Done'u basar. Java'nın kapsayıcı ekranı varken `setScreen` kapatma sırası taklit edilmedi (sohbet/duraklatma kapanır).
- Açık: IME önizleme (`preeditOverlay`) ve bidi şekillendirme yok; panoya kopyalama/yapıştırma oracle'da yok (birim testte). Canlıda denenmedi.
