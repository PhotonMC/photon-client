# P13 — Survival inventory dye component correct prediction sözleşmesi

Bu dilim, `DYE` component'i taşıyan bir inventory item'ı pickup ederken carried
`HashedStack` içindeki component hash'i vanilla ile eşleştiğinde server'ın ek
slot veya cursor düzeltmesi göndermemesini sınar.

## Java referansı

`DataComponents.DYE`, `DyeColor.CODEC` ile enum adını persistent olarak ve
`DyeColor.STREAM_CODEC` ile color ID'sini VarInt olarak taşır. Pinned component
registry'de type ID'si `43`, stone item registry ID'si `1`'dir. `DyeColor.RED`
(`red`, color ID `14`) için Java CRC32C component hash'i `0x8c6452aa`'dir.

`ServerGamePacketListenerImpl.handleContainerClick`, state-id eşleştiğinde
changed-slot ve carried hash'lerini remote beklentiye kaydeder ve
`broadcastChanges` çağırır. Pickup sonrası slot empty, carried ise red dye
component'li stone olduğundan prediction gerçek server durumuyla eşleştiğinde
ek `container_set_slot` veya `set_cursor_item` paketi oluşmaz.

## Probe girdisi ve akışı

- Oyun modu: `SURVIVAL`
- Container: id `0`, player inventory menu
- Hazırlık: hotbar index `0` / menu slotu `36` içine
  `minecraft:stone[dye=red]`
- Stone item registry ID: `1`
- `dye` component type ID: `43`
- Component değeri: `DyeColor.RED` / `red` / ID `14`
- Java oracle hash'i: `0x8c6452aa`
- Menu state-id: `2` (initial content state `1` sonrasındaki hazırlık state'i)
- Click: slot `36`, button `0`, `PICKUP` input id `0`
- Changed-slot prediction: map içinde slot `36`, pickup sonrası boş hash
- Carried prediction: item ID `1`, count `1`, added component type `43` ve Java
  oracle hash'i, removed set'i boş

Probe, item hazırlığı sonrası tek `container_click` paketi yollar; ardından
slot, cursor ve full content güncellemelerini toplar.

Beklenen invariant'lar şunlardır:

- Başlangıç content'i container `0`, state `1`, 46 slot ve carried için boş
  değer taşır.
- Console item hazırlığı slot `36` için state `2`, item ID `1`, count `1` ve
  positive component ID `43` gözlemi üretir.
- Server click'i state-id mismatch saymaz ve pickup davranışını uygular.
- Changed-slot prediction gerçek empty slot ile eşleştiği için click sonrası
  slot düzeltmesi gönderilmez.
- Carried red dye prediction gerçek color ID hash'i ile eşleştiği için cursor
  düzeltmesi gönderilmez.
- Item hazırlığı dışındaki `container_set_content` gözlemi oluşmaz; full resync
  yolu çalışmaz.

## Sınır

Bu dilim `DYE=red` component'li stone pickup akışını kapsar. Diğer dye renkleri,
diğer component codec'leri, yanlış veya kısmi component prediction, diğer click
input'ları ve özel container'lar sonraki P13 dilimleridir.
