# P13 — Survival inventory note block sound component correct prediction sözleşmesi

Bu dilim, `NOTE_BLOCK_SOUND` component'i taşıyan bir inventory item'ı pickup
ederken carried `HashedStack` içindeki component hash'i vanilla ile eşleştiğinde
server'ın ek slot veya cursor düzeltmesi göndermemesini sınar.

## Java referansı

`DataComponents.NOTE_BLOCK_SOUND`, `Identifier.CODEC` ile resource identifier'ı
persistent olarak ve `Identifier.STREAM_CODEC` ile length-prefixed UTF-8 olarak
taşır. Pinned runtime component registry'de type ID'si `71`, stone item registry
ID'si `1`'dir. `minecraft:block.note_block.harp` için Java CRC32C component
hash'i `0xda545935`'tir.

`ServerGamePacketListenerImpl.handleContainerClick`, state-id eşleştiğinde
changed-slot ve carried hash'lerini remote beklentiye kaydeder ve
`broadcastChanges` çağırır. Pickup sonrası slot empty, carried ise note block
sound component'li stone olduğundan prediction gerçek server durumuyla
eşleştiğinde ek `container_set_slot` veya `set_cursor_item` paketi oluşmaz.

## Probe girdisi ve akışı

- Oyun modu: `SURVIVAL`
- Container: id `0`, player inventory menu
- Hazırlık: hotbar index `0` / menu slotu `36` içine
  `minecraft:stone[note_block_sound="minecraft:block.note_block.harp"]`
- Stone item registry ID: `1`
- `note_block_sound` component type ID: `71`
- Component değeri: `minecraft:block.note_block.harp`
- Java oracle hash'i: `0xda545935`
- Menu state-id: `2` (initial content state `1` sonrasındaki hazırlık state'i)
- Click: slot `36`, button `0`, `PICKUP` input id `0`
- Changed-slot prediction: map içinde slot `36`, pickup sonrası boş hash
- Carried prediction: item ID `1`, count `1`, added component type `71` ve Java
  oracle hash'i, removed set'i boş

Probe, item hazırlığı sonrası tek `container_click` paketi yollar; ardından slot,
cursor ve full content güncellemelerini toplar.

Beklenen invariant'lar şunlardır:

- Başlangıç content'i container `0`, state `1`, 46 slot ve carried için boş
  değer taşır.
- Console item hazırlığı slot `36` için state `2`, item ID `1`, count `1` ve
  positive component ID `71` gözlemi üretir.
- Server click'i state-id mismatch saymaz ve pickup davranışını uygular.
- Changed-slot prediction gerçek empty slot ile eşleştiği için click sonrası
  slot düzeltmesi gönderilmez.
- Carried note block sound prediction gerçek identifier hash'i ile eşleştiği
  için cursor düzeltmesi gönderilmez.
- Item hazırlığı dışındaki `container_set_content` gözlemi oluşmaz; full resync
  yolu çalışmaz.

## Sınır

Bu dilim `NOTE_BLOCK_SOUND` component'li stone pickup akışını kapsar. Diğer
identifier değerleri, diğer component codec'leri, yanlış veya kısmi component
prediction, diğer click input'ları ve özel container'lar sonraki P13 dilimleridir.
