# P13 — Survival inventory wrong slot ve carried prediction sözleşmesi

Bu dilim, client'ın doğru menu state-id'siyle gönderdiği inventory click
packet'inde hem değişen slot hem carried için yanlış post-click hash taşıdığı
durumda vanilla server'ın slot ve cursor düzeltmelerini bağımsız yayınlamasını
sınar.

## Java referansı

`MultiPlayerGameMode.handleContainerInput`, click'i client menüsünde
uyguladıktan sonra değişen slotların ve carried değerinin sonraki durumundan
`HashedStack` üretir. Bu probe slot hash'ini click öncesi stone stack'i, carried
hash'ini ise click sonrası empty değer olarak gönderir; iki değer de gerçek
click sonrası durumu yanlış gösterir.

`ServerGamePacketListenerImpl.handleContainerClick`, eşleşen state-id ile
gelen changed-slot ve carried hash'lerini remote beklentiye kaydeder, click'i
uygular ve `broadcastChanges` çağırır. Slotun remote beklentisi stone stack'i
olarak kaldığı için gerçek empty slotla eşleşmez; carried remote beklentisi
empty kaldığı için gerçek stone stack'le eşleşmez. `AbstractContainerMenu`
slot için state-id artıran `ClientboundContainerSetSlotPacket`, cursor için
state taşımayan `ClientboundSetCursorItemPacket` gönderir.

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
- Changed-slot prediction: click öncesi item id `1`, count `8`, boş component
  map/set içeren non-empty hash
- Carried prediction: bilerek empty hash

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
- Yanlış carried hash'i nedeniyle count `8`, item ID `1` ve boş component
  patch'i taşıyan `set_cursor_item` paketi gönderilir.
- Item hazırlığı dışındaki `container_set_content` gözlemi oluşmaz; full resync
  yolu çalışmaz.
- Rust probe bir `container_click` paketi yollar ve gönderilen state-id `2`
  olarak gözlenir.

## Sınır

Bu dilim component'siz stone stack için iki yanlış hash'in iki bağımsız
düzeltmeye yol açmasını kapsar. Component hash'leri, kısmi changed-slot map'i,
diğer click input'ları, stale state varyantları ve özel container'lar sonraki
P13 dilimleridir.
