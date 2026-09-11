# P13 — Survival inventory secondary pickup state-id correction sözleşmesi

Bu dilim, survival oyuncunun player inventory hotbar stack'inin yarısını
secondary pickup tıklamasıyla cursor'a almasını ve client'ın eski container
state-id göndermesi durumunda server'ın authoritative full content düzeltmesini
sınar. Probe, normal istemcinin changed-slot hash tahminini taklit etmez;
server'ın secondary pickup ve state-id correction dallarını boş prediction
alanlarıyla görünür kılar.

## Java referansı

`GameProtocols.java` serverbound dispatch tablosunda
`ServerboundContainerClickPacket` 18 numaralı play paketidir. Packet alanları
container id, state id, slot short'u, button byte'ı, `ContainerInput`, değişen
slotların kısa indeksli map'i ve `HashedStack` carried değeridir.
`ContainerInput.PICKUP` kimliği `0`'dır; secondary davranış packet input'ı
değiştirmeden button `1` ile seçilir. Boş `HashedStack` optional codec'te tek
boolean `false` olarak taşınır.

`AbstractContainerMenu` pickup dalında button `0` primary, button `1`
secondary olarak ayrılır. Carried boşken secondary pickup miktarı
`(clicked count + 1) / 2`'dir; count `8` stack için 4 item cursor'a alınır ve
slotta 4 item bırakılır. `ServerGamePacketListenerImpl.handleContainerClick`
packet state-id'sini menu state-id'siyle karşılaştırır, geçerli click'i uygular,
sonra state-id farklıysa `broadcastFullState` çağırır. `ServerPlayer` full
content yayınında menu state-id'sini artırır; `ClientboundContainerSetContentPacket`
container id, state id, 46 optional slot ve carried item'i taşır.

## Probe girdisi ve akışı

- Oyun modu: `SURVIVAL`
- Sunucu hazırlığı, oyuncu oyuna girdikten sonra:
  `item replace entity <player> hotbar.0 with minecraft:stone 8`
- Container: id `0`, player inventory menu
- Tıklanan slot: `36` (hotbar slot 0)
- Button: `1` (secondary pickup)
- Container input: `PICKUP`, id `0`
- Gönderilen state-id: `0` (hazırlık güncellemesinin state `2` değerinden eski)
- Gönderilen changed-slot map'i: boş
- Gönderilen carried hash: empty
- Item replacement komutu click'ten önce başarıyla tamamlanır; Rust probe dünya
  hazır olduktan sonra 20 saniye bekleyip tek `container_click` paketi yollar.
  Ardından 2.5 saniyelik drain ile düzeltme paketlerini toplar.

Beklenen invariant'lar şunlardır:

- Başlangıç content'i container `0`, state `1`, 46 slot ve slot 36 ile carried
  için boş değer taşır.
- Console hazırlığı slot 36 için state `2`, count `8`, item registry `1`
  (stone), boş component patch'i taşır.
- Rust bir `container_click` paketi yollar; packet state-id'si `0`, slot `36`,
  button `1` ve input id `0`'dır.
- Server secondary pickup işlemini uygular; tam düzeltme content'i state `3`
  ile slot 36'da count `4`, item `1`, carried'da count `4`, item `1` taşır.
- Bu senaryoda block ACK'i, effect veya projectile gözlemi beklenmez.

## Sınır

Bu dilim server state-id uyuşmazlığı, survival secondary pickup ve full
slot/cursor resync'ini kapsar. Doğru state-id ile normal client prediction/hash
map'i, primary pickup'ın diğer stack koşulları, throw, drag, double-click, stack
sınırı, creative menu ve özel container'lar sonraki P13 dilimleridir.
