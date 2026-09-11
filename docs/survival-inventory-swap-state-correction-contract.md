# P13 — Survival inventory swap state-id correction sözleşmesi

Bu dilim, survival oyuncunun hotbar slotundaki stack'i player inventory menu
slotuna `SWAP` tıklamasıyla taşımasını ve client'ın eski container state-id
göndermesi durumunda server'ın authoritative full content düzeltmesini sınar.
Probe, normal istemcinin changed-slot hash tahminini taklit etmez; server'ın
swap ve state-id correction dallarını boş prediction alanlarıyla görünür kılar.

## Java referansı

`GameProtocols.java` serverbound dispatch tablosunda
`ServerboundContainerClickPacket` 18 numaralı play paketidir. Packet alanları
container id, state id, slot short'u, button byte'ı, `ContainerInput`, değişen
slotların kısa indeksli map'i ve `HashedStack` carried değeridir.
`ContainerInput.SWAP` kimliği `2`'dir; boş `HashedStack` optional codec'te tek
boolean `false` olarak taşınır.

`AbstractContainerMenu` içindeki `SWAP` dalı button `0..8` değerini oyuncunun
hotbar inventory index'i olarak kullanır. Target menu slotu boş ve source
stack dolu olduğunda source hotbar slotu temizlenir, stack target slota yazılır.
Player inventory menu'sünde hotbar index `0`, menu slotu `36`'ya karşılık gelir;
bu probe target olarak ilk main inventory slotu olan menu slotu `9`'u seçer.

`ServerGamePacketListenerImpl.handleContainerClick` packet state-id'sini menu
state-id'siyle karşılaştırır, geçerli click'i uygular, sonra state-id farklıysa
`broadcastFullState` çağırır. `ServerPlayer` full content yayınında menu
state-id'sini artırır; `ClientboundContainerSetContentPacket` container id,
state id, 46 optional slot ve carried item'i taşır.

## Probe girdisi ve akışı

- Oyun modu: `SURVIVAL`
- Sunucu hazırlığı, oyuncu oyuna girdikten sonra:
  `item replace entity <player> hotbar.0 with minecraft:stone 8`
- Container: id `0`, player inventory menu
- Target slot: `9` (main inventory slot 0)
- Button: `0` (hotbar inventory index 0)
- Container input: `SWAP`, id `2`
- Gönderilen state-id: `0` (hazırlık güncellemesinin state `2` değerinden eski)
- Gönderilen changed-slot map'i: boş
- Gönderilen carried hash: empty
- Item replacement komutu click'ten önce başarıyla tamamlanır; Rust probe dünya
  hazır olduktan sonra 12 saniye bekleyip tek `container_click` paketi yollar.
  Ardından 2.5 saniyelik drain ile düzeltme paketlerini toplar.

Beklenen invariant'lar şunlardır:

- Başlangıç content'i container `0`, state `1`, 46 slot ve slot 9, slot 36 ile
  carried için boş değer taşır.
- Console hazırlığı slot 36 için state `2`, count `8`, item registry `1`
  (stone), boş component patch'i taşır.
- Rust bir `container_click` paketi yollar; packet state-id'si `0`, slot `9`,
  button `0` ve input id `2`'dir.
- Server swap işlemini uygular; tam düzeltme content'i state `3` ile menu slot
  9'da count `8`, item `1`, slot 36'da boş ve carried'da boş değer taşır.
- Bu senaryoda block ACK'i, effect veya projectile gözlemi beklenmez.

## Sınır

Bu dilim server state-id uyuşmazlığı, survival player-inventory swap ve full
slot/cursor resync'ini kapsar. Doğru state-id ile normal client prediction/hash
map'i, pickup button varyantları, throw, drag, double-click, stack sınırı,
creative menu ve özel container'lar sonraki P13 dilimleridir.
