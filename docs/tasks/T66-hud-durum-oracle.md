# T66 — HUD durum çubukları oracle'ı

**Amaç:** `Hud.extractPlayerHealth` (zırh, kalpler, yiyecek, hava) ve deneyim çubuğunu (`ExperienceBar`, `extractExperienceLevel`) gerçek `Hud`'a karşı karşılaştır; kalpler yalnız yerleşimle taşınmıştı (`hud-conformance.md`).
**Bağımlılık:** T45 (`hud_hotbar` oracle düzeneği). **Boy:** M.

## Oku
- Java: `client/gui/Hud.java` (`extractPlayerHealth`, `extractHearts`, `extractFood`, `extractAirBubbles`, `HeartType`), `client/gui/contextualbar/{ContextualBar,ExperienceBar}.java`.
- Rust: `crates/photon-render/src/gui/hud.rs` (`player_health`, `contextual_bar`), `docs/records/hud-conformance.md`.

## Yapılacaklar
`GuiOracle.hudHotbar`'ın sahnesini ayır, `hud_status` işlemini ekle: gerçek `LocalPlayer`'a can, en yüksek can, emilim, zırh, yiyecek/doygunluk, hava, etkiler (yenilenme, zehir, solgunluk, açlık), donma, zorlu mod, dokunulmazlık ve deneyim ver; kare adımlarıyla (tik, milisaniye, can) yanıp sönmeyi sürdür. Rust: `Hud::status` kamusal.

## Oracle
`GuiOracle.java` `hud_status`; `cases_gui.py` `gui_hud_status_cases`; `gui.rs` kolu; `gui-oracle.md` satırı.

## Kabul
`tools/photon.py check --capture` → `CHECK OK`.

## Kapsam dışı
Araç canı, atlama çubuğu, konum çubuğu (kendi oracle'ı var), efekt simgeleri, seçili eşya adı.

## Belgeler
`docs/records/hud-conformance.md`, `docs/gui-oracle.md`.

## Devir notu
- `GuiOracle.hudScene` (ortak sahne) + `hudStatus`; `hud_status` 150 satır, ilk denemede bit bit eşit çıktı (yalnız Java'da emilim `MAX_ABSORPTION` ile sınırlıydı: oracle özniteliği yükseltir).
- Rust: `Hud::{player_health, contextual_bar}` kamusal; HUD sprite'ları günlüğe yazılır (`sprite_blend`, `sprite_region_blend`).
- Açık: araç canı, atlama çubuğu, etki simgeleri, seçili eşya adı; kalp yanıp sönmesi gerçek saatle (`Util.getMillis`) oracle'da sahte saatle karşılaştırıldı.
