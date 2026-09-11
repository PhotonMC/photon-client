# P13 — Vanilla survival inventory wrong slot ve carried prediction doğrulaması

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
cargo run --offline --release --example survival_inventory_wrong_both_prediction_probe -- 127.0.0.1:25566 InvWrongBoth2
cargo run --offline --release --example survival_inventory_wrong_both_prediction_probe -- 127.0.0.1:25566 InvWrongBoth3
```

Probe state `2`, changed-slot hash'i olarak click öncesi stone stack'ini ve
carried hash'i olarak empty değeri gönderdi.

## Sonuçlar

| Gözlem | InvWrongBoth2 | InvWrongBoth3 |
| --- | ---: | ---: |
| play login / player loaded | `true / true` | `true / true` |
| container click packets sent | `1` | `1` |
| gönderilen state-id / slot / button / input id | `2 / 36 / 0 / 0` | `2 / 36 / 0 / 0` |
| changed-slot prediction | `slot36=item1,count8,empty components` | `slot36=item1,count8,empty components` |
| carried prediction | `empty` | `empty` |
| initial container content | `state=1, items=46, all empty, carried=empty` | `state=1, items=46, all empty, carried=empty` |
| item hazırlık slot observation | `state=2, slot36=count8,item1` | `state=2, slot36=count8,item1` |
| click sonrası slot düzeltmesi | `state=3, slot36=empty` | `state=3, slot36=empty` |
| click sonrası cursor update | `set_cursor_item,count8,item1,empty components` | `set_cursor_item,count8,item1,empty components` |
| additional full content observations | `0` | `0` |
| chunk packet / decoded / batch finished | `157 / 157 / 51` | `157 / 157 / 51` |
| received packet | `1595` | `2506` |

Her iki vanilla oturumunda state-id doğru olduğu için click uygulandı ve server
full resync göndermedi. İki remote hash gerçek click sonrası durumdan farklı
olduğu için state `3` empty slot ve count `8` stone cursor düzeltmeleri birlikte
geldi. Server logunda geçerli koşular için protocol error veya timeout
gözlenmedi.

## Sonuç ve açık kapsam

İki bağımsız Mojang vanilla 26.2 oturumu, changed-slot ve carried hash'lerinin
bağımsız karşılaştırıldığını ve iki yanlış hash'in iki ayrı authoritative
düzeltme ürettiğini doğrular. Sözleşme ayrıntıları [survival inventory wrong
slot ve carried prediction contract belgesindedir](survival-inventory-wrong-both-prediction-contract.md).

Component hash'leri, kısmi changed-slot map'i, diğer click input'ları, stale
state varyantları ve özel menu akışları açık kalır.
