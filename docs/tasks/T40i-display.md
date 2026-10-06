# T40i — Display entity'leri

**Amaç:** `block_display`, `item_display`, `text_display` (`DisplayRenderer`): dönüşüm, aradeğerleme, billboard, parlaklık, gölge, metin arka planı. Bu görevde block ve item display'leri; metin [T40i2](T40i2-metin-display.md), gölge/ayıklama/parıltı [T40i3](T40i3-display-golge-ayiklama.md).
**Bağımlılık:** T40. **Boy:** L.

## Oku
- Java: `client/renderer/entity/DisplayRenderer.java`, `world/entity/Display.java`.

## Oracle
Gerçek renderer/model sınıfı ve durumu (örnek: `docs/client-arrow-parity-conformance.md`, `docs/client-minecart-parity-conformance.md`).

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`. Canlı: `summon`.

## Kapsam dışı
Sesler, partiküller, dış çizgi.

## Belgeler
`docs/index.md` satırı, yeni kayıt.

## Devir notu
- Yapıldı: `Display` verisi, render durumu, `Transformation.slerp` aradeğerlemesi, ilerleme (`voxel-world/display.rs`; oracle'da gerçek `Display` varlığı + gerçek dinleyici), konum/dönüş aradeğerleme uzunluğu entity başına (`InterpolationHandler`), yerleşim (`voxel-render/display.rs`; gerçek `DisplayRenderer.submit`, normal matrisi dahil), `Pose::mul_matrix` genel, oyunda `displays.rs`. Canlıda doğrulandı. Kayıt: `docs/client-display-parity-conformance.md`.
- Sapma/açık: metin display'i T40i2; gölge, ayıklama, parıltı, ışık geçersiz kılma ve görüş menzili oracle'ı T40i3; oyun içi çizim oracle'sız.
