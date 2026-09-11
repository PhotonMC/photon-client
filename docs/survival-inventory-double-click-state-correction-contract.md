# P13 — Survival inventory double-click state-id correction sözleşmesi

Bu dilim, carried stack ile player inventory içindeki aynı item stack'lerini
vanilla `PICKUP_ALL` davranışıyla toplama ve client'ın eski container state-id
göndermesi durumunda server'ın authoritative full content düzeltmesini sınar.
Probe, boş hedef slot koşulunu ve iki ayrı stack'in cursor'da birleşmesini
kablo seviyesinde görünür kılar.

## Java referansı

`GameProtocols.java` serverbound dispatch tablosunda
`ServerboundContainerClickPacket` 18 numaralı play paketidir. Packet alanları
container id, state id, slot short'u, button byte'ı, `ContainerInput`, değişen
slotların kısa indeksli map'i ve `HashedStack` carried değeridir.

`ContainerInput.PICKUP` kimliği `0`, `ContainerInput.PICKUP_ALL` kimliği `6`'dır.
`AbstractContainerMenu` `PICKUP_ALL` dalını yalnızca carried item boş değilken
ve tıklanan slot boş veya pickup'a kapalıyken işler. Button `0` slotları
başlangıçtan sona doğru tarar; aynı item ve component'lere sahip stack'lerden
carried max stack sınırına kadar item alır.

`ServerGamePacketListenerImpl.handleContainerClick` packet state-id'sini menu
state-id'siyle karşılaştırır, click'i uygular ve state-id farklıysa
`broadcastFullState` çağırır. `ServerPlayer` full content yayınında menu
state-id'sini artırır; `ClientboundContainerSetContentPacket` container id,
state id, 46 optional slot ve carried item'i taşır.

## Probe girdisi ve akışı

- Oyun modu: `SURVIVAL`
- Container: id `0`, player inventory menu
- Hazırlık: hotbar index `0` / menu slotu `36` içine stone count `8`
- Hazırlık: main inventory index `0` / menu slotu `9` içine stone count `3`
- Stone item registry ID: `1`
- Tüm client click packet'leri: state-id `0`, boş changed-slot map'i ve empty
  carried hash'i
- Slot dizisi:
  1. `(slot 36, button 0, input 0)` — 8'li stack'i cursor'a pickup
  2. `(slot 10, button 0, input 6)` — boş hedef üzerinden `PICKUP_ALL`

Slot `10`, `PICKUP_ALL` için boş hedef koşulunu sağlar; ileri tarama slot `9`'
daki 3 taş stack'ini cursor'a ekler. Probe, dünya hazır olduktan ve iki item
hazırlığı için bekledikten sonra bu iki paketi yollar; ardından full content
düzeltmelerini toplar.

Beklenen invariant'lar şunlardır:

- Başlangıç content'i container `0`, state `1`, 46 slot ve carried için boş
  değer taşır.
- Console item hazırlıkları state `2`'de slot `36` için count `8`, state `3`'te
  slot `9` için count `3` ve item ID `1` gözlemleri üretir.
- İlk eski state-id'li click pickup sonrası state `4` full content'inde slot
  `36` boştur, slot `9` count `3` taşır ve carried count `8` olur.
- İkinci eski state-id'li click sonrası state `5` full content'inde slot `9` ve
  slot `36` boştur; carried count `11` ve item ID `1` olur.
- Rust probe iki `container_click` paketi gönderir ve gönderilen state-id `0`
  olarak gözlenir.

## Sınır

Bu dilim button `0` ile ileri taranan survival `PICKUP_ALL` ve iki stack'in
cursor'da birleşmesini kapsar. Button `1` ile geriye tarama, cursor max stack
sınırı, doğru state-id ile client prediction/hash map'i, yanlış state'in diğer
varyantları, creative menu ve özel container'lar sonraki P13 dilimleridir.
