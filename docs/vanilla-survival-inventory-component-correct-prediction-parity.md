# P13 — Vanilla survival inventory component correct prediction doğrulaması

**Tarih:** 2026-09-11

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
item replace entity <player> hotbar.0 with minecraft:milk_bucket[minecraft:custom_model_data={floats:[1.0f]}]
```

Java oracle, bu `custom_model_data` değeri için component hash'ini
`0x77a4a24f` verdi. Geçerli iki probe koşusu:

```text
cargo run --offline --release --example survival_inventory_component_correct_prediction_probe -- 127.0.0.1:25566 InvCompOk3
cargo run --offline --release --example survival_inventory_component_correct_prediction_probe -- 127.0.0.1:25566 InvCompOk4
```

Probe state `2`, slot `36` için pickup sonrası empty hash'i; carried için item
ID `1046`, count `1`, type `17` added hash'i `0x77a4a24f` ve boş removed set'i
taşıyan doğru component patch'ini gönderdi.

İlk `InvCompOk2` keşif koşusunda hazırlık dışında beklenmeyen bir stone slot
paketi state `2` ile gözlendi; click state'i bu nedenle server state'iyle
eşleşmedi ve full resync oluştu. Başlangıç/preparation state sırası bozulduğu
için bu koşu parity kanıtına dahil edilmedi.

## Sonuçlar

| Gözlem | InvCompOk3 | InvCompOk4 |
| --- | ---: | ---: |
| play login / player loaded | `true / true` | `true / true` |
| container click packets sent | `1` | `1` |
| gönderilen state-id / slot / button / input id | `2 / 36 / 0 / 0` | `2 / 36 / 0 / 0` |
| changed-slot prediction | `slot36=empty` | `slot36=empty` |
| carried prediction | `item1046,count1,added17 hash=0x77a4a24f,removed0` | `item1046,count1,added17 hash=0x77a4a24f,removed0` |
| initial container content | `state=1, items=46, all empty, carried=empty` | `state=1, items=46, all empty, carried=empty` |
| item hazırlık slot observation | `state=2, slot36=count1,item1046,positive17` | `state=2, slot36=count1,item1046,positive17` |
| click sonrası incremental slot update | `0` | `0` |
| click sonrası cursor update | `0` | `0` |
| additional full content observations | `0` | `0` |
| chunk packet / decoded / batch finished | `157 / 157 / 51` | `157 / 157 / 51` |
| received packet | `1778` | `1969` |

Her iki geçerli vanilla oturumunda doğru state-id ve doğru component hash'iyle
click uygulandı. Hazırlık state `2` slot gözlemi dışında slot veya cursor
düzeltmesi gelmedi; item hazırlığı dışındaki full content paket sayısı `0`
kaldı. Server logunda geçerli koşular için protocol error veya timeout
gözlenmedi.

## Sonuç ve açık kapsam

İki bağımsız Mojang vanilla 26.2 oturumu, `custom_model_data` component hash'inin
carried `HashedStack` içinde doğru eşleştiğinde slot/cursor correction paketlerini
bastırdığını doğrular. Sözleşme ayrıntıları [survival inventory component correct
prediction contract belgesindedir](survival-inventory-component-correct-prediction-contract.md).

Diğer component codec'leri, yanlış veya kısmi component prediction, diğer click
input'ları, değişen slot haritasının kısmi varyantları ve özel menu akışları açık
kalır.
