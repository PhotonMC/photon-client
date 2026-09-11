# P13 — Vanilla survival inventory max damage component correct prediction doğrulaması

**Tarih:** 2026-09-11

**Referans:** Java Edition 26.2 / protokol 776 / Java 25

**Sunucu:** Mojang vanilla 26.2 dedicated server, offline yerel koşu

**Sunucu SHA-1:** `823e2250d24b3ddac457a60c92a6a941943fcd6a`

## Çalıştırma

Sunucu `/tmp/voxel-vanilla-26.2-test` altında, `online-mode=false`, seed `776`,
view/simulation distance `5`, `gamemode=survival`, `force-gamemode=true`,
peaceful ve spawn protection `0` ayarlarıyla çalıştı.

Vanilla, yalnız `max_damage` taşıyan stone stack'ini reddettiği için her geçerli
oturumda player bağlandıktan sonra şu hazırlık komutu kullanıldı:

```text
item replace entity <player> hotbar.0 with minecraft:stone[max_stack_size=1,max_damage=1]
```

Geçerli iki probe koşusu:

```text
cargo run --offline --release --example survival_inventory_max_damage_component_correct_prediction_probe -- 127.0.0.1:25566 InvMaxDamage2
cargo run --offline --release --example survival_inventory_max_damage_component_correct_prediction_probe -- 127.0.0.1:25566 InvMaxDamage3
```

Her iki koşuda hazırlık paketi state `2`, slot `36`, item ID `1`, count `1` ve
positive component ID'leri `[1, 2]` taşıdı. Probe, carried prediction içinde
type `1` ve `2` için `0x5d50d71c` hash'lerini ve boş removed set'i gönderdi.

## Sonuçlar

| Gözlem | InvMaxDamage2 | InvMaxDamage3 |
| --- | ---: | ---: |
| play login / player loaded | `true / true` | `true / true` |
| container click packets sent | `1` | `1` |
| gönderilen state-id / slot / button / input id | `2 / 36 / 0 / 0` | `2 / 36 / 0 / 0` |
| changed-slot prediction | `slot36=empty` | `slot36=empty` |
| carried prediction | `item1,count1,added1+2 hash=0x5d50d71c,removed0` | `item1,count1,added1+2 hash=0x5d50d71c,removed0` |
| initial container content | `state=1, items=46, all empty, carried=empty` | `state=1, items=46, all empty, carried=empty` |
| item hazırlık slot observation | `state=2, slot36=count1,item1,positive1+2` | `state=2, slot36=count1,item1,positive1+2` |
| click sonrası incremental slot update | `0` | `0` |
| click sonrası cursor update | `0` | `0` |
| additional full content observations | `0` | `0` |
| chunk packet / decoded / batch finished | `157 / 157 / 51` | `157 / 157 / 51` |
| received packet | `2130` | `1623` |

Her iki geçerli vanilla oturumunda doğru state-id ve iki doğru component hash'i
ile click uygulandı. Hazırlık state `2` slot gözlemi dışında slot veya cursor
düzeltmesi gelmedi; item hazırlığı dışındaki full content paket sayısı `0`
kaldı. Server logunda geçerli koşular için protocol error veya timeout
gözlenmedi.

## Sonuç ve açık kapsam

İki bağımsız Mojang vanilla 26.2 oturumu, `MAX_DAMAGE` ve onunla birlikte
gereken `MAX_STACK_SIZE` component hash'leri carried `HashedStack` içinde doğru
eşleştiğinde slot/cursor correction paketlerini bastırdığını doğrular. Sözleşme
ayrıntıları [survival inventory max damage component correct prediction contract
belgesindedir](survival-inventory-max-damage-component-correct-prediction-contract.md).

Diğer max damage değerleri, diğer component codec'leri, yanlış veya kısmi
component prediction, diğer click input'ları, değişen slot haritasının
varyantları ve özel menu akışları açık kalır.
