# P13 — Vanilla survival inventory partial changed-slot prediction doğrulaması

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
item replace entity <player> hotbar.0 with minecraft:stone 8
```

Geçerli iki probe koşusu:

```text
cargo run --offline --release --example survival_inventory_partial_changed_slot_prediction_probe -- 127.0.0.1:25566 InvPart2
cargo run --offline --release --example survival_inventory_partial_changed_slot_prediction_probe -- 127.0.0.1:25566 InvPart3
```

Probe state `2`, slot `36` için changed-slot hash map'ini boş gönderdi; carried
prediction ise item ID `1`, count `8` ve boş component map/set içeren doğru
non-empty hash olarak gönderildi.

## Sonuçlar

| Gözlem | InvPart2 | InvPart3 |
| --- | ---: | ---: |
| play login / player loaded | `true / true` | `true / true` |
| container click packets sent | `1` | `1` |
| gönderilen state-id / slot / button / input id | `2 / 36 / 0 / 0` | `2 / 36 / 0 / 0` |
| changed-slot prediction | `boş map` | `boş map` |
| carried prediction | `item1,count8,empty components` | `item1,count8,empty components` |
| initial container content | `state=1, items=46, all empty, carried=empty` | `state=1, items=46, all empty, carried=empty` |
| item hazırlık slot observation | `state=2, slot36=count8,item1` | `state=2, slot36=count8,item1` |
| click sonrası slot düzeltmesi | `state=3, slot36=empty` | `state=3, slot36=empty` |
| click sonrası cursor update | `0` | `0` |
| additional full content observations | `0` | `0` |
| chunk packet / decoded / batch finished | `157 / 157 / 51` | `157 / 157 / 51` |
| received packet | `1424` | `1442` |

Her iki vanilla oturumunda state-id doğru olduğu için pickup uygulandı. Changed
slot map'inde slot `36` bulunmadığından önceki remote beklenti korundu ve gerçek
empty slot için yalnızca state `3` `container_set_slot` düzeltmesi geldi. Doğru
carried hash'i cursor düzeltmesini bastırdı; item hazırlığı dışındaki full
content paket sayısı `0` kaldı. Server logunda geçerli koşular için protocol
error veya timeout gözlenmedi.

## Sonuç ve açık kapsam

İki bağımsız Mojang vanilla 26.2 oturumu, changed-slot hash girdisi eksik olsa
da carried prediction'ın bağımsız değerlendirildiğini ve yalnızca slot
düzeltmesi üretildiğini doğrular. Sözleşme ayrıntıları [survival inventory
partial changed-slot prediction contract belgesindedir](survival-inventory-partial-changed-slot-prediction-contract.md).

Birden fazla eksik slot, yanlış veya kısmi carried component prediction, diğer
click input'ları ve özel container akışları açık kalır.
