# P13 — Survival inventory quick-move state-id correction sözleşmesi

Bu dilim, survival oyuncunun hotbar slotundaki stack'e shift-click karşılığı
olan `QUICK_MOVE` tıklamasını göndermesini ve eski container state-id ile gelen
client tahmini karşısında server'ın taşıma sonucunu full content düzeltmesiyle
bildirmesini sınar. Probe, normal istemcinin changed-slot tahminini taklit
etmez; boş prediction alanlarıyla server'ın authoritative quick-move ve
correction yolunu görünür kılar.

## Java referansı

`GameProtocols.java` serverbound dispatch tablosunda
`ServerboundContainerClickPacket` 18 numaralı play paketidir. Packet alanları
container id, state id, slot short'u, button byte'ı, `ContainerInput`, değişen
slotların kısa indeksli map'i ve `HashedStack` carried değeridir.
`ContainerInput.QUICK_MOVE` kimliği `1`'dir; boş `HashedStack` optional codec'te
tek boolean `false` olarak taşınır.

`ServerGamePacketListenerImpl.handleContainerClick` aynı container id ve
geçerli slot koşullarında state-id uyuşmazlığı olsa bile
`AbstractContainerMenu.clicked` çağrısını çalıştırır. `InventoryMenu` slot 36
hotbar stack'i için `quickMoveStack` çağrısında hedef aralığı menu slotları
`9..35` olarak seçer; ilk boş main inventory slotu menu slotu `9`'dur. Stack
slot 36'dan slot 9'a taşındıktan sonra eski state-id nedeniyle
`broadcastFullState` çalışır ve `ServerPlayer` artmış state-id ile 46 slotu ve
boş carried değerini `ClientboundContainerSetContentPacket` olarak yollar.

## Probe girdisi ve akışı

- Oyun modu: `SURVIVAL`
- Sunucu hazırlığı, oyuncu oyuna girdikten sonra:
  `item replace entity <player> hotbar.0 with minecraft:stone 8`
- Container: id `0`, player inventory menu
- Tıklanan slot: `36` (hotbar slot 0)
- Button: `0`
- Container input: `QUICK_MOVE`, id `1`
- Gönderilen state-id: `0` (hazırlık güncellemesinin state `2` değerinden eski)
- Gönderilen changed-slot map'i: boş
- Gönderilen carried hash: empty
- Rust probe, login/chunk/player-loaded akışından sonra 12 saniye bekler ve
  tek `container_click` paketi yollar; ardından 2.5 saniyelik drain ile
  düzeltme paketlerini toplar.

Beklenen invariant'lar şunlardır:

- Başlangıç content'i container `0`, state `1`, 46 slot, menu slot 9 ve slot 36
  için boş değerler taşır.
- Console hazırlığı slot 36 için state `2`, count `8`, item registry `1`
  (stone), boş component patch'i taşır.
- Rust bir `container_click` paketi yollar, input id `1` ve packet state-id'si
  `0`'dır.
- Server quick-move işlemini uygular; tam düzeltme content'i state `3` ile
  menu slot 9'da count `8`, item `1`, slot 36'da boş ve carried'da boş değer
  taşır.
- Bu senaryoda block ACK'i, effect veya projectile gözlemi beklenmez.

## Sınır

Bu dilim server state-id uyuşmazlığı, survival quick-move ve full inventory
resync'ini kapsar. Doğru state-id ile normal client prediction/hash map'i,
primary/secondary pickup, swap, throw, drag, double-click, stack sınırı,
creative menu ve özel container'lar sonraki P13 dilimleridir.
