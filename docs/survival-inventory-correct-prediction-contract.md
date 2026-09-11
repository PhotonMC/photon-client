# P13 — Survival inventory correct prediction sözleşmesi

Bu dilim, client'ın inventory click packet'inde vanilla'nın beklediği mevcut
slot/cursor hash'lerini ve doğru menu state-id'sini göndermesi durumunda server'ın
full resync yerine artımlı slot/cursor güncellemeleri yayınlamasını sınar.

## Java referansı

`GameProtocols.java` serverbound dispatch tablosunda
`ServerboundContainerClickPacket` 18 numaralı play paketidir. `HashedStack`
non-empty değer için optional presence byte'ı, item holder'ı, count'u ve
`HashedPatchMap` component patch'ini taşır. Empty hash yalnızca optional empty
değeridir. Stone stack'inin component patch'i boş olduğundan prediction içinde
added component map'i ve removed component set'i boş olur.

`ServerPlayer` component hash'lerini registry serialization context ile CRC32C
üzerinden üretir. Empty component patch'inde hash alanı olmadığı için bu probe
component hash değerine ihtiyaç duymaz. `RemoteSlot.receive` packet'teki hash'i
server'ın mevcut remote beklentisi olarak kaydeder.

`ServerGamePacketListenerImpl.handleContainerClick`, state-id packet ile menu
state-id'si eşitse `broadcastChanges` yolunu kullanır. Pickup click'i slotu
boşaltıp carried stack'i oluşturur; remote slot hash'i eski stack'i doğru
gösterdiği için yalnızca slot değişimi yayınlanır. `ServerPlayer` slot değişimi
için state-id'yi artıran `ClientboundContainerSetSlotPacket`, cursor değişimi
için state-id taşımayan `ClientboundSetCursorItemPacket` gönderir.

## Probe girdisi ve akışı

- Oyun modu: `SURVIVAL`
- Container: id `0`, player inventory menu
- Hazırlık: hotbar index `0` / menu slotu `36` içine `minecraft:stone 8`
- Stone item registry ID: `1`
- Menu state-id: `2` (initial content state `1` sonrasındaki hazırlık state'i)
- Click: slot `36`, button `0`, `PICKUP` input id `0`
- Changed-slot prediction: map içinde slot `36`, item id `1`, count `8`, boş
  component map/set
- Carried prediction: empty hash

Probe, item hazırlığı sonrası doğru state-id ve prediction ile tek
`container_click` paketi yollar; ardından slot/cursor güncellemelerini toplar.

Beklenen invariant'lar şunlardır:

- Başlangıç content'i container `0`, state `1`, 46 slot ve carried için boş
  değer taşır.
- Console item hazırlığı menu slotu `36` için state `2`, count `8` ve item ID
  `1` slot gözlemi üretir.
- Server click'i mismatch saymaz ve pickup davranışını uygular.
- Slot `36` için state `3` ile empty `container_set_slot` gönderilir.
- Cursor için ayrı `set_cursor_item` paketi count `8`, item ID `1` ve boş
  component patch'i taşır.
- Item hazırlığı dışındaki `container_set_content` gözlemi oluşmaz; full resync
  yerine artımlı yayın yapılır.
- Rust probe bir `container_click` paketi yollar ve gönderilen state-id `2`
  olarak gözlenir.

## Sınır

Bu dilim component'siz stone stack için doğru state-id, changed-slot hash'i,
empty carried hash'i ve incremental slot/cursor yayınını kapsar. Component
hash değerleri, yanlış veya kısmi prediction, diğer click input'ları, stale
state varyantları ve özel container'lar sonraki P13 dilimleridir.
