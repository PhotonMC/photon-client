# P13 — Survival inventory ahead component correct prediction sözleşmesi

Bu dilim, component taşıyan bir inventory item'ı pickup ederken component hash'i
doğru olsa bile state-id menu state-id'sinin ilerisinde olduğunda vanilla server'ın
component'li full resync seçmesini sınar.

## Java referansı

`ServerGamePacketListenerImpl.handleContainerClick`, packet state-id'si menu
state-id'sine eşit değilse full resync dalını seçer. Bu kontrol ileri ve eski
state-id'leri aynı mismatch olarak ele alır. Click ve remote hash kayıtları
yapıldıktan sonra `AbstractContainerMenu.broadcastFullState` çağrılır.

`ServerPlayer` component hash'lerini registry serialization context ile
`HashOps.CRC32C_INSTANCE` üzerinden üretir. `CustomModelData` içinde yalnızca
`floats=[1.0]` bulunan milk bucket için Java oracle hash'i `0x77a4a24f`'dir.
Bu hash'in doğru olması state mismatch nedeniyle full content yayınının yerini
değiştirmez.

## Probe girdisi ve akışı

- Oyun modu: `SURVIVAL`
- Container: id `0`, player inventory menu
- Hazırlık: hotbar index `0` / menu slotu `36` içine
  `minecraft:milk_bucket[minecraft:custom_model_data={floats:[1.0f]}]`
- Milk bucket item registry ID: `1046`
- `custom_model_data` component type ID: `17`
- Server menu state-id: `2`
- Gönderilen ahead state-id: `99`
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
- Server click'i uygular, fakat ahead state-id mismatch nedeniyle full resync
  dalını seçer.
- Pickup sonrası full content state `3` ile slot `36` empty, carried item
  `1046`, count `1` ve positive component ID `17` olur.
- Doğru component hash'ine rağmen artımlı slot veya cursor düzeltmesi
  gönderilmez; full content gözlemi oluşur.

## Sınır

Bu dilim ahead state-id ile component'li doğru slot/carried prediction'ın
birlikte gönderilmesini ve full resync önceliğini kapsar. Diğer component
codec'leri, yanlış component hash'leri, diğer click input'ları ve özel
container'lar sonraki P13 dilimleridir.
