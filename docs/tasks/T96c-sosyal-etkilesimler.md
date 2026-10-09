# T96c — Sosyal etkileşimler ekranı

**Amaç:** `SocialInteractionsScreen` (oyuncu listesi, gizleme, sekmeler; P tuşu); gizleme sohbet filtresine bağlanır.
**Bağımlılık:** T85. **Boy:** M. [T96](T96-kredi-sosyal-oyun-modu.md)dan ayrıldı.

## Oku
Java: `client/gui/screens/social/*.java`. Rust: tablist oyuncu bilgisi, sohbet günlüğü.

## Kapsam dışı
Raporlama ekranları (çevrimiçi servis).

## Oracle
G tarifi: `social_interactions_screen`.

## Kabul
`tools/photon.py check --capture` → `CHECK OK`.

## Belgeler
`docs/gui-oracle.md`.

## Devir notu

Yapıldı: `SocialInteractionsScreen` çizimi ve oracle'ı (`social_interactions_screen`, `client-parity`, 62/62 gui): sayfalar, arama süzgeci, sıralama, yüzler (varsayılan skin, atlasa 18 doku eklendi), durum satırları, düğme sprite'ları, kırpma, kaydırma çubuğu, ilk karedeki engelleme düğmesi. Rust: `photon-render/src/gui/screen/social.rs`.
Düzeltme: `Place::scroller_height` kısa listede (`height - 8 < 32`) Java'nın `min(max(..))` sırasını izlemiyordu.
Sapma/açık: sohbet günlüğü profilleri, düğme ipuçları (500 ms), özel skinler, odak kaybında arama ipucu çizilmez. Tıklama/klavye, P tuşu ve gizlemenin sohbet süzgecine bağlanması [T96d](T96d-sosyal-baglama.md)'dedir.
