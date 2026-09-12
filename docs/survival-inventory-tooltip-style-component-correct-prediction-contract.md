# P13 — Survival inventory tooltip style component correct prediction sözleşmesi

Bu dilim, `TOOLTIP_STYLE` component'i taşıyan bir inventory item'ı pickup
ederken carried `HashedStack` içindeki component hash'i vanilla ile eşleştiğinde
server'ın ek slot veya cursor düzeltmesi göndermemesini sınar.

## Java referansı

`DataComponents.TOOLTIP_STYLE`, persistent `Identifier.CODEC` ve
`Identifier.STREAM_CODEC` kullanan bir component'tir. Pinned component
registry'de type ID'si `35`, stone item registry ID'si `1`'dir. `minecraft:default`
identifier değerinin Java CRC32C component hash'i `0xababf3db`'dir.

`ServerGamePacketListenerImpl.handleContainerClick`, state-id eşleştiğinde
changed-slot ve carried hash'lerini remote beklentiye kaydeder ve
`broadcastChanges` çağırır. Pickup sonrası slot empty, carried ise tooltip style
component'li stone olduğundan prediction gerçek server durumuyla eşleştiğinde
ek `container_set_slot` veya `set_cursor_item` paketi oluşmaz.

## Probe girdisi ve akışı

- Oyun modu: `SURVIVAL`
- Container: id `0`, player inventory menu
- Hazırlık: hotbar index `0` / menu slotu `36` içine
  `minecraft:stone[tooltip_style="minecraft:default"]`
- Stone item registry ID: `1`
- `tooltip_style` component type ID: `35`
- Component değeri: `minecraft:default`
- Java oracle hash'i: `0xababf3db`
- Menu state-id: `2` (initial content state `1` sonrasındaki hazırlık state'i)
- Click: slot `36`, button `0`, `PICKUP` input id `0`
- Changed-slot prediction: map içinde slot `36`, pickup sonrası boş hash
- Carried prediction: item ID `1`, count `1`, added component type `35` ve Java
  oracle hash'i, removed set'i boş

Probe, item hazırlığı sonrası tek `container_click` paketi yollar; ardından
slot, cursor ve full content güncellemelerini toplar.

Beklenen invariant'lar şunlardır:

- Başlangıç content'i container `0`, state `1`, 46 slot ve carried için boş
  değer taşır.
- Console item hazırlığı slot `36` için state `2`, item ID `1`, count `1` ve
  positive component ID `35` gözlemi üretir.
- Server click'i state-id mismatch saymaz ve pickup davranışını uygular.
- Changed-slot prediction gerçek empty slot ile eşleştiği için click sonrası
  slot düzeltmesi gönderilmez.
- Carried tooltip style prediction gerçek identifier hash'i ile eşleştiği için
  cursor düzeltmesi gönderilmez.
- Item hazırlığı dışındaki `container_set_content` gözlemi oluşmaz; full resync
  yolu çalışmaz.

## Sınır

Bu dilim `TOOLTIP_STYLE` component'li stone pickup akışını kapsar. Identifier
component'lerinin diğer değerleri, diğer component codec'leri, yanlış veya
kısmi component prediction, diğer click input'ları ve özel container'lar
sonraki P13 dilimleridir.
