# T83b — Ekran örtüleri ve etki sisleri

**Amaç:** T83'ün kalanı: hasar alırken ekranın kırmızı örtüsü, yanma ve su altı örtüleri (`ScreenEffectRenderer`), gece görüşü ve `Darkness` etkisinin ışık/sis etkisi, körlük sisi, `Poof` ekran etkileri (portal/bulantı dönmesi `spinningEffect`).
**Bağımlılık:** T83. **Boy:** M.

## Oku
- Java: `client/renderer/ScreenEffectRenderer.java`, `client/renderer/fog/*`, `client/renderer/GameRenderer.java` (`spinningEffectTime`, `renderLevel`), `client/renderer/LightmapRenderStateExtractor.java`.
- Rust: `crates/voxel-render/src/{camera,sky,atmosphere}.rs`, `crates/voxel/src/atmosphere.rs`.

## Oracle
Gerçek `ScreenEffectRenderer`/`fog` sayısal çıktısı ve `gui` ailesinde örtü satırları.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`.

## Belgeler
`docs/client-camera-hurt-parity-conformance.md`, `docs/remaining-work.md` §8.

## Devir notu
(boş)
