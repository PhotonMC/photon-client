# P13 — Vanilla survival inventory use effects component correct prediction doğrulaması

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
item replace entity <player> hotbar.0 with minecraft:stone[use_effects={can_sprint:true,interact_vibrations:false,speed_multiplier:1.0}]
```

Java oracle, `UseEffects(true, false, 1.0F)` component hash'ini `0x9619e224`
verdi. Geçerli iki probe koşusu:

```text
cargo run --offline --release --example survival_inventory_use_effects_component_correct_prediction_probe -- 127.0.0.1:25566 InvUseEffectsB
cargo run --offline --release --example survival_inventory_use_effects_component_correct_prediction_probe -- 127.0.0.1:25566 InvUseEffectsC
```

Her iki koşuda hazırlık paketi state `2`, slot `36`, item ID `1`, count `1` ve
positive component ID `5` taşıdı. Probe, carried prediction içinde type `5`
ve `0x9619e224` hash'ini, boş removed set'i gönderdi.

## Sonuçlar

| Gözlem | InvUseEffectsB | InvUseEffectsC |
| --- | ---: | ---: |
| play login / player loaded | `true / true` | `true / true` |
| container click packets sent | `1` | `1` |
| gönderilen state-id / slot / button / input id | `2 / 36 / 0 / 0` | `2 / 36 / 0 / 0` |
| changed-slot prediction | `slot36=empty` | `slot36=empty` |
| carried prediction | `item1,count1,added5 hash=0x9619e224,removed0` | `item1,count1,added5 hash=0x9619e224,removed0` |
| initial container content | `state=1, items=46, all empty, carried=empty` | `state=1, items=46, all empty, carried=empty` |
| item hazırlık slot observation | `state=2, slot36=count1,item1,positive5` | `state=2, slot36=count1,item1,positive5` |
| click sonrası incremental slot update | `0` | `0` |
| click sonrası cursor update | `0` | `0` |
| additional full content observations | `0` | `0` |
| chunk packet / decoded / batch finished | `157 / 157 / 51` | `157 / 157 / 51` |
| received packet | `8233` | `8351` |

Her iki vanilla oturumunda doğru state-id ve doğru use effects component
hash'iyle click uygulandı. Hazırlık state `2` slot gözlemi dışında slot veya
cursor düzeltmesi gelmedi; item hazırlığı dışındaki full content paket sayısı
`0` kaldı. Server logunda geçerli koşular için protocol error veya timeout
gözlenmedi.

## Sonuç ve açık kapsam

İki bağımsız Mojang vanilla 26.2 oturumu, `UseEffects(true, false, 1.0F)`
component hash'i carried `HashedStack` içinde doğru eşleştiğinde slot/cursor
correction paketlerini bastırdığını doğrular. Sözleşme ayrıntıları [survival
inventory use effects component correct prediction contract belgesindedir](survival-inventory-use-effects-component-correct-prediction-contract.md).

Diğer alan değerleri, diğer component codec'leri, yanlış veya kısmi component
prediction, diğer click input'ları, değişen slot haritasının varyantları ve özel
menu akışları açık kalır.
