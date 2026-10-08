# T78d — Kalan blok olayları

**Amaç:** `BlockEvent`'in nota bloğu ve sandık/shulker/çan dışındaki kolları: piston
(`PistonBaseBlock.triggerEvent`: istemcide hareket eden blok ve uzama/çekilme sesi), süslü saksının
sallanması (`DecoratedPotBlockEntity.wobble`), spawner (`SpawnerBlockEntity` gecikme sıfırlama),
end geçidi (soğuma ışını), potent sulfur patlaması.
**Bağımlılık:** T78c. **Boy:** L.

## Oku
Java: `world/level/block/piston/{PistonBaseBlock,PistonMovingBlockEntity}.java`,
`world/level/block/entity/{DecoratedPotBlockEntity,SpawnerBlockEntity,TheEndGatewayBlockEntity}.java`,
`world/level/block/PotentSulfurBlock.java`. Rust: `crates/voxel-world/src/world/sounds.rs`
(`apply_note_block_event`).

## Oracle
`movement/particle-engine` `block_event` olayları; piston için blok durumu ve hareket eden blok.

## Kabul
`CHECK OK`.

## Belgeler
`docs/records/client-note-block-parity-conformance.md`.

## Devir notu
