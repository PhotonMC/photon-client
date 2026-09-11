# P13 — Survival inventory tool component correct prediction sözleşmesi

Bu dilim, tool component taşıyan bir inventory item'ı pickup ederken carried
`HashedStack` içindeki component hash'i vanilla ile eşleştiğinde server'ın ek
slot veya cursor düzeltmesi göndermemesini sınar.

## Java referansı

`Tool.CODEC`, `rules` listesini ve `default_mining_speed`, `damage_per_block`
ile `can_destroy_blocks_in_creative` alanlarını encode eder. `DataComponents.TOOL`
bu codec'i network-synchronized component olarak kaydeder. Pinned registry'de
tool component type ID'si `28`'dir. Stone item registry ID'si `1`'dir.

Java oracle'da boş `rules`, `default_mining_speed=1.0`, `damage_per_block=1`
ve `can_destroy_blocks_in_creative=true` değeri için hash `0x37de0062` elde
edildi.

`ServerGamePacketListenerImpl.handleContainerClick`, state-id eşleştiğinde
changed-slot ve carried hash'lerini remote beklentiye kaydeder ve
`broadcastChanges` çağırır. Pickup sonrası slot empty, carried ise tool
component'li stone olduğundan iki prediction gerçek server durumuyla
eşleştiğinde ek `container_set_slot` veya `set_cursor_item` paketi oluşmaz.

## Probe girdisi ve akışı

- Oyun modu: `SURVIVAL`
- Container: id `0`, player inventory menu
- Hazırlık: hotbar index `0` / menu slotu `36` içine
  `minecraft:stone[tool={rules:[]}]`
- Stone item registry ID: `1`
- `tool` component type ID: `28`
- Menu state-id: `2` (initial content state `1` sonrasındaki hazırlık state'i)
- Click: slot `36`, button `0`, `PICKUP` input id `0`
- Changed-slot prediction: map içinde slot `36`, pickup sonrası boş hash
- Carried prediction: item ID `1`, count `1`, added component type `28` ve
  Java oracle hash'i `0x37de0062`, removed set'i boş

Probe, item hazırlığı sonrası tek `container_click` paketi yollar; ardından
slot, cursor ve full content güncellemelerini toplar.

Beklenen invariant'lar şunlardır:

- Başlangıç content'i container `0`, state `1`, 46 slot ve carried için boş
  değer taşır.
- Console item hazırlığı slot `36` için state `2`, item ID `1`, count `1` ve
  positive component ID `28` gözlemi üretir.
- Server click'i state-id mismatch saymaz ve pickup davranışını uygular.
- Changed-slot prediction gerçek empty slot ile eşleştiği için click sonrası
  slot düzeltmesi gönderilmez.
- Carried tool component prediction gerçek hash ile eşleştiği için cursor
  düzeltmesi gönderilmez.
- Item hazırlığı dışındaki `container_set_content` gözlemi oluşmaz; full resync
  yolu çalışmaz.

## Sınır

Bu dilim `Tool` component hash'inin taşıdığı stone pickup akışını kapsar.
Tool kuralları içeren farklı değerler, diğer component codec'leri, yanlış veya
kısmi component prediction, diğer click input'ları ve özel container'lar
sonraki P13 dilimleridir.
