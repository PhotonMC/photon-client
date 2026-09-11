# P13 — Survival inventory max stack size component correct prediction sözleşmesi

Bu dilim, max stack size component taşıyan bir inventory item'ı pickup ederken
carried `HashedStack` içindeki component hash'i vanilla ile eşleştiğinde
server'ın ek slot veya cursor düzeltmesi göndermemesini sınar.

## Java referansı

`DataComponents.MAX_STACK_SIZE`, değeri `1..99` aralığında integer olarak
tanımlar ve network stream codec'inde VarInt kullanır. Pinned registry'de max
stack size component type ID'si `1`'dir. Stone item registry ID'si `1`'dir.

Java oracle'da `DataComponents.MAX_STACK_SIZE` değeri `1` için hash
`0x5d50d71c` elde edildi. Bu, `DAMAGE=1` oracle'ı ile aynı sonuçtur; component
türü hash değerine katılmaz, her iki codec de aynı integer VarInt değerini
encode eder.

`ServerGamePacketListenerImpl.handleContainerClick`, state-id eşleştiğinde
changed-slot ve carried hash'lerini remote beklentiye kaydeder ve
`broadcastChanges` çağırır. Pickup sonrası slot empty, carried ise max stack
size component'li stone olduğundan iki prediction gerçek server durumuyla
eşleştiğinde ek `container_set_slot` veya `set_cursor_item` paketi oluşmaz.

## Probe girdisi ve akışı

- Oyun modu: `SURVIVAL`
- Container: id `0`, player inventory menu
- Hazırlık: hotbar index `0` / menu slotu `36` içine
  `minecraft:stone[max_stack_size=1]`
- Stone item registry ID: `1`
- `max_stack_size` component type ID: `1`
- Menu state-id: `2` (initial content state `1` sonrasındaki hazırlık state'i)
- Click: slot `36`, button `0`, `PICKUP` input id `0`
- Changed-slot prediction: map içinde slot `36`, pickup sonrası boş hash
- Carried prediction: item ID `1`, count `1`, added component type `1` ve
  Java oracle hash'i `0x5d50d71c`, removed set'i boş

Probe, item hazırlığı sonrası tek `container_click` paketi yollar; ardından
slot, cursor ve full content güncellemelerini toplar.

Beklenen invariant'lar şunlardır:

- Başlangıç content'i container `0`, state `1`, 46 slot ve carried için boş
  değer taşır.
- Console item hazırlığı slot `36` için state `2`, item ID `1`, count `1` ve
  positive component ID `1` gözlemi üretir.
- Server click'i state-id mismatch saymaz ve pickup davranışını uygular.
- Changed-slot prediction gerçek empty slot ile eşleştiği için click sonrası
  slot düzeltmesi gönderilmez.
- Carried max stack size component prediction gerçek hash ile eşleştiği için
  cursor düzeltmesi gönderilmez.
- Item hazırlığı dışındaki `container_set_content` gözlemi oluşmaz; full resync
  yolu çalışmaz.

## Sınır

Bu dilim integer `MAX_STACK_SIZE` component hash'inin taşıdığı stone pickup
akışını kapsar. Diğer integer component değerleri, diğer component codec'leri,
yanlış veya kısmi component prediction, diğer click input'ları ve özel
container'lar sonraki P13 dilimleridir.
