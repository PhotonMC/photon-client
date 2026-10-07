# T40h — Tablo

**Amaç:** `Painting` (`PaintingRenderer`): `PaintingVariant` kaydı ve atlası, yön ve boyut.
**Bağımlılık:** T40. **Boy:** M.

## Oku
- Java: `client/renderer/entity/PaintingRenderer.java`, `world/entity/decoration/painting/Painting.java`.

## Oracle
Gerçek renderer/model sınıfı ve durumu (örnek: `docs/records/client-arrow-parity-conformance.md`, `docs/records/client-minecart-parity-conformance.md`).

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`. Canlı: `summon`.

## Kapsam dışı
Sesler, partiküller, dış çizgi.

## Belgeler
`docs/index.md` satırı, yeni kayıt.

## Devir notu
- Yapıldı: çeşit tablosu (`painting_variants.py`, kayıttan), `painting` geometrisi (oracle'da gerçek `renderPainting`), `paintings.rs`; bağ düğümü konumu da blok merkezine düzeltildi. Kayıt: `docs/records/client-painting-parity-conformance.md`.
- Sapma: asılma konumu render anında, oracle'sız; entity konumu/vuruş kutusu hâlâ blok köşesi; ışık kuralı oracle'sız; z-ofset yok.
