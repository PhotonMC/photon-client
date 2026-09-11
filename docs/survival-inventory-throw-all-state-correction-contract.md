# P13 — Survival inventory throw-all state-id correction sözleşmesi

Bu dilim, survival oyuncunun player inventory slotundaki stack'in tamamını
düşürmesini ve client'ın eski container state-id göndermesi durumunda server'ın
authoritative full content düzeltmesini sınar. Probe, normal istemcinin
changed-slot hash tahminini taklit etmez; server'ın throw-all ve state-id
correction dallarını boş prediction alanlarıyla görünür kılar.

## Java referansı

`GameProtocols.java` serverbound dispatch tablosunda
`ServerboundContainerClickPacket` 18 numaralı play paketidir. Packet alanları
container id, state id, slot short'u, button byte'ı, `ContainerInput`, değişen
slotların kısa indeksli map'i ve `HashedStack` carried değeridir.
`ContainerInput.THROW` kimliği `4`'tür; boş `HashedStack` optional codec'te tek
boolean `false` olarak taşınır.

`AbstractContainerMenu` throw dalında carried boşken button `1` için miktarı
slotun mevcut stack count'u olarak seçer, bu stack'i `player.drop` yoluna verir
ve aynı item slotta kaldığı sürece döngüyü kontrol eder. Count `8` stack bu
senaryoda tek drop ile tamamen boşalır. Button `0` ile tek item düşürme ayrı
bir dilimde doğrulanmıştır.

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
- Button: `1` (stack'in tamamı)
- Container input: `THROW`, id `4`
- Gönderilen state-id: `0` (hazırlık güncellemesinin state `2` değerinden eski)
- Gönderilen changed-slot map'i: boş
- Gönderilen carried hash'i: empty
- Item replacement komutu click'ten önce başarıyla tamamlanır; Rust probe dünya
  hazır olduktan sonra 30 saniye bekleyip tek `container_click` paketi yollar.
  Ardından 2.5 saniyelik drain ile slot, content ve add-entity paketlerini
  toplar.

Beklenen invariant'lar şunlardır:

- Başlangıç content'i container `0`, state `1`, 46 slot ve slot 36 ile carried
  için boş değer taşır.
- Console hazırlığı slot 36 için state `2`, count `8`, item registry `1`
  (stone), boş component patch'i taşır.
- Rust bir `container_click` paketi yollar; packet state-id'si `0`, slot `36`,
  button `1` ve input id `4`'tür.
- Server throw-all işlemini uygular; tam düzeltme content'i state `3` ile slot
  36'da boş, carried'da boş değer taşır.
- Düşen stack için player konumuna yakın bir `add_entity` gözlemi oluşur.
- Bu senaryoda block ACK'i veya effect gözlemi beklenmez.

## Sınır

Bu dilim server state-id uyuşmazlığı, survival throw-all ve full slot/cursor
resync'ini kapsar. Doğru state-id ile client'ın hashed changed-slot/cursor
tahmini, drag, double-click, stack sınırı, creative menu ve özel container'lar
sonraki P13 dilimleridir.
