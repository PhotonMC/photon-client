# T95b — Kostüm ekranı ve oyuncu bilgisi

**Amaç:** `SkinCustomizationScreen` (yedi model parçası ve ana el), seçeneklerde `modelPart_*` ve `mainHand`, ve `ClientInformation`'ın bunlardan kurulup değişince sunucuya yeniden gönderilmesi. [T95](T95-kalan-ayar-ekranlari.md)'ten ayrıldı.
**Bağımlılık:** T06. **Boy:** M.

## Oku
Java: `client/gui/screens/options/SkinCustomizationScreen.java`, `client/Options.java` (`modelParts`, `mainHand`, `buildPlayerInformation`). Rust: `crates/photon-render/src/gui/options/chat.rs` (örnek), `crates/photon/src/controls/settings.rs`.

## Devir notu
Yapıldı: `Options::{model_parts, main_hand}` ve dosya okuma/yazma (`modelPart_*`, `mainHand`); `SkinCustomizationScreen` (`photon-render/.../options/skin.rs`) gerçek Java ekranına karşı `skin_customization_screen` oracle'ında (`client-parity`, 15 satır: parçalar, ana el, tıklama, Shift); oyunda Seçenekler → Kostüm Özelleştirme, kapanınca kayıt ve `broadcastClientInformation` (`controls/skin.rs`, `client_information`).
Sapma/açık: sohbet renkleri, sunucu listesi izni, parçacık durumu ve metin süzme hâlâ varsayılan sabitlerle gidiyor (T95c); ana el yalnız sunucuya bildirilir, birinci şahıs elinin soldan çizilmesi ve oyuncu modelinin kolu ayrı iştir; `Tab`/ok odağı diğer ayar ekranlarındaki gibi.
