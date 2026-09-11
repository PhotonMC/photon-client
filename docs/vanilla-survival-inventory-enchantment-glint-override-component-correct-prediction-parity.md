# P13 — Vanilla survival inventory enchantment glint override component correct prediction doğrulaması

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
item replace entity <player> hotbar.0 with minecraft:stone[enchantment_glint_override=true]
```

Java oracle, `ENCHANTMENT_GLINT_OVERRIDE=true` component hash'ini
`0xc336cec2` verdi. Geçerli iki probe koşusu:

```text
cargo run --offline --release --example survival_inventory_enchantment_glint_override_component_correct_prediction_probe -- 127.0.0.1:25566 InvGlintC
cargo run --offline --release --example survival_inventory_enchantment_glint_override_component_correct_prediction_probe -- 127.0.0.1:25566 InvGlintD
```

Her iki koşuda hazırlık paketi state `2`, slot `36`, item ID `1`, count `1` ve
positive component ID `21` taşıdı. Probe, carried prediction içinde type `21`
ve `0xc336cec2` hash'ini, boş removed set'i gönderdi.

## Sonuçlar

| Gözlem | InvGlintC | InvGlintD |
| --- | ---: | ---: |
| play login / player loaded | `true / true` | `true / true` |
| container click packets sent | `1` | `1` |
| gönderilen state-id / slot / button / input id | `2 / 36 / 0 / 0` | `2 / 36 / 0 / 0` |
| changed-slot prediction | `slot36=empty` | `slot36=empty` |
| carried prediction | `item1,count1,added21 hash=0xc336cec2,removed0` | `item1,count1,added21 hash=0xc336cec2,removed0` |
| initial container content | `state=1, items=46, all empty, carried=empty` | `state=1, items=46, all empty, carried=empty` |
| item hazırlık slot observation | `state=2, slot36=count1,item1,positive21` | `state=2, slot36=count1,item1,positive21` |
| click sonrası incremental slot update | `0` | `0` |
| click sonrası cursor update | `0` | `0` |
| additional full content observations | `0` | `0` |
| chunk packet / decoded / batch finished | `157 / 157 / 51` | `157 / 157 / 51` |
| received packet | `2172` | `1694` |

Her iki vanilla oturumunda doğru state-id ve doğru boolean component hash'iyle
click uygulandı. Hazırlık state `2` slot gözlemi dışında slot veya cursor
düzeltmesi gelmedi; item hazırlığı dışındaki full content paket sayısı `0`
kaldı. Server logunda geçerli koşular için protocol error veya timeout
gözlenmedi.

## Sonuç ve açık kapsam

İki bağımsız Mojang vanilla 26.2 oturumu, `ENCHANTMENT_GLINT_OVERRIDE`
component hash'i carried `HashedStack` içinde doğru eşleştiğinde slot/cursor
correction paketlerini bastırdığını doğrular. Sözleşme ayrıntıları [survival
inventory enchantment glint override component correct prediction contract
belgesindedir](survival-inventory-enchantment-glint-override-component-correct-prediction-contract.md).

`false` değeri, diğer component codec'leri, yanlış veya kısmi component
prediction, diğer click input'ları, değişen slot haritasının varyantları ve
özel menu akışları açık kalır.
