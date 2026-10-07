# P13 — Survival inventory component hash prediction sözleşmesi — [server-conformance] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

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


## Vanilla server-conformance doğrulaması

**Tarih:** 2026-09-11

**Referans:** Java Edition 26.2 / protokol 776 / Java 25

**Sunucu:** Mojang vanilla 26.2 dedicated server, offline yerel koşu

**Sunucu SHA-1:** `823e2250d24b3ddac457a60c92a6a941943fcd6a`

## Çalıştırma

Sunucu `/tmp/voxel-vanilla-26.2-test` altında, `online-mode=false`, seed `776`,
view/simulation distance `5`, `gamemode=survival`, `force-gamemode=true`,
peaceful ve spawn protection `0` ayarlarıyla çalıştı.

Her oturumda player bağlandıktan sonra vanilla server console'undan şu hazırlık
komutu gönderildi:

```text
item replace entity <player> hotbar.0 with minecraft:milk_bucket[minecraft:custom_model_data={floats:[1.0f]}]
```

Geçerli iki probe koşusu:

```text
cargo run --offline --release --example survival_inventory_component_hash_prediction_probe -- 127.0.0.1:25566 InvCompHash2
cargo run --offline --release --example survival_inventory_component_hash_prediction_probe -- 127.0.0.1:25566 InvCompHash3
```

Probe state `2`, slot `36` için pickup sonrası empty hash'i; carried için item
ID `1046`, count `1`, type `17` added hash'i `0x01020304` ve type `17` removed
set'i taşıyan bilerek yanlış component patch'i gönderdi.

## Sonuçlar

| Gözlem | InvCompHash2 | InvCompHash3 |
| --- | ---: | ---: |
| play login / player loaded | `true / true` | `true / true` |
| container click packets sent | `1` | `1` |
| gönderilen state-id / slot / button / input id | `2 / 36 / 0 / 0` | `2 / 36 / 0 / 0` |
| changed-slot prediction | `slot36=empty` | `slot36=empty` |
| carried prediction | `item1046,count1,added17/removed17,bilerek yanlış hash` | `item1046,count1,added17/removed17,bilerek yanlış hash` |
| initial container content | `state=1, items=46, all empty, carried=empty` | `state=1, items=46, all empty, carried=empty` |
| item hazırlık slot observation | `state=2, slot36=count1,item1046,positive17` | `state=2, slot36=count1,item1046,positive17` |
| click sonrası slot düzeltmesi | `0` | `0` |
| click sonrası cursor update | `item1046,count1,positive17,negative0` | `item1046,count1,positive17,negative0` |
| additional full content observations | `0` | `0` |
| chunk packet / decoded / batch finished | `157 / 157 / 51` | `157 / 157 / 51` |
| received packet | `1757` | `3468` |

Her iki vanilla oturumunda state-id doğru olduğu için click uygulandı ve server
full resync göndermedi. Slot prediction gerçek pickup sonrası empty ile eşleştiği
için slot paketi gelmedi. Yanlış component patch'i nedeniyle cursor update geldi;
bu update item ID `1046`, count `1` ve `custom_model_data` component ID `17`
taşıdı. Server logunda geçerli koşular için protocol error veya timeout
gözlenmedi.

## Sonuç ve açık kapsam

İki bağımsız Mojang vanilla 26.2 oturumu, component patch eşleşmesinin
carried hash içinde doğrulandığını ve yanlış component hash'inin yalnızca
component'li cursor düzeltmesi ürettiğini doğrular. Sözleşme ayrıntıları
[survival inventory component hash prediction contract belgesindedir](#sözleşme).

Diğer component codec'leri, doğru component hash'i, yanlış changed-slot hash'i,
iki hash'in birlikte yanlış olması, kısmi changed-slot map'i, diğer click
input'ları ve özel menu akışları açık kalır.
