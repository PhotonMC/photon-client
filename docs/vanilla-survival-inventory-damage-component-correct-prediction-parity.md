# P13 — Vanilla survival inventory damage component correct prediction doğrulaması

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
item replace entity <player> hotbar.0 with minecraft:stone[damage=1]
```

Java oracle, `DataComponents.DAMAGE` değeri `1` için component hash'ini
`0x5d50d71c` verdi. Geçerli iki probe koşusu:

```text
cargo run --offline --release --example survival_inventory_damage_component_correct_prediction_probe -- 127.0.0.1:25566 InvDamage2
cargo run --offline --release --example survival_inventory_damage_component_correct_prediction_probe -- 127.0.0.1:25566 InvDamage3
```

Her iki probe state `2`, slot `36` için pickup sonrası empty hash'i; carried
için item ID `1`, count `1`, type `3` added hash'i `0x5d50d71c` ve boş removed
set'i taşıyan doğru damage component patch'ini gönderdi.

## Sonuçlar

| Gözlem | InvDamage2 | InvDamage3 |
| --- | ---: | ---: |
| play login / player loaded | `true / true` | `true / true` |
| container click packets sent | `1` | `1` |
| gönderilen state-id / slot / button / input id | `2 / 36 / 0 / 0` | `2 / 36 / 0 / 0` |
| changed-slot prediction | `slot36=empty` | `slot36=empty` |
| carried prediction | `item1,count1,added3 hash=0x5d50d71c,removed0` | `item1,count1,added3 hash=0x5d50d71c,removed0` |
| initial container content | `state=1, items=46, all empty, carried=empty` | `state=1, items=46, all empty, carried=empty` |
| item hazırlık slot observation | `state=2, slot36=count1,item1,positive3` | `state=2, slot36=count1,item1,positive3` |
| click sonrası incremental slot update | `0` | `0` |
| click sonrası cursor update | `0` | `0` |
| additional full content observations | `0` | `0` |
| chunk packet / decoded / batch finished | `157 / 157 / 51` | `157 / 157 / 51` |
| received packet | `3136` | `1239` |

Her iki geçerli vanilla oturumunda doğru state-id ve doğru damage component
hash'iyle click uygulandı. Hazırlık state `2` slot gözlemi dışında slot veya
cursor düzeltmesi gelmedi; item hazırlığı dışındaki full content paket sayısı
`0` kaldı. Server logunda geçerli koşular için protocol error veya timeout
gözlenmedi.

## Sonuç ve açık kapsam

İki bağımsız Mojang vanilla 26.2 oturumu, integer `DAMAGE` component hash'inin
carried `HashedStack` içinde doğru eşleştiğinde slot/cursor correction
paketlerini bastırdığını doğrular. Sözleşme ayrıntıları [survival inventory
damage component correct prediction contract belgesindedir](survival-inventory-damage-component-correct-prediction-contract.md).

Diğer component codec'leri, yanlış veya kısmi component prediction, diğer
click input'ları, değişen slot haritasının kısmi varyantları ve özel menu
akışları açık kalır.
