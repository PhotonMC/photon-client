# P13 — Survival inventory pickup state-id correction sözleşmesi

Bu dilim, survival oyuncunun hotbar slotundaki stack'i pickup tıklamasıyla
almasını ve client'ın eski container state-id göndermesi durumunda server'ın
authoritative slot/cursor tam düzeltmesini sınar. Probe, normal istemcinin
öngörüsünü temsil etmek için değil, server state-id uyuşmazlığı ve correction
dalını kontrollü olarak görünür kılmak için boş `changedSlots` ve boş carried
hash gönderir.

## Java referansı

`GameProtocols.java` serverbound dispatch tablosunda
`ServerboundContainerClickPacket` 18 numaralı play paketidir. Packet alanları
sırasıyla container id, state id, slot short'u, button byte'ı, `ContainerInput`,
değişen slotların kısa indeksli map'i ve `HashedStack` carried değeridir.
`ContainerInput.PICKUP` kimliği `0`'dır; boş `HashedStack` optional codec'te tek
boolean `false` olarak taşınır.

`ServerGamePacketListenerImpl.handleContainerClick` aynı container id ve
geçerli slot koşullarında state-id'yi mevcut menu state-id ile karşılaştırır,
ardından `AbstractContainerMenu.clicked` çağrısını yine çalıştırır. Pickup
button `0`, carried boşken slot 36'daki stack'in tamamını carried'a alır ve
slotu boşaltır. Paket state-id'si farklıysa listener değişiklikleri uyguladıktan
sonra `broadcastFullState` çağırır; `ServerPlayer` bu tam yayında menu state-id
değerini artırarak tüm 46 slotu ve carried item'i
`ClientboundContainerSetContentPacket` ile yollar.

## Probe girdisi ve akışı

- Oyun modu: `SURVIVAL`
- Sunucu hazırlığı, oyuncu oyuna girdikten sonra:
  `item replace entity <player> hotbar.0 with minecraft:stone 8`
- Container: id `0`, player inventory menu
- Tıklanan slot: `36` (hotbar slot 0)
- Button: `0` (primary pickup)
- Container input: `PICKUP`, id `0`
- Gönderilen state-id: `0` (hazırlık güncellemesinin state `2` değerinden eski)
- Gönderilen changed-slot map'i: boş
- Gönderilen carried hash: empty
- Rust probe, login/chunk/player-loaded akışından sonra 12 saniye bekler ve
  tek `container_click` paketi yollar; ardından 2.5 saniyelik drain ile
  düzeltme paketlerini toplar.

Beklenen invariant'lar şunlardır:

- Başlangıç content'i container `0`, state `1`, 46 slot ve boş slot 36 taşır.
- Console hazırlığı slot 36 için state `2`, count `8`, item registry `1`
  (stone), boş component patch'i taşır.
- Rust bir `container_click` paketi yollar ve packet state-id'si `0`'dır.
- Server tıklamayı uygular; tam düzeltme content'i state `3` ile 46 slot taşır,
  slot 36 boştur ve carried item count `8`, registry `1`'dir.
- Bu senaryoda block ACK'i, effect veya projectile gözlemi beklenmez.

## Sınır

Bu dilim server state-id uyuşmazlığı, primary pickup ve full slot/cursor
resync'ini kapsar. Doğru state-id ile normal client prediction/hash map'i,
secondary pickup, quick-move, swap, throw, drag, double-click, stack sınırı,
creative menu ve özel container'lar sonraki P13 dilimleridir.
