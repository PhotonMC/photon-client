# P13 — Survival inventory component hash prediction sözleşmesi

Bu dilim, component taşıyan bir inventory item'ı pickup ederken carried
`HashedStack` içindeki component patch hash'i yanlış olduğunda vanilla
server'ın component'li cursor düzeltmesi göndermesini sınar.

## Java referansı

`MultiPlayerGameMode.handleContainerInput`, click sonrası carried
`ItemStack` patch'ini `HashedPatchMap.create` ile component türü başına integer
hash'e çevirir. `HashedPatchMap.STREAM_CODEC` added map anahtarını data
component registry ID'siyle, değeri sabit dört byte `INT` ile; removed set'i
registry ID'leriyle taşır.

`ServerGamePacketListenerImpl.handleContainerClick`, state-id eşleştiğinde
gelen changed-slot ve carried hash'lerini remote beklentiye kaydeder. Slot
prediction doğru empty olduğu için slot düzeltmesi gerekmez. Carried prediction
component patch'i gerçek stack ile eşleşmediği için
`AbstractContainerMenu.broadcastChanges`, gerçek component patch'ini taşıyan
`ClientboundSetCursorItemPacket` gönderir.

`custom_model_data` component'i bu koşuda type ID `17`, milk bucket item'ı
registry ID `1046`'dır. Rust clientbound item parser'ı component payload'ını
tüketip component type ID'sini gözleme yazar.

## Probe girdisi ve akışı

- Oyun modu: `SURVIVAL`
- Hazırlık: hotbar index `0` / menu slotu `36` içine
  `minecraft:milk_bucket[minecraft:custom_model_data={floats:[1.0f]}]`
- Menu state-id: `2` (initial content state `1` sonrasındaki hazırlık state'i)
- Click: slot `36`, button `0`, `PICKUP` input id `0`
- Changed-slot prediction: pickup sonrası empty hash
- Carried prediction: item ID `1046`, count `1`; added component type `17`
  için bilerek yanlış fixed `INT` hash `0x01020304` ve bilerek yanlış removed
  component type `17`

Probe, item hazırlığı sonrası tek `container_click` paketi yollar; ardından
slot, cursor ve full content güncellemelerini toplar.

Beklenen invariant'lar şunlardır:

- Başlangıç content'i container `0`, state `1`, 46 slot ve carried için boş
  değer taşır.
- Console item hazırlığı slot `36` için state `2`, item ID `1046`, count `1`
  ve positive component ID `17` gözlemi üretir.
- Server click'i state-id mismatch saymaz ve pickup davranışını uygular.
- Changed-slot prediction doğru olduğu için click sonrası slot düzeltmesi
  gönderilmez.
- Yanlış component hash/patch'i nedeniyle cursor için item ID `1046`, count `1`
  ve component ID `17` taşıyan `set_cursor_item` paketi gönderilir.
- Item hazırlığı dışındaki `container_set_content` gözlemi oluşmaz; full resync
  yolu çalışmaz.

## Sınır

Bu dilim `custom_model_data` taşıyan milk bucket'ta yanlış added/removed
component hash'inin cursor düzeltmesine yol açmasını kapsar. Diğer component
codec'leri, doğru component hash'i, yanlış changed-slot hash'i, iki hash'in
birlikte yanlış olması, kısmi changed-slot map'i, diğer click input'ları ve
özel container'lar sonraki P13 dilimleridir.
