# T80b — Yerleştirilmiş kafa ve kafatası blokları

**Amaç:** `SkullBlockRenderer` (kafatası, zombi/creeper/ejderha/piglin kafası, oyuncu kafası profili ve dönüş, ejderha çenesi ve piglin kulağı animasyonu, güçle güçlenen güç noktası) — ekipman olarak çizilen modeller `skull.rs`'dedir.
**Bağımlılık:** T80. **Boy:** T79.

## Oku
Java: `client/renderer/blockentity/SkullBlockRenderer.java`, `state/SkullBlockRenderState.java`. Rust: `crates/voxel-render/src/skull.rs`, `crates/voxel/src/block_entities.rs`; `BlockEntityData` ile profil.

## Oracle
Gerçek renderer kaydı `entity/chest-pose` kalıbıyla (poz, ışık, doku, dörtgenler). Java kaynağı önce satır satır okunur.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`.

## Kapsam dışı
Başka blok entity aileleri.

## Belgeler
`docs/client-chest-parity-conformance.md`'ye bölüm, `docs/index.md` gerekirse.

## Devir notu
Yapılan: yerleştirilmiş kafa/kafatası blokları (yedi tür, ayakta ve duvarda, güçlü ejderha/piglin animasyonu) gerçek `SkullBlockRenderer`'a bit bit eşit. Kayıt: [client-chest-parity-conformance.md](../client-chest-parity-conformance.md).
Sapma: oyuncu kafası profili çözülmez (hep varsayılan görünüm; görünüm indirmesi M9'a bağlı).
Açık: güçlü animasyon canlıda denenmedi.
