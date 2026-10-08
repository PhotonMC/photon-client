# T80d — Tabela bloğu (ayakta ve asılı)

**Amaç:** `StandingSignRenderer`, `HangingSignRenderer`, `AbstractSignRenderer`: model, ahşap dokusu, metin yerleşimi (iki yüz), parıltı, balmumu; `BlockEntityData` ile metin.
**Bağımlılık:** T80. **Boy:** L.

## Oku
Java: `client/renderer/blockentity/{AbstractSignRenderer,StandingSignRenderer,HangingSignRenderer}.java`, `state/*SignRenderState.java`. Rust: `crates/voxel-render/src/gui/font.rs`, `crates/voxel/src/world/block_entities.rs`.

## Oracle
Gerçek renderer kaydı `entity/chest-pose` kalıbıyla (poz, ışık, doku, dörtgenler). Java kaynağı önce satır satır okunur.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`.

## Kapsam dışı
Başka blok entity aileleri.

## Belgeler
`docs/records/client-chest-parity-conformance.md`'ye bölüm, `docs/index.md` gerekirse.

## Devir notu
Yapılan: tabela metni (ayakta, duvar, asılı; iki yüz, boya, parlama, kontur, filtre) gerçek renderer'a bit bit eşit (`gui/sign-texts`, GUI ailesi: gerçek font orada). Tahta 26.2'de blok modeli; renderer yalnız metin verir.
Kayıt: [client-chest-parity-conformance.md](../records/client-chest-parity-conformance.md#tabela-t80d).
Sapma: `POLYGON_OFFSET` yok (ad etiketi boru hattı); oyunda filtre hep kapalı; glif/kontur dörtgenleri oracle'sız.
Açık: canlıda görülmedi (Kabul istemiyor); balmumu/düzenleme T89. Kapıda yalnız bilinen Q04 farkı (`movement/particle-engine`).
