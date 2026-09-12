# P13 — Survival inventory break sound component correct prediction sözleşmesi

Bu dilim, `BREAK_SOUND` component'i taşıyan bir inventory item'ı pickup ederken
carried `HashedStack` içindeki component hash'i vanilla ile eşleştiğinde
server'ın ek slot veya cursor düzeltmesi göndermemesini sınar.

## Java referansı

`DataComponents.BREAK_SOUND`, `SoundEvent.CODEC` ile sound event holder'ını
persistent olarak ve `SoundEvent.STREAM_CODEC` ile registry/direct holder
kodlaması üzerinden taşır. Pinned runtime component registry'de type ID'si `81`,
stone item registry ID'si `1`'dir. `minecraft:block.stone.break` sound registry
ID'si `1596`'dır. Registry-aware Java CRC32C component hash'i `0x9662f221`'dir.

`ServerGamePacketListenerImpl.handleContainerClick`, state-id eşleştiğinde
changed-slot ve carried hash'lerini remote beklentiye kaydeder ve
`broadcastChanges` çağırır. Pickup sonrası slot empty, carried ise break sound
component'li stone olduğundan prediction gerçek server durumuyla eşleştiğinde
ek `container_set_slot` veya `set_cursor_item` paketi oluşmaz.

## Probe girdisi ve akışı

- Oyun modu: `SURVIVAL`
- Container: id `0`, player inventory menu
- Hazırlık: hotbar index `0` / menu slotu `36` içine
  `minecraft:stone[break_sound="minecraft:block.stone.break"]`
- Stone item registry ID: `1`
- `break_sound` component type ID: `81`
- Sound registry ID: `1596`
- Component değeri: `minecraft:block.stone.break`
- Java registry-aware oracle hash'i: `0x9662f221`
- Menu state-id: `2` (initial content state `1` sonrasındaki hazırlık state'i)
- Click: slot `36`, button `0`, `PICKUP` input id `0`
- Changed-slot prediction: map içinde slot `36`, pickup sonrası boş hash
- Carried prediction: item ID `1`, count `1`, added component type `81` ve Java
  oracle hash'i, removed set'i boş

Probe, item hazırlığı sonrası tek `container_click` paketi yollar; ardından slot,
cursor ve full content güncellemelerini toplar.

Beklenen invariant'lar şunlardır:

- Başlangıç content'i container `0`, state `1`, 46 slot ve carried için boş
  değer taşır.
- Console item hazırlığı slot `36` için state `2`, item ID `1`, count `1` ve
  positive component ID `81` gözlemi üretir.
- Server click'i state-id mismatch saymaz ve pickup davranışını uygular.
- Changed-slot prediction gerçek empty slot ile eşleştiği için click sonrası
  slot düzeltmesi gönderilmez.
- Carried break sound prediction gerçek registry-aware holder hash'i ile
  eşleştiği için cursor düzeltmesi gönderilmez.
- Item hazırlığı dışındaki `container_set_content` gözlemi oluşmaz; full resync
  yolu çalışmaz.

## Sınır

Bu dilim `BREAK_SOUND` component'li stone pickup akışını kapsar. Diğer sound
event'leri, direct holder varyantları, diğer component codec'leri, yanlış veya
kısmi component prediction, diğer click input'ları ve özel container'lar sonraki
P13 dilimleridir.
