# P13 — Survival inventory enchantment glint override component correct prediction sözleşmesi

Bu dilim, `ENCHANTMENT_GLINT_OVERRIDE` component'i taşıyan bir inventory
item'ı pickup ederken carried `HashedStack` içindeki component hash'i vanilla
ile eşleştiğinde server'ın ek slot veya cursor düzeltmesi göndermemesini sınar.

## Java referansı

`DataComponents.ENCHANTMENT_GLINT_OVERRIDE`, boolean değeri network
stream codec'inde BOOL olarak taşır. Pinned registry'de component type ID'si
`21`, stone item registry ID'si `1`'dir.

Java oracle, `ENCHANTMENT_GLINT_OVERRIDE=true` değerinin component hash'ini
`0xc336cec2` verdi.

`ServerGamePacketListenerImpl.handleContainerClick`, state-id eşleştiğinde
changed-slot ve carried hash'lerini remote beklentiye kaydeder ve
`broadcastChanges` çağırır. Pickup sonrası slot empty, carried ise glint
override component'li stone olduğundan prediction gerçek server durumuyla
eşleştiğinde ek `container_set_slot` veya `set_cursor_item` paketi oluşmaz.

## Probe girdisi ve akışı

- Oyun modu: `SURVIVAL`
- Container: id `0`, player inventory menu
- Hazırlık: hotbar index `0` / menu slotu `36` içine
  `minecraft:stone[enchantment_glint_override=true]`
- Stone item registry ID: `1`
- `enchantment_glint_override` component type ID: `21`
- Component değeri: `true`
- Java oracle hash'i: `0xc336cec2`
- Menu state-id: `2` (initial content state `1` sonrasındaki hazırlık state'i)
- Click: slot `36`, button `0`, `PICKUP` input id `0`
- Changed-slot prediction: map içinde slot `36`, pickup sonrası boş hash
- Carried prediction: item ID `1`, count `1`, added component type `21` ve Java
  oracle hash'i, removed set'i boş

Probe, item hazırlığı sonrası tek `container_click` paketi yollar; ardından
slot, cursor ve full content güncellemelerini toplar.

Beklenen invariant'lar şunlardır:

- Başlangıç content'i container `0`, state `1`, 46 slot ve carried için boş
  değer taşır.
- Console item hazırlığı slot `36` için state `2`, item ID `1`, count `1` ve
  positive component ID `21` gözlemi üretir.
- Server click'i state-id mismatch saymaz ve pickup davranışını uygular.
- Changed-slot prediction gerçek empty slot ile eşleştiği için click sonrası
  slot düzeltmesi gönderilmez.
- Carried component prediction gerçek hash ile eşleştiği için cursor düzeltmesi
  gönderilmez.
- Item hazırlığı dışındaki `container_set_content` gözlemi oluşmaz; full resync
  yolu çalışmaz.

## Sınır

Bu dilim `ENCHANTMENT_GLINT_OVERRIDE=true` boolean component'li stone pickup
akışını kapsar. `false` değeri, diğer component codec'leri, yanlış veya kısmi
component prediction, diğer click input'ları ve özel container'lar sonraki P13
dilimleridir.
