# P13 — Survival inventory component correct prediction sözleşmesi

Bu dilim, component taşıyan bir inventory item'ı pickup ederken carried
`HashedStack` içindeki component patch hash'i vanilla ile eşleştiğinde server'ın
ek slot veya cursor düzeltmesi göndermemesini sınar.

## Java referansı

`MultiPlayerGameMode.handleContainerInput`, click sonrası carried
`ItemStack`'ten component patch hash'lerini üretir. `HashedPatchMap` added
component türlerini registry ID ve sabit dört byte integer hash ile, removed
component türlerini registry ID ile taşır.

`ServerPlayer` component hash'lerini registry serialization context ile
`HashOps.CRC32C_INSTANCE` üzerinden üretir. Java oracle'da
`CustomModelData` değeri yalnızca `floats=[1.0]` içerdiğinde bu dilimin
component hash'i `0x77a4a24f` olarak elde edildi.

`ServerGamePacketListenerImpl.handleContainerClick`, state-id eşleştiğinde
changed-slot ve carried hash'lerini remote beklentiye kaydeder ve
`broadcastChanges` çağırır. Pickup sonrası slot empty, carried ise
component'li milk bucket olduğundan iki prediction gerçek server durumuyla
eşleştiğinde ek `container_set_slot` veya `set_cursor_item` paketi oluşmaz.

## Probe girdisi ve akışı

- Oyun modu: `SURVIVAL`
- Container: id `0`, player inventory menu
- Hazırlık: hotbar index `0` / menu slotu `36` içine
  `minecraft:milk_bucket[minecraft:custom_model_data={floats:[1.0f]}]`
- Milk bucket item registry ID: `1046`
- `custom_model_data` component type ID: `17`
- Menu state-id: `2` (initial content state `1` sonrasındaki hazırlık state'i)
- Click: slot `36`, button `0`, `PICKUP` input id `0`
- Changed-slot prediction: map içinde slot `36`, pickup sonrası boş hash
- Carried prediction: item ID `1046`, count `1`, added component type `17` ve
  Java oracle hash'i `0x77a4a24f`, removed set'i boş

Probe, item hazırlığı sonrası tek `container_click` paketi yollar; ardından
slot, cursor ve full content güncellemelerini toplar.

Beklenen invariant'lar şunlardır:

- Başlangıç content'i container `0`, state `1`, 46 slot ve carried için boş
  değer taşır.
- Console item hazırlığı slot `36` için state `2`, item ID `1046`, count `1`
  ve positive component ID `17` gözlemi üretir.
- Server click'i state-id mismatch saymaz ve pickup davranışını uygular.
- Changed-slot prediction gerçek empty slot ile eşleştiği için click sonrası
  slot düzeltmesi gönderilmez.
- Carried component prediction gerçek hash ile eşleştiği için cursor düzeltmesi
  gönderilmez.
- Item hazırlığı dışındaki `container_set_content` gözlemi oluşmaz; full resync
  yolu çalışmaz.

## Sınır

Bu dilim `custom_model_data` taşıyan milk bucket'ta Java CRC32C component
hash'inin doğru taşınmasını kapsar. Diğer component codec'leri, yanlış veya
kısmi component prediction, diğer click input'ları ve özel container'lar
sonraki P13 dilimleridir.
