# T70e — Blokların rastgele görünüm tiki (`ClientLevel.animateTick`)

**Amaç:** Her tik oyuncunun çevresinde seviye rastgelesiyle 667 çift blok seçilir, bloğun ve
sıvısının `animateTick`'i kendi rastgelesiyle çalışır: meşale alevi ve dumanı, kızıltaş meşalesi
tozu, lav kıvılcımı ve sesi, su zerreleri ve mırıltısı, sıvı altı damla yeri. Bu döngü hiç yoktu;
meşaleler alevsiz görünüyordu.
**Bağımlılık:** T70b, T75. **Boy:** M.

## Oku
Java: `client/multiplayer/ClientLevel.java` (`animateTick`, `doAnimateTick`, `trySpawnDripParticles`),
`world/level/block/{TorchBlock,WallTorchBlock,RedstoneTorchBlock,RedstoneWallTorchBlock}.java`,
`world/level/material/{LavaFluid,WaterFluid}.java`, `client/Minecraft.java` (`tick` sırası).

## Oracle
`movement/particle-engine`'e `animate_tick` olayı (tohumlu blok rastgelesiyle `doAnimateTick`).

## Kabul
`CHECK OK`.

## Kapsam dışı
Öteki blokların `animateTick`'i (ateş, kamp ateşi, mum, yaprak, fırın, portal…) → T70f; biyom
ortam parçacıkları; damla parçacığının kendisi (T73b).

## Belgeler
`docs/records/client-block-animate-parity-conformance.md`.

## Devir notu
Yapılan: `particles/animate.rs` (döngü, meşaleler, lav, su, damla yeri), oyun döngüsünde motor
tikinden önce oyuncunun bloğu çevresinde. Oracle: `animate-torches`, `animate-fluids` bit eşit.
Sapma: biyomun `AMBIENT_PARTICLES`'ı yok (o biyomlarda seviye rastgelesi kayar); işaretçi
parçacığı (yaratıcıda bariyer/ışık) yok; damla parçacığı motorda olmadığından doğmuyor.
Açık: öteki bloklar → T70f, damla → T73b.
