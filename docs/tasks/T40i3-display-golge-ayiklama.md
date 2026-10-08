# T40i3 — Display gölgesi, ayıklama ve parıltı

**Amaç:** Display'lerin `shadowRadius`/`shadowStrength` ile gölgesi (`getShadowRadius/Strength`, T42 gölge hattı), frustum ayıklaması (`getBoundingBoxForCulling`, `affectedByCulling`: genişlik/yükseklik), ışık geçersiz kılmanın ve görüş menzilinin oracle'ı, `glow_color_override` ve dış çizgi.
**Bağımlılık:** T40i, T42. **Boy:** M.

## Oku
- Java: `client/renderer/entity/DisplayRenderer.java` (`getShadowRadius`, `getBoundingBoxForCulling`, `getSkyLightLevel`), `world/entity/Display.java` (`updateCulling`, `shouldRenderAtSqrDistance`, `getTeamColor`).
- Rust: `crates/photon/src/objects/displays.rs`, `crates/photon-render/src/entity/shadow.rs`.

## Kabul
`tools/photon.py check --capture` → `CHECK OK`.

## Devir notu
- Yapıldı: display gölgesi (`displays::display_shadow`, mevcut gölge hattına bir kaynak), `shouldRenderAtSqrDistance` (oracle'da gerçek varlık, altı mesafe) ve çizimde uygulanması, ışık geçersiz kılma (`photon-render::display::displayed_light`, gerçek `DisplayRenderer.getBlockLightLevel/getSkyLightLevel`'a karşı 6 satır). Kayıt: `docs/records/client-display-parity-conformance.md`.
- Sapma: frustum ayıklaması yapılmadı (istemcide hiçbir varlık için yok); `glow_color_override` ve dış çizgi kapsam dışı; gölge canlıda görülemedi (düz zemin yok), gölge çizimi oracle'sız.
