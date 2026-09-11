# P13 — Survival inventory weapon component correct prediction sözleşmesi

Bu dilim, weapon component taşıyan bir inventory item'ı pickup ederken carried
`HashedStack` içindeki component hash'i vanilla ile eşleştiğinde server'ın ek
slot veya cursor düzeltmesi göndermemesini sınar.

## Java referansı

`Weapon.CODEC`, `item_damage_per_attack` ve `disable_blocking_for_seconds`
alanlarını encode eder. Varsayılan değerler sırasıyla `1` ve `0.0F`'dir.
`DataComponents.WEAPON` bu codec'i network-synchronized component olarak
kaydeder. Pinned registry'de weapon component type ID'si `29`'dur. Stone item
registry ID'si `1`'dir.

Java oracle'da `Weapon(1, 0.0F)` değeri için hash `0xc574b4c8` elde edildi.

`ServerGamePacketListenerImpl.handleContainerClick`, state-id eşleştiğinde
changed-slot ve carried hash'lerini remote beklentiye kaydeder ve
`broadcastChanges` çağırır. Pickup sonrası slot empty, carried ise weapon
component'li stone olduğundan iki prediction gerçek server durumuyla
eşleştiğinde ek `container_set_slot` veya `set_cursor_item` paketi oluşmaz.

## Probe girdisi ve akışı

- Oyun modu: `SURVIVAL`
- Container: id `0`, player inventory menu
- Hazırlık: hotbar index `0` / menu slotu `36` içine
  `minecraft:stone[weapon={item_damage_per_attack:1,disable_blocking_for_seconds:0.0}]`
- Stone item registry ID: `1`
- `weapon` component type ID: `29`
- Menu state-id: `2` (initial content state `1` sonrasındaki hazırlık state'i)
- Click: slot `36`, button `0`, `PICKUP` input id `0`
- Changed-slot prediction: map içinde slot `36`, pickup sonrası boş hash
- Carried prediction: item ID `1`, count `1`, added component type `29` ve
  Java oracle hash'i `0xc574b4c8`, removed set'i boş

Probe, item hazırlığı sonrası tek `container_click` paketi yollar; ardından
slot, cursor ve full content güncellemelerini toplar.

Beklenen invariant'lar şunlardır:

- Başlangıç content'i container `0`, state `1`, 46 slot ve carried için boş
  değer taşır.
- Console item hazırlığı slot `36` için state `2`, item ID `1`, count `1` ve
  positive component ID `29` gözlemi üretir.
- Server click'i state-id mismatch saymaz ve pickup davranışını uygular.
- Changed-slot prediction gerçek empty slot ile eşleştiği için click sonrası
  slot düzeltmesi gönderilmez.
- Carried weapon component prediction gerçek hash ile eşleştiği için cursor
  düzeltmesi gönderilmez.
- Item hazırlığı dışındaki `container_set_content` gözlemi oluşmaz; full resync
  yolu çalışmaz.

## Sınır

Bu dilim `Weapon` component hash'inin taşıdığı stone pickup akışını kapsar.
Farklı weapon alanları, diğer component codec'leri, yanlış veya kısmi component
prediction, diğer click input'ları ve özel container'lar sonraki P13
dilimleridir.
