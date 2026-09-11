# P13 — Survival inventory double-click stack limit state-id correction sözleşmesi

Bu dilim, vanilla `PICKUP_ALL` taramasının carried item'ın maksimum stack
boyutunda durmasını ve client'ın eski container state-id göndermesi durumunda
server'ın authoritative full content düzeltmesini sınar. Probe, ikinci stack'in
tamamının değil yalnızca cursor kapasitesi kadarının alınmasını görünür kılar.

## Java referansı

`GameProtocols.java` serverbound dispatch tablosunda
`ServerboundContainerClickPacket` 18 numaralı play paketidir. Packet alanları
container id, state id, slot short'u, button byte'ı, `ContainerInput`, değişen
slotların kısa indeksli map'i ve `HashedStack` carried değeridir.

`ContainerInput.PICKUP` kimliği `0`, `ContainerInput.PICKUP_ALL` kimliği `6`'dır.
`AbstractContainerMenu` `PICKUP_ALL` dalında carried count'u
`getMaxStackSize()` ile karşılaştırır. `target.safeTake`, carried'da kalan
kapasiteyi üst sınır alır; böylece count `60` taşıyan cursor, aynı item'dan
count `8` olan source slotun yalnızca `4` item'ını alır ve maksimum count `64`'
te kalır.

`ServerGamePacketListenerImpl.handleContainerClick` packet state-id'sini menu
state-id'siyle karşılaştırır, click'i uygular ve state-id farklıysa
`broadcastFullState` çağırır. `ServerPlayer` full content yayınında menu
state-id'sini artırır; `ClientboundContainerSetContentPacket` container id,
state id, 46 optional slot ve carried item'i taşır.

## Probe girdisi ve akışı

- Oyun modu: `SURVIVAL`
- Container: id `0`, player inventory menu
- Hazırlık: hotbar index `0` / menu slotu `36` içine stone count `60`
- Hazırlık: main inventory index `0` / menu slotu `9` içine stone count `8`
- Stone item registry ID: `1`
- Tüm client click packet'leri: state-id `0`, boş changed-slot map'i ve empty
  carried hash'i
- Slot dizisi:
  1. `(slot 36, button 0, input 0)` — 60'lı stack'i cursor'a pickup
  2. `(slot 10, button 0, input 6)` — boş hedef üzerinden `PICKUP_ALL`

Slot `10`, `PICKUP_ALL` için boş hedef koşulunu sağlar; button `0` ile ileri
tarama slot `9`'daki stack'e ulaşır. Probe, dünya hazır olduktan ve iki item
hazırlığı için bekledikten sonra bu iki paketi yollar; ardından full content
düzeltmelerini toplar.

Beklenen invariant'lar şunlardır:

- Başlangıç content'i container `0`, state `1`, 46 slot ve carried için boş
  değer taşır.
- Console item hazırlıkları state `2`'de slot `36` için count `60`, state `3`'te
  slot `9` için count `8` ve item ID `1` gözlemleri üretir.
- İlk eski state-id'li click pickup sonrası state `4` full content'inde slot
  `36` boştur, slot `9` count `8` taşır ve carried count `60` olur.
- İkinci eski state-id'li click sonrası state `5` full content'inde slot `9`
  count `4`, slot `36` boş ve carried count `64` olur.
- Rust probe iki `container_click` paketi gönderir ve gönderilen state-id `0`
  olarak gözlenir.

## Sınır

Bu dilim ileri `PICKUP_ALL` taramasında carried stack kapasitesinin dolmasını
kapsar. Button `1` ile geriye tarama, farklı item/component eşleşmeleri, doğru
state-id ile client prediction/hash map'i, yanlış state'in diğer varyantları,
creative menu ve özel container'lar sonraki P13 dilimleridir.
