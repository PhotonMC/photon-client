# P13 — Survival inventory cat collar component correct prediction sözleşmesi

Bu dilim, `CAT_COLLAR` component'i taşıyan bir inventory item'ı pickup ederken
carried `HashedStack` içindeki component hash'i vanilla ile eşleştiğinde
server'ın ek slot veya cursor düzeltmesi göndermemesini sınar.

## Java referansı

`DataComponents.CAT_COLLAR`, `DyeColor.CODEC` ile dye rengini persistent olarak
ve `DyeColor.STREAM_CODEC` ile color ID'sini VarInt olarak taşır. `Cat`, bu
component'i collar renginden okur ve implicit component olarak uygular. Pinned
runtime component registry'de type ID'si `108`, stone item registry ID'si `1`'dir.
`DyeColor.RED` color ID'si `14`; Java CRC32C component hash'i `0x8c6452aa`'dır.

`ServerGamePacketListenerImpl.handleContainerClick`, state-id eşleştiğinde
changed-slot ve carried hash'lerini remote beklentiye kaydeder. Pickup sonrası
slot empty, carried ise cat collar component'li stone olduğundan prediction
gerçek server durumuyla eşleştiğinde ek slot/cursor yayını oluşmaz.

## Probe girdisi ve akışı

- Oyun modu: `SURVIVAL`
- Container: id `0`, player inventory menu
- Hazırlık: hotbar index `0` / menu slotu `36` içine
  `minecraft:stone[cat/collar=red]`
- Stone item registry ID: `1`
- `cat/collar` component type ID: `108`
- Component değeri: `DyeColor.RED`, color ID `14`
- Java CRC32C oracle hash'i: `0x8c6452aa`
- Menu state-id: hazırlık slotunda gözlenen son state-id (`2` veya daha yeni
  component hazırlık güncellemesi varsa `3`)
- Click: slot `36`, button `0`, `PICKUP` input id `0`
- Changed-slot prediction: map içinde slot `36`, pickup sonrası boş hash
- Carried prediction: item ID `1`, count `1`, added component type `108` ve Java
  oracle hash'i, removed set'i boş

Probe, preparation component'i slot state'inde gözlenene kadar paketleri toplar
ve son state-id ile tek `container_click` paketi yollar; ardından slot, cursor
ve full content güncellemelerini toplar.

Beklenen invariant'lar şunlardır:

- Başlangıç content'i container `0`, state `1`, 46 slot ve carried için boş
  değer taşır.
- Console item hazırlığı slot `36` için item ID `1`, count `1` ve positive
  component ID `108` gözlemi üretir.
- Server click'i preparation state-id ile mismatch saymaz ve pickup davranışını
  uygular.
- Changed-slot prediction gerçek empty slot ile eşleştiği için click sonrası
  slot düzeltmesi gönderilmez.
- Carried cat collar prediction gerçek component hash'i ile eşleştiği için
  cursor düzeltmesi gönderilmez.
- Item hazırlığı dışındaki `container_set_content` gözlemi oluşmaz; full resync
  yolu çalışmaz.

## Sınır

Bu dilim `CAT_COLLAR` component'li stone pickup akışını kapsar. Diğer entity
component'leri, diğer dye renkleri, diğer component codec'leri, yanlış veya
kısmi component prediction, diğer click input'ları ve özel container'lar sonraki
P13 dilimleridir.
