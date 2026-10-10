# T93 — Hata ayıklama katmanı (F3) ve işlev tuşları

**Amaç:** `DebugScreenOverlay` ve 26.2'nin `DebugScreenEntries` sistemi (girdi başına satır üreticisi,
görünürlük profilleri, `DebugOptionsScreen`), sol/sağ sütun çizimi; F3 kombinasyonları
(F3+H gelişmiş ipucu, F3+B vuruş kutuları, F3+G chunk sınırları kapsam dışı). F1 ve F2 [T93a](T93a-f1-f2.md)'de bitti.
**Bağımlılık:** — . **Boy:** L.

## Oku
- Java: `client/gui/components/DebugScreenOverlay.java`, `client/gui/components/debug/*.java`
  (`DebugScreenEntries`, `DebugEntry*`), `client/gui/screens/debug/DebugOptionsScreen.java`,
  `client/KeyboardHandler.java` (F3 kombinasyonları, F1, F2), `client/Screenshot.java`.
- Rust: `crates/photon/src/controls/events.rs` (F5 örneği), `crates/photon-render/src/gui/hud.rs`,
  `crates/photon/src/profile.rs` (FPS/kare süresi).

## Yapılacaklar
1. Girdi tablosu: Java'daki girdilerden istemcinin hesaplayabildikleri (konum, bakış, blok, biyom,
   ışık, chunk, fps, sürüm…); sunucu verisi isteyenler (TPS, spawn) Java'daki "yok" yazısıyla.
2. Çizim: sütunlar, arka plan dikdörtgenleri; F3 açık/kapalı ve profiller.
3. F3+H.
4. G tarifi: `debug_overlay` (değerler sabitlenmiş).

## Oracle
G (satır metinleri ve düzen); ekran görüntüsü adı kuralı birim testi.

## Kabul
`CHECK OK`; oyunda F3 (`shot`).

## Kapsam dışı
Profil grafikleri (F3+2 vb. pasta/çubuk), F3+B/G çizimleri (ayrı görev).

## Belgeler
`docs/gui-oracle.md`.

## Devir notu
