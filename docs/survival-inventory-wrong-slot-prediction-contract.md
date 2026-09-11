# P13 — Survival inventory wrong changed-slot prediction sözleşmesi

Bu dilim, client'ın doğru menu state-id'siyle gönderdiği inventory click
packet'inde değişen slot için yanlış post-click hash, carried için doğru
post-click hash taşıdığı durumda vanilla server'ın yalnızca slot düzeltmesi
göndermesini sınar.

## Java referansı

`MultiPlayerGameMode.handleContainerInput`, click'i client menüsünde
uyguladıktan sonra önceki ve sonraki slotları karşılaştırır; değişen slot için
sonraki değerin `HashedStack` hash'ini, carried için de click sonrası değerin
hash'ini packet'e koyar. Bu probe slot hash'ini bilerek click öncesi stone stack'i
olarak gönderir.

`ServerGamePacketListenerImpl.handleContainerClick`, eşleşen state-id ile
gelen changed-slot ve carried hash'lerini remote beklentiye kaydeder, click'i
uygular ve `broadcastChanges` çağırır. Slotun remote beklentisi stone stack'i
olarak kaldığı için gerçek click sonrası empty slot ile eşleşmez; carried
beklentisi gerçek stone stack'iyle eşleştiği için cursor düzeltmesi gerekmez.
`AbstractContainerMenu` bu durumda slot için state-id artıran
`ClientboundContainerSetSlotPacket` gönderir.

`HashedStack` non-empty değer için optional presence byte'ı, item holder'ı,
count'u ve `HashedPatchMap` component patch'ini taşır. Component'siz stone
stack'inde added component map'i ve removed component set'i boştur.

## Probe girdisi ve akışı

- Oyun modu: `SURVIVAL`
- Container: id `0`, player inventory menu
- Hazırlık: hotbar index `0` / menu slotu `36` içine `minecraft:stone 8`
- Stone item registry ID: `1`
- Menu state-id: `2` (initial content state `1` sonrasındaki hazırlık state'i)
- Click: slot `36`, button `0`, `PICKUP` input id `0`
- Changed-slot prediction: slot `36` için click öncesi item id `1`, count `8`,
  boş component map/set içeren non-empty hash
- Carried prediction: click sonrası item id `1`, count `8`, boş component
  map/set içeren non-empty hash

Probe, item hazırlığı sonrası tek `container_click` paketi yollar; ardından
slot, cursor ve full content güncellemelerini toplar.

Beklenen invariant'lar şunlardır:

- Başlangıç content'i container `0`, state `1`, 46 slot ve carried için boş
  değer taşır.
- Console item hazırlığı menu slotu `36` için state `2`, count `8` ve item ID
  `1` slot gözlemi üretir.
- Server click'i mismatch saymaz ve pickup davranışını uygular.
- Yanlış changed-slot hash'i nedeniyle slot `36` için state `3` ile empty
  `container_set_slot` gönderilir.
- Carried hash'i doğru olduğu için `set_cursor_item` gönderilmez.
- Item hazırlığı dışındaki `container_set_content` gözlemi oluşmaz; full resync
  yolu çalışmaz.
- Rust probe bir `container_click` paketi yollar ve gönderilen state-id `2`
  olarak gözlenir.

## Sınır

Bu dilim component'siz stone stack için yanlış changed-slot hash'inin tek slot
düzeltmesine yol açmasını kapsar. Yanlış carried hash'i, iki hash'in birlikte
yanlış olması, component hash'leri, kısmi changed-slot map'i, diğer click
input'ları, stale state varyantları ve özel container'lar sonraki P13
dilimleridir.
