# T78c — `BlockEvent` işleyicileri

**Amaç:** `ClientLevel.blockEvent` → `BlockState.triggerEvent`: nota bloğu (ses ve nota
parçacığı), piston (hareket eden blok başlangıcı), çan (sallanma), shulker kutusu, sandığın
dışındaki kaplar (ender sandığı, varil yok), kapı; sandık kapağı zaten var.
**Bağımlılık:** T78. **Boy:** M.

## Oku
Java: `network/protocol/game/ClientboundBlockEventPacket.java`,
`world/level/block/{NoteBlock,piston/PistonBaseBlock,BellBlock}.java`,
`world/level/block/entity/{BellBlockEntity,ShulkerBoxBlockEntity,EnderChestBlockEntity}.java`
(`triggerEvent`). Rust: `crates/voxel-net/src/connection/play/incoming/world.rs` (`on_block_event`).

## Oracle
`movement/particle-engine`'e `block_event` olayı (nota sesi ve parçacığı); çan/shulker durumu
için blok entity oracle'ı.

## Kabul
`CHECK OK`.

## Belgeler
Yeni kayıt.

## Devir notu
