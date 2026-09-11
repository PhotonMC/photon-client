# P13 — Vanilla survival inventory wrong changed-slot prediction doğrulaması

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
cargo run --offline --release --example survival_inventory_wrong_slot_prediction_probe -- 127.0.0.1:25566 InvWrongSlot2
cargo run --offline --release --example survival_inventory_wrong_slot_prediction_probe -- 127.0.0.1:25566 InvWrongSlot3
```

Probe state `2`, slot `36`, changed-slot hash'i olarak click öncesi stone
stack'i ve carried hash'i olarak click sonrası stone stack'ini gönderdi.

## Sonuçlar

| Gözlem | InvWrongSlot2 | InvWrongSlot3 |
| --- | ---: | ---: |
| play login / player loaded | `true / true` | `true / true` |
| container click packets sent | `1` | `1` |
| gönderilen state-id / slot / button / input id | `2 / 36 / 0 / 0` | `2 / 36 / 0 / 0` |
| changed-slot prediction | `slot36=item1,count8,empty components` | `slot36=item1,count8,empty components` |
| carried prediction | `item1,count8,empty components` | `item1,count8,empty components` |
| initial container content | `state=1, items=46, all empty, carried=empty` | `state=1, items=46, all empty, carried=empty` |
| item hazırlık slot observation | `state=2, slot36=count8,item1` | `state=2, slot36=count8,item1` |
| click sonrası slot düzeltmesi | `state=3, slot36=empty` | `state=3, slot36=empty` |
| click sonrası cursor update | `0` | `0` |
| additional full content observations | `0` | `0` |
| chunk packet / decoded / batch finished | `157 / 157 / 51` | `157 / 157 / 51` |
| received packet | `1078` | `1246` |

Her iki vanilla oturumunda state-id doğru olduğu için click uygulandı ve server
full resync göndermedi. Changed-slot remote hash'i click öncesi stone stack'i
olarak kaldığından yalnızca state `3` slot düzeltmesi geldi. Carried hash'i
click sonrası stone stack'iyle eşleştiği için cursor paketi gelmedi. Server
logunda geçerli koşular için protocol error veya timeout gözlenmedi.

## Sonuç ve açık kapsam

İki bağımsız Mojang vanilla 26.2 oturumu, yanlış changed-slot hash'inin doğru
carried prediction ile birlikte yalnızca authoritative slot düzeltmesi
ürettiğini doğrular. Sözleşme ayrıntıları [survival inventory wrong changed-slot
prediction contract belgesindedir](survival-inventory-wrong-slot-prediction-contract.md).

Yanlış carried hash'i, iki hash'in birlikte yanlış olması, component hash'leri,
kısmi changed-slot map'i, diğer click input'ları ve özel menu akışları açık
kalır.
