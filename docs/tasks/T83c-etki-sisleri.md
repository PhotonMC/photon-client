# T83c — Etki sisleri, ışık etkileri ve dönme

**Amaç:** Gece görüşü ve `Darkness` etkisinin ışık haritasına etkisi, körlük ve karanlık sisi (`client/renderer/fog/*`), portal/bulantı dönmesi (`spinningEffect`), totem etkinleştirme animasyonu (`displayItemActivation`).
**Bağımlılık:** T83b. **Boy:** L (sığmazsa böl).

## Oku
Java: `client/renderer/fog/*`, `client/renderer/GameRenderer.java` (`spinningEffectTime`, `renderLevel`), `client/renderer/LightmapRenderStateExtractor.java`, `client/renderer/ScreenEffectRenderer.java` (`renderItemActivationAnimation`). Rust: `crates/photon-render/src/{camera,sky}.rs`, `crates/photon/src/world/atmosphere.rs`.

## Kabul
`tools/photon.py check --capture` → `CHECK OK`.

## Devir notu
(boş)
