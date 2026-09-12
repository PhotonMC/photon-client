# P13 — Survival inventory damage type component correct prediction sözleşmesi

Bu dilim, `DAMAGE_TYPE` component'i taşıyan bir inventory item'ı pickup ederken
carried `HashedStack` içindeki component hash'i vanilla ile eşleştiğinde
server'ın ek slot veya cursor düzeltmesi göndermemesini sınar.

## Java referansı

`DataComponents.DAMAGE_TYPE`, `DamageType.CODEC` üzerinden registry'ye bağlı bir
`Holder<DamageType>` taşır. Persistent codec değeri registry-aware serialization
context içinde damage type anahtarını encode eder. Network stream codec'i
`ByteBufCodecs.holderRegistry` kullanır ve referans holder'ın registry ID'sini
VarInt olarak taşır. Pinned component registry'de component type ID'si `8`,
stone item registry ID'si `1`, `minecraft:generic` damage type registry ID'si
`27`'dir.

Java oracle, `minecraft:generic` holder'ının component hash'ini
`0x55e33a50` verdi.

`ServerGamePacketListenerImpl.handleContainerClick`, state-id eşleştiğinde
changed-slot ve carried hash'lerini remote beklentiye kaydeder ve
`broadcastChanges` çağırır. Pickup sonrası slot empty, carried ise generic
damage type component'li stone olduğundan prediction gerçek server durumuyla
eşleştiğinde ek `container_set_slot` veya `set_cursor_item` paketi oluşmaz.

## Probe girdisi ve akışı

- Oyun modu: `SURVIVAL`
- Container: id `0`, player inventory menu
- Hazırlık: hotbar index `0` / menu slotu `36` içine
  `minecraft:stone[damage_type=generic]`
- Stone item registry ID: `1`
- `damage_type` component type ID: `8`
- Component değeri: `minecraft:generic`, registry ID `27`
- Java oracle hash'i: `0x55e33a50`
- Menu state-id: `2` (initial content state `1` sonrasındaki hazırlık state'i)
- Click: slot `36`, button `0`, `PICKUP` input id `0`
- Changed-slot prediction: map içinde slot `36`, pickup sonrası boş hash
- Carried prediction: item ID `1`, count `1`, added component type `8` ve Java
  oracle hash'i, removed set'i boş

Probe, item hazırlığı sonrası tek `container_click` paketi yollar; ardından
slot, cursor ve full content güncellemelerini toplar.

Beklenen invariant'lar şunlardır:

- Başlangıç content'i container `0`, state `1`, 46 slot ve carried için boş
  değer taşır.
- Console item hazırlığı slot `36` için state `2`, item ID `1`, count `1` ve
  positive component ID `8` gözlemi üretir.
- Server click'i state-id mismatch saymaz ve pickup davranışını uygular.
- Changed-slot prediction gerçek empty slot ile eşleştiği için click sonrası
  slot düzeltmesi gönderilmez.
- Carried damage type prediction gerçek hash ile eşleştiği için cursor düzeltmesi
  gönderilmez.
- Item hazırlığı dışındaki `container_set_content` gözlemi oluşmaz; full resync
  yolu çalışmaz.

## Sınır

Bu dilim `DAMAGE_TYPE=minecraft:generic` component'li stone pickup akışını
kapsar. Diğer damage type değerleri, diğer component codec'leri, yanlış veya
kısmi component prediction, diğer click input'ları ve özel container'lar
sonraki P13 dilimleridir.
