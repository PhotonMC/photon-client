# P13 — Survival inventory use effects component correct prediction sözleşmesi

Bu dilim, `USE_EFFECTS` component'i taşıyan bir inventory item'ı pickup ederken
carried `HashedStack` içindeki component hash'i vanilla ile eşleştiğinde
server'ın ek slot veya cursor düzeltmesi göndermemesini sınar.

## Java referansı

`DataComponents.USE_EFFECTS`, `UseEffects` kaydındaki `can_sprint`,
`interact_vibrations` ve `speed_multiplier` alanlarını taşır. Persistent codec
alanları sırasıyla boolean, boolean ve `0.0..1.0` float olarak doğrular.
Network stream codec'i aynı sırayı BOOL, BOOL ve dört baytlık FLOAT olarak
kullanır. Pinned registry'de component type ID'si `5`, stone item registry
ID'si `1`'dir.

Java oracle, `UseEffects(true, false, 1.0F)` değerinin component hash'ini
`0x9619e224` verdi.

`ServerGamePacketListenerImpl.handleContainerClick`, state-id eşleştiğinde
changed-slot ve carried hash'lerini remote beklentiye kaydeder ve
`broadcastChanges` çağırır. Pickup sonrası slot empty, carried ise use effects
component'li stone olduğundan prediction gerçek server durumuyla eşleştiğinde
ek `container_set_slot` veya `set_cursor_item` paketi oluşmaz.

## Probe girdisi ve akışı

- Oyun modu: `SURVIVAL`
- Container: id `0`, player inventory menu
- Hazırlık: hotbar index `0` / menu slotu `36` içine
  `minecraft:stone[use_effects={can_sprint:true,interact_vibrations:false,speed_multiplier:1.0}]`
- Stone item registry ID: `1`
- `use_effects` component type ID: `5`
- Component değeri: `can_sprint=true`, `interact_vibrations=false`,
  `speed_multiplier=1.0`
- Java oracle hash'i: `0x9619e224`
- Menu state-id: `2` (initial content state `1` sonrasındaki hazırlık state'i)
- Click: slot `36`, button `0`, `PICKUP` input id `0`
- Changed-slot prediction: map içinde slot `36`, pickup sonrası boş hash
- Carried prediction: item ID `1`, count `1`, added component type `5` ve Java
  oracle hash'i, removed set'i boş

Probe, item hazırlığı sonrası tek `container_click` paketi yollar; ardından
slot, cursor ve full content güncellemelerini toplar.

Beklenen invariant'lar şunlardır:

- Başlangıç content'i container `0`, state `1`, 46 slot ve carried için boş
  değer taşır.
- Console item hazırlığı slot `36` için state `2`, item ID `1`, count `1` ve
  positive component ID `5` gözlemi üretir.
- Server click'i state-id mismatch saymaz ve pickup davranışını uygular.
- Changed-slot prediction gerçek empty slot ile eşleştiği için click sonrası
  slot düzeltmesi gönderilmez.
- Carried use effects prediction gerçek hash ile eşleştiği için cursor düzeltmesi
  gönderilmez.
- Item hazırlığı dışındaki `container_set_content` gözlemi oluşmaz; full resync
  yolu çalışmaz.

## Sınır

Bu dilim `UseEffects(true, false, 1.0F)` component'li stone pickup akışını
kapsar. Diğer alan değerleri, diğer component codec'leri, yanlış veya kısmi
component prediction, diğer click input'ları ve özel container'lar sonraki P13
dilimleridir.
