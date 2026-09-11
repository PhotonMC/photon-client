# P13 — Survival inventory max damage component correct prediction sözleşmesi

Bu dilim, `MAX_DAMAGE` component'i taşıyan bir inventory item'ı pickup ederken
carried `HashedStack` içindeki component hash'leri vanilla ile eşleştiğinde
server'ın ek slot veya cursor düzeltmesi göndermemesini sınar.

## Java referansı

`DataComponents.MAX_DAMAGE`, pozitif integer değeri network stream codec'inde
VarInt olarak taşır. Pinned registry'de max damage component type ID'si `2`,
`MAX_STACK_SIZE` type ID'si `1` ve stone item registry ID'si `1`'dir.

Vanilla, damageable bir stack'in stackable olmasını kabul etmediği için probe
item'ı iki component'i birlikte taşır:
`minecraft:stone[max_stack_size=1,max_damage=1]`. Her iki component'in değeri
`1` olduğundan Java oracle'da iki component hash'i de `0x5d50d71c` elde edilir;
component type ID'si hash değerinin içine katılmaz.

`ServerGamePacketListenerImpl.handleContainerClick`, state-id eşleştiğinde
changed-slot ve carried hash'lerini remote beklentiye kaydeder ve
`broadcastChanges` çağırır. Pickup sonrası slot empty, carried ise iki
component'li stone olduğundan prediction gerçek server durumuyla eşleştiğinde
ek `container_set_slot` veya `set_cursor_item` paketi oluşmaz.

## Probe girdisi ve akışı

- Oyun modu: `SURVIVAL`
- Container: id `0`, player inventory menu
- Hazırlık: hotbar index `0` / menu slotu `36` içine
  `minecraft:stone[max_stack_size=1,max_damage=1]`
- Stone item registry ID: `1`
- `max_stack_size` component type ID: `1`
- `max_damage` component type ID: `2`
- Her iki component için değer `1` ve Java oracle hash'i `0x5d50d71c`
- Menu state-id: `2` (initial content state `1` sonrasındaki hazırlık state'i)
- Click: slot `36`, button `0`, `PICKUP` input id `0`
- Changed-slot prediction: map içinde slot `36`, pickup sonrası boş hash
- Carried prediction: item ID `1`, count `1`, added component type `1` ve `2`,
  her ikisi için Java oracle hash'i, removed set'i boş

Probe, item hazırlığı sonrası tek `container_click` paketi yollar; ardından
slot, cursor ve full content güncellemelerini toplar.

Beklenen invariant'lar şunlardır:

- Başlangıç content'i container `0`, state `1`, 46 slot ve carried için boş
  değer taşır.
- Console item hazırlığı slot `36` için state `2`, item ID `1`, count `1` ve
  positive component ID'leri `1, 2` gözlemi üretir.
- Server click'i state-id mismatch saymaz ve pickup davranışını uygular.
- Changed-slot prediction gerçek empty slot ile eşleştiği için click sonrası
  slot düzeltmesi gönderilmez.
- Carried prediction iki component'in gerçek hash'leriyle eşleştiği için cursor
  düzeltmesi gönderilmez.
- Item hazırlığı dışındaki `container_set_content` gözlemi oluşmaz; full resync
  yolu çalışmaz.

## Sınır

Bu dilim, `MAX_DAMAGE=1` ile birlikte gerekli `MAX_STACK_SIZE=1` component'ini
taşıyan stone pickup akışını kapsar. Diğer max damage değerleri, diğer
component codec'leri, yanlış veya kısmi component prediction, diğer click
input'ları ve özel container'lar sonraki P13 dilimleridir.
