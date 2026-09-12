# P13 — Vanilla survival inventory item model component correct prediction doğrulaması

**Tarih:** 2026-09-12

**Referans:** Java Edition 26.2 / protokol 776 / Java 25

**Sunucu:** Mojang vanilla 26.2 dedicated server, offline yerel koşu

**Sunucu SHA-1:** `823e2250d24b3ddac457a60c92a6a941943fcd6a`

## Çalıştırma

Sunucu `/tmp/voxel-vanilla-26.2-test` altında, `online-mode=false`, seed `776`,
view/simulation distance `5`, `gamemode=survival`, `force-gamemode=true`,
peaceful ve spawn protection `0` ayarlarıyla çalıştı.

Her geçerli oturumda player bağlandıktan sonra vanilla server console'undan şu
hazırlık komutu gönderildi:

```text
item replace entity <player> hotbar.0 with minecraft:stone[item_model="minecraft:dirt"]
```

Çıplak `item_model=minecraft:dirt` biçimi vanilla tarafından reddedildi; geçerli
koşular tırnaklı değerle tekrarlandı. Java oracle, `ITEM_MODEL=minecraft:dirt`
component hash'ini `0x75c93192` verdi. Geçerli iki probe koşusu:

```text
cargo run --offline --release --example survival_inventory_item_model_component_correct_prediction_probe -- 127.0.0.1:25566 InvItemModelB
cargo run --offline --release --example survival_inventory_item_model_component_correct_prediction_probe -- 127.0.0.1:25566 InvItemModelC
```

Her iki koşuda hazırlık paketi state `2`, slot `36`, item ID `1`, count `1` ve
positive component ID `10` taşıdı. Probe, carried prediction içinde type `10`
ve `0x75c93192` hash'ini, boş removed set'i gönderdi.

## Sonuçlar

| Gözlem | InvItemModelB | InvItemModelC |
| --- | ---: | ---: |
| play login / player loaded | `true / true` | `true / true` |
| container click packets sent | `1` | `1` |
| gönderilen state-id / slot / button / input id | `2 / 36 / 0 / 0` | `2 / 36 / 0 / 0` |
| changed-slot prediction | `slot36=empty` | `slot36=empty` |
| carried prediction | `item1,count1,added10 hash=0x75c93192,removed0` | `item1,count1,added10 hash=0x75c93192,removed0` |
| initial container content | `state=1, items=46, all empty, carried=empty` | `state=1, items=46, all empty, carried=empty` |
| item hazırlık slot observation | `state=2, slot36=count1,item1,positive10` | `state=2, slot36=count1,item1,positive10` |
| click sonrası incremental slot update | `0` | `0` |
| click sonrası cursor update | `0` | `0` |
| additional full content observations | `0` | `0` |
| chunk packet / decoded / batch finished | `157 / 157 / 51` | `157 / 157 / 51` |
| received packet | `8373` | `6395` |

Her iki vanilla oturumunda doğru state-id ve doğru item model component hash'iyle
click uygulandı. Hazırlık state `2` slot gözlemi dışında slot veya cursor
düzeltmesi gelmedi; item hazırlığı dışındaki full content paket sayısı `0`
kaldı. Server logunda geçerli koşular için protocol error veya timeout
gözlenmedi.

## Sonuç ve açık kapsam

İki bağımsız Mojang vanilla 26.2 oturumu, `ITEM_MODEL=minecraft:dirt`
component hash'i carried `HashedStack` içinde doğru eşleştiğinde slot/cursor
correction paketlerini bastırdığını doğrular. Sözleşme ayrıntıları [survival
inventory item model component correct prediction contract belgesindedir](survival-inventory-item-model-component-correct-prediction-contract.md).

Diğer identifier değerleri, diğer component codec'leri, yanlış veya kısmi
component prediction, diğer click input'ları, değişen slot haritasının
varyantları ve özel menu akışları açık kalır.
