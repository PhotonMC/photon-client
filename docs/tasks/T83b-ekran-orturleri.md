# T83b — Ekran örtüleri ve etki sisleri

**Amaç:** `ScreenEffectRenderer`: gömülü blok, su altı ve yanma örtüleri. Gece görüşü/`Darkness` ışığı, körlük sisi, portal/bulantı dönmesi ve totem animasyonu [T83c](T83c-etki-sisleri.md)'dedir.
**Bağımlılık:** T83. **Boy:** M.

## Oku
- Java: `client/renderer/ScreenEffectRenderer.java`, `client/renderer/fog/*`, `client/renderer/GameRenderer.java` (`spinningEffectTime`, `renderLevel`), `client/renderer/LightmapRenderStateExtractor.java`.
- Rust: `crates/photon-render/src/{camera,sky,atmosphere}.rs`, `crates/photon/src/world/atmosphere.rs`.

## Oracle
Gerçek `ScreenEffectRenderer`/`fog` sayısal çıktısı ve `gui` ailesinde örtü satırları.

## Kabul
`tools/photon.py check --capture` → `CHECK OK`.

## Belgeler
`docs/records/client-screen-effects-parity-conformance.md`.

## Devir notu
Yapıldı: `view_blocking_state` (movement oracle `view_block` bit bit eşit), `photon-render/screen_effect.rs` (blok/su/ateş dörtgenleri; `entity/screen-effect-quads` eşit), `ScreenEffect` boru hattı ve `underwater.png`, `photon/overlays/screen_effect.rs`. Canlıda ateş ve su görüldü. Sapma: su parlaklığı gökyüzü kararmasını taşımaz, oyuncu ölçeği 1. Açık: T83c.
