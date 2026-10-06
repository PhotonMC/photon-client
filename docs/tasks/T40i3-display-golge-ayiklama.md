# T40i3 — Display gölgesi, ayıklama ve parıltı

**Amaç:** Display'lerin `shadowRadius`/`shadowStrength` ile gölgesi (`getShadowRadius/Strength`, T42 gölge hattı), frustum ayıklaması (`getBoundingBoxForCulling`, `affectedByCulling`: genişlik/yükseklik), ışık geçersiz kılmanın ve görüş menzilinin oracle'ı, `glow_color_override` ve dış çizgi.
**Bağımlılık:** T40i, T42. **Boy:** M.

## Oku
- Java: `client/renderer/entity/DisplayRenderer.java` (`getShadowRadius`, `getBoundingBoxForCulling`, `getSkyLightLevel`), `world/entity/Display.java` (`updateCulling`, `shouldRenderAtSqrDistance`, `getTeamColor`).
- Rust: `crates/voxel/src/displays.rs`, `crates/voxel-render/src/shadow.rs`.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`.

## Devir notu
(boş)
