# T77b — Kırılan eşya parçacıkları

**Amaç:** `BreakingItemParticle` (`item`, `item_slime`, `item_cobweb`, `item_snowball`,
`sulfur_cube_goo`), `ItemParticleOption` paketi (eşya yığını), eşya modelinin parçacık malzemesi
(`ItemStackRenderState.pickParticleMaterial`, katman sayısıyla rastgele) ve eşya atlası.
**Bağımlılık:** T77. **Boy:** M.

## Oku
Java: `client/particle/BreakingItemParticle.java`, `core/particles/ItemParticleOption.java`,
`client/renderer/item/ItemStackRenderState.java`. Rust: `crates/voxel-world/src/particles/terrain.rs`
(blok kırıntısı aynı dörtte birlik pencereyi kullanır), `crates/voxel/src/particles.rs`.

## Oracle
`particle-ticks` satırları; malzeme seçimi için sabit katmanlı model.

## Kabul
`CHECK OK`.

## Belgeler
`docs/records/client-item-pickup-parity-conformance.md`.

## Devir notu
