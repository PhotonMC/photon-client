# P13 — Vanilla survival inventory double-click state-id correction doğrulaması

**Tarih:** 2026-09-11

**Referans:** Java Edition 26.2 / protokol 776 / Java 25

**Sunucu:** Mojang vanilla 26.2 dedicated server, offline yerel koşu

**Sunucu SHA-1:** `823e2250d24b3ddac457a60c92a6a941943fcd6a`

## Çalıştırma

Sunucu `/tmp/voxel-vanilla-26.2-test` altında, `online-mode=false`, seed `776`,
view/simulation distance `5`, `gamemode=survival`, `force-gamemode=true`,
peaceful ve spawn protection `0` ayarlarıyla çalıştı.

Her oturumda player bağlandıktan sonra vanilla server console'undan şu iki
hazırlık komutu gönderildi:

```text
item replace entity <player> hotbar.0 with minecraft:stone 8
item replace entity <player> inventory.0 with minecraft:stone 3
```

Geçerli iki probe koşusu:

```text
cargo run --offline --release --example survival_inventory_double_click_state_correction_probe -- 127.0.0.1:25566 InvDouble2
cargo run --offline --release --example survival_inventory_double_click_state_correction_probe -- 127.0.0.1:25566 InvDouble3
```

Probe, player inventory menu state `1` iken hotbar slotu `36`'ya count `8`, main
inventory slotu `9`'a count `3` hazırlanmasını bekledi. Ardından state-id `0`,
boş changed-slot map'i ve empty carried hash'i ile pickup ve boş slot `10`
üzerinden `PICKUP_ALL` click'lerini gönderdi.

## Sonuçlar

| Gözlem | InvDouble2 | InvDouble3 |
| --- | ---: | ---: |
| play login / player loaded | `true / true` | `true / true` |
| container click packets sent | `2` | `2` |
| gönderilen state-id / slot-button-input dizisi | `0 / 36-0-0, 10-0-6` | `0 / 36-0-0, 10-0-6` |
| initial container content | `state=1, items=46, all empty, carried=empty` | `state=1, items=46, all empty, carried=empty` |
| item hazırlık slot observations | `state=2, slot36=count8,item1; state=3, slot9=count3,item1` | `state=2, slot36=count8,item1; state=3, slot9=count3,item1` |
| pickup full resync | `state=4, slot36=empty, slot9=count3,item1, carried=count8,item1` | `state=4, slot36=empty, slot9=count3,item1, carried=count8,item1` |
| double-click full resync | `state=5, slot9=empty, slot36=empty, carried=count11,item1` | `state=5, slot9=empty, slot36=empty, carried=count11,item1` |
| chunk packet / decoded / batch finished | `157 / 157 / 51` | `157 / 157 / 51` |
| received packet | `1833` | `1529` |

Her iki vanilla oturumunda pickup sonrası cursor'daki 8 taş, boş hedef slot
`10` üzerinden ileri taranan slot `9`'daki 3 taş ile birleşti. Eski state-id
`0` nedeniyle her click sonrası full content state'i arttı; son state `5`'te
slot `9` ve `36` boş, carried count `11` oldu. Server logunda bu koşular için
protocol error veya timeout gözlenmedi.

## Sonuç ve açık kapsam

İki bağımsız Mojang vanilla 26.2 oturumu, survival button `0` `PICKUP_ALL`
akışının boş hedef koşulunu, item stack birleşmesini ve stale state-id sonrası
slot/cursor full resync'ini doğrular. Sözleşme ayrıntıları [survival inventory
double-click state-id correction contract belgesindedir](survival-inventory-double-click-state-correction-contract.md).

Button `1` ile geriye tarama, cursor max stack sınırı, doğru state-id ile
client'ın hashed prediction'ı, yanlış state'in diğer varyantları ve özel menu
akışları açık kalır.
