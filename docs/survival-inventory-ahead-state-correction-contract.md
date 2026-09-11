# P13 — Survival inventory ahead state-id correction sözleşmesi

Bu dilim, player inventory click packet'i menu state-id'sinden büyük bir
state-id taşıdığında vanilla server'ın click'i yine uygulayıp authoritative
full content düzeltmesi göndermesini sınar. Önceki P13 probe'ları state `0` ile
menu state'inden eski packet'leri kullandı; bu probe mismatch'in diğer yönünü
izole eder.

## Java referansı

`GameProtocols.java` serverbound dispatch tablosunda
`ServerboundContainerClickPacket` 18 numaralı play paketidir. Packet alanları
container id, state id, slot short'u, button byte'ı, `ContainerInput`, değişen
slotların kısa indeksli map'i ve `HashedStack` carried değeridir.

`ServerGamePacketListenerImpl.handleContainerClick`, packet state-id'sini menu
state-id'siyle eşitlik üzerinden karşılaştırır. Eşit değilse remote update'leri
geçici olarak bastırır, `containerMenu.clicked` ile click davranışını uygular,
packet'in değişen slot/cursor tahminini kaydeder ve `broadcastFullState` ile
authoritative içeriği yollar. Bu dal state-id'nin menu değerinden küçük veya
büyük olmasını ayırmaz.

`ContainerInput.PICKUP` kimliği `0`'dır. Carried boşken button `0` ile menu slotu
`36`'daki stack alınır; slot boşalır ve stack cursor'a taşınır. `ServerPlayer`
full content yayınında menu state-id'sini artırır; `ClientboundContainerSetContentPacket`
container id, state id, 46 optional slot ve carried item'i taşır.

## Probe girdisi ve akışı

- Oyun modu: `SURVIVAL`
- Container: id `0`, player inventory menu
- Hazırlık: hotbar index `0` / menu slotu `36` içine `minecraft:stone 8`
- Stone item registry ID: `1`
- Gönderilen state-id: `99` (hazırlık sonrası menu state `2` değerinden büyük)
- Slot: `36`
- Button: `0`
- Container input: `PICKUP`, id `0`
- Gönderilen changed-slot map'i: boş
- Gönderilen carried hash'i: empty

Probe, dünya hazır olduktan ve item hazırlığı için bekledikten sonra tek
`container_click` paketi yollar; ardından full content düzeltmesini toplar.

Beklenen invariant'lar şunlardır:

- Başlangıç content'i container `0`, state `1`, 46 slot ve carried için boş
  değer taşır.
- Console item hazırlığı menu slotu `36` için state `2`, count `8` ve item ID
  `1` slot gözlemi üretir.
- State-id `99`, menu state `2` ile eşit olmadığı için mismatch branch'i seçilir;
  server pickup davranışını yine uygular.
- Authoritative correction state `3` ile slot `36` boş, carried count `8` ve
  item ID `1` taşır.
- Rust probe bir `container_click` paketi gönderir ve gönderilen state-id `99`
  olarak gözlenir.

## Sınır

Bu dilim state-id'nin menu değerinden büyük olduğu tekli survival pickup
durumunu kapsar. Non-empty changed-slot map'i, non-empty carried hash'i, aynı
pakette diğer input/slot varyantları, doğru state-id ile client prediction ve
özel container'lar sonraki P13 dilimleridir.
