# P13 — Survival inventory food component correct prediction sözleşmesi

Bu dilim, food component taşıyan bir inventory item'ı pickup ederken carried
`HashedStack` içindeki component hash'i vanilla ile eşleştiğinde server'ın ek
slot veya cursor düzeltmesi göndermemesini sınar.

## Java referansı

`FoodProperties.DIRECT_CODEC`, food component değerini `nutrition`,
`saturation` ve `can_always_eat` alanlarından oluşturur. `DataComponents.FOOD`
bu codec'i ve aynı değerin network stream codec'ini kullanır. `Pufferfish`
item'ının registry ID'si `1089`, food component type ID'si `23` olarak
registry'de sabittir.

`ServerPlayer` component hash'lerini registry serialization context ile
`HashOps.CRC32C_INSTANCE` üzerinden üretir. Java oracle'da
`nutrition=1`, `saturation=0.1` ve `can_always_eat=true` değerleri için hash
`0xfac69b4d` elde edildi.

`ServerGamePacketListenerImpl.handleContainerClick`, state-id eşleştiğinde
changed-slot ve carried hash'lerini remote beklentiye kaydeder ve
`broadcastChanges` çağırır. Pickup sonrası slot empty, carried ise food
component'li pufferfish olduğundan iki prediction gerçek server durumuyla
eşleştiğinde ek `container_set_slot` veya `set_cursor_item` paketi oluşmaz.

## Probe girdisi ve akışı

- Oyun modu: `SURVIVAL`
- Container: id `0`, player inventory menu
- Hazırlık: hotbar index `0` / menu slotu `36` içine
  `minecraft:pufferfish[food={nutrition:1,saturation:0.1,can_always_eat:true}]`
- Pufferfish item registry ID: `1089`
- `food` component type ID: `23`
- Menu state-id: `2` (initial content state `1` sonrasındaki hazırlık state'i)
- Click: slot `36`, button `0`, `PICKUP` input id `0`
- Changed-slot prediction: map içinde slot `36`, pickup sonrası boş hash
- Carried prediction: item ID `1089`, count `1`, added component type `23` ve
  Java oracle hash'i `0xfac69b4d`, removed set'i boş

Probe, item hazırlığı sonrası tek `container_click` paketi yollar; ardından
slot, cursor ve full content güncellemelerini toplar.

Beklenen invariant'lar şunlardır:

- Başlangıç content'i container `0`, state `1`, 46 slot ve carried için boş
  değer taşır.
- Console item hazırlığı slot `36` için state `2`, item ID `1089`, count `1`
  ve positive component ID `23` gözlemi üretir.
- Server click'i state-id mismatch saymaz ve pickup davranışını uygular.
- Changed-slot prediction gerçek empty slot ile eşleştiği için click sonrası
  slot düzeltmesi gönderilmez.
- Carried food component prediction gerçek hash ile eşleştiği için cursor
  düzeltmesi gönderilmez.
- Item hazırlığı dışındaki `container_set_content` gözlemi oluşmaz; full resync
  yolu çalışmaz.

## Sınır

Bu dilim `FoodProperties` codec'inin component hash'ini taşıyan pufferfish
pickup akışını kapsar. Diğer component codec'leri, yanlış veya kısmi component
prediction, diğer click input'ları ve özel container'lar sonraki P13
dilimleridir.
