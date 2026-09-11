# P13 — Vanilla survival inventory component hash prediction doğrulaması

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
[survival inventory component hash prediction contract belgesindedir](survival-inventory-component-hash-prediction-contract.md).

Diğer component codec'leri, doğru component hash'i, yanlış changed-slot hash'i,
iki hash'in birlikte yanlış olması, kısmi changed-slot map'i, diğer click
input'ları ve özel menu akışları açık kalır.
