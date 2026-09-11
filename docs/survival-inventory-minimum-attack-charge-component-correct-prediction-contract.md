# P13 — Survival inventory minimum attack charge component correct prediction sözleşmesi

Bu dilim, `MINIMUM_ATTACK_CHARGE` component'i taşıyan bir inventory item'ı
pickup ederken carried `HashedStack` içindeki component hash'i vanilla ile
eşleştiğinde server'ın ek slot veya cursor düzeltmesi göndermemesini sınar.

## Java referansı

`DataComponents.MINIMUM_ATTACK_CHARGE`, `0.0..1.0` aralığındaki f32 değeri
network stream codec'inde IEEE-754 float olarak taşır. Pinned registry'de
component type ID'si `7`, stone item registry ID'si `1`'dir.

Java oracle, `MINIMUM_ATTACK_CHARGE=0.5` değerinin component hash'ini
`0x9ec7d769` verdi. Hash, component'in persistent codec değerinden; network
wire gövdesi ise dört baytlık big-endian float değerinden oluşur.

`ServerGamePacketListenerImpl.handleContainerClick`, state-id eşleştiğinde
changed-slot ve carried hash'lerini remote beklentiye kaydeder ve
`broadcastChanges` çağırır. Pickup sonrası slot empty, carried ise minimum
attack charge component'li stone olduğundan prediction gerçek server durumuyla
eşleştiğinde ek `container_set_slot` veya `set_cursor_item` paketi oluşmaz.

## Probe girdisi ve akışı

- Oyun modu: `SURVIVAL`
- Container: id `0`, player inventory menu
- Hazırlık: hotbar index `0` / menu slotu `36` içine
  `minecraft:stone[minimum_attack_charge=0.5]`
- Stone item registry ID: `1`
- `minimum_attack_charge` component type ID: `7`
- Component değeri: `0.5f`
- Java oracle hash'i: `0x9ec7d769`
- Menu state-id: `2` (initial content state `1` sonrasındaki hazırlık state'i)
- Click: slot `36`, button `0`, `PICKUP` input id `0`
- Changed-slot prediction: map içinde slot `36`, pickup sonrası boş hash
- Carried prediction: item ID `1`, count `1`, added component type `7` ve Java
  oracle hash'i, removed set'i boş

Probe, item hazırlığı sonrası tek `container_click` paketi yollar; ardından
slot, cursor ve full content güncellemelerini toplar.

Beklenen invariant'lar şunlardır:

- Başlangıç content'i container `0`, state `1`, 46 slot ve carried için boş
  değer taşır.
- Console item hazırlığı slot `36` için state `2`, item ID `1`, count `1` ve
  positive component ID `7` gözlemi üretir.
- Server click'i state-id mismatch saymaz ve pickup davranışını uygular.
- Changed-slot prediction gerçek empty slot ile eşleştiği için click sonrası
  slot düzeltmesi gönderilmez.
- Carried component prediction gerçek hash ile eşleştiği için cursor düzeltmesi
  gönderilmez.
- Item hazırlığı dışındaki `container_set_content` gözlemi oluşmaz; full resync
  yolu çalışmaz.

## Sınır

Bu dilim `MINIMUM_ATTACK_CHARGE=0.5` f32 component'li stone pickup akışını
kapsar. Diğer float değerleri, diğer component codec'leri, yanlış veya kısmi
component prediction, diğer click input'ları ve özel container'lar sonraki P13
dilimleridir.
