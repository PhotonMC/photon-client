# P13 — Survival inventory throw state-id correction sözleşmesi

Bu dilim, survival oyuncunun player inventory slotundaki stack'ten tek item
düşürmesini ve client'ın eski container state-id göndermesi durumunda server'ın
authoritative full content düzeltmesini sınar. Probe, normal istemcinin
changed-slot hash tahminini taklit etmez; server'ın throw ve state-id correction
dallarını boş prediction alanlarıyla görünür kılar.

## Java referansı

`GameProtocols.java` serverbound dispatch tablosunda
`ServerboundContainerClickPacket` 18 numaralı play paketidir. Packet alanları
container id, state id, slot short'u, button byte'ı, `ContainerInput`, değişen
slotların kısa indeksli map'i ve `HashedStack` carried değeridir.
`ContainerInput.THROW` kimliği `4`'tür; boş `HashedStack` optional codec'te tek
boolean `false` olarak taşınır.

`AbstractContainerMenu` throw dalında carried boşken button `0` için miktarı
`1` seçer, slot üzerinden bir item alır ve `player.drop` çağrısı ile dünyaya
çıkarır. Stack'in geri kalanı slotta kalır. Button `1` ile tüm stack döngüsü
ayrı bir davranıştır ve bu dilime dahil değildir.

`ServerGamePacketListenerImpl.handleContainerClick` packet state-id'sini menu
state-id'siyle karşılaştırır, geçerli click'i ve drop yan etkisini uygular,
sonra state-id farklıysa `broadcastFullState` çağırır. `ServerPlayer` full
content yayınında menu state-id'sini artırır; `ClientboundContainerSetContentPacket`
container id, state id, 46 optional slot ve carried item'i taşır. Düşen item için
ayrıca clientbound `add_entity` akışı gözlenir; bu probe item metadata paketini
decode etmez.

## Probe girdisi ve akışı

- Oyun modu: `SURVIVAL`
- Sunucu hazırlığı, oyuncu oyuna girdikten sonra:
  `item replace entity <player> hotbar.0 with minecraft:stone 8`
- Container: id `0`, player inventory menu
- Tıklanan slot: `36` (hotbar slot 0)
- Button: `0` (tek item)
- Container input: `THROW`, id `4`
- Gönderilen state-id: `0` (hazırlık güncellemesinin state `2` değerinden eski)
- Gönderilen changed-slot map'i: boş
- Gönderilen carried hash: empty
- Item replacement komutu click'ten önce başarıyla tamamlanır; Rust probe dünya
  hazır olduktan sonra 20 saniye bekleyip tek `container_click` paketi yollar.
  Ardından 2.5 saniyelik drain ile slot, content ve add-entity paketlerini
  toplar.

Beklenen invariant'lar şunlardır:

- Başlangıç content'i container `0`, state `1`, 46 slot ve slot 36 ile carried
  için boş değer taşır.
- Console hazırlığı slot 36 için state `2`, count `8`, item registry `1`
  (stone), boş component patch'i taşır.
- Rust bir `container_click` paketi yollar; packet state-id'si `0`, slot `36`,
  button `0` ve input id `4`'tür.
- Server tek item drop işlemini uygular; tam düzeltme content'i state `3` ile
  slot 36'da count `7`, item `1`, carried'da boş değer taşır.
- Düşen item için player konumuna yakın en az bir `add_entity` gözlemi oluşur.
- Bu senaryoda block ACK'i veya effect gözlemi beklenmez.

## Sınır

Bu dilim server state-id uyuşmazlığı, survival tek item throw ve full slot/cursor
resync'ini kapsar. Throw-all, doğru state-id ile normal client prediction/hash
map'i, diğer pickup button'ları, drag, double-click, stack sınırı, creative menu
ve özel container'lar sonraki P13 dilimleridir.
