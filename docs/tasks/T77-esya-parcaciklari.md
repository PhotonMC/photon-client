# T77 — Eşya parçacıkları ve eşya toplama animasyonu

**Amaç:** `BreakingItemParticle` (kartopu, balçık, örümcek ağı, eşya kırılması), `ItemParticleOption` paketi, `ItemPickupParticle` ve grubu, `ClientboundTakeItemEntityPacket`.
**Bağımlılık:** T70, T25. **Boy:** M.

## Oku
Java: `client/particle/{BreakingItemParticle,ItemPickupParticle,ItemPickupParticleGroup}.java`, `core/particles/ItemParticleOption.java`, `network/protocol/game/ClientboundTakeItemEntityPacket.java`. Rust: T70 ve `crates/voxel-world/src/entity.rs` (yerdeki eşya).

## Oracle
`particle_state`; toplama animasyonunun konumları (yarım tik aralığı, `ItemPickupParticle.extractRenderState`). Önce Java kaynağı satır satır okunur, oracle senaryosu yazılır, sonra Rust ([davranış testi belgesi](../behavior-testing.md)).

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`. Canlı gözlem yalnız görsel bir sonuç varsa, `tools/voxel.py shot` ile bir kez.

## Kapsam dışı
Başka aileler (kendi görevlerinde); performans optimizasyonu.

## Belgeler
İlgili contract/parity kaydı (`docs/` altında yeni dosya), `docs/index.md`, `docs/remaining-work.md` §8 işareti.

## Devir notu
Yapılan: `TakeItemEntity` (paket 124, ses, yığın küçültme, kaldırma), `particles/pickup.rs`
(`ItemPickupParticle`, motorda `ITEM_PICKUP` grubu), eşya ve kürelerin hedefe uçarak çizimi.
Oracle: `movement/item-pickup` üç satır bit eşit; paket okuyucu birim testli.
Sapma: uçan varlığın gölgesi yok, ok/trident uçarken çizilmiyor, ışık çizim anında okunuyor.
Açık: `BreakingItemParticle` ve `ItemParticleOption` → T77b.
