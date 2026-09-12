# P13 — Vanilla survival inventory break sound component correct prediction doğrulaması

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
item replace entity <player> hotbar.0 with minecraft:stone[break_sound="minecraft:block.stone.break"]
```

Java registry-aware oracle, `BREAK_SOUND` type ID `81`, sound registry ID `1596`
ve component hash `0x9662f221` verdi. Geçerli iki probe koşusu:

```text
cargo run --offline --release --example survival_inventory_break_sound_component_correct_prediction_probe -- 127.0.0.1:25566 BreakSoundC26
cargo run --offline --release --example survival_inventory_break_sound_component_correct_prediction_probe -- 127.0.0.1:25566 BreakSoundD26
```

Her iki koşuda hazırlık paketi state `2`, slot `36`, item ID `1`, count `1` ve
positive component ID `81` taşıdı. Probe, carried prediction içinde type `81`,
`0x9662f221` hash'ini ve boş removed set'ini gönderdi.

## Sonuçlar

| Gözlem | BreakSoundC26 | BreakSoundD26 |
| --- | ---: | ---: |
| play login / player loaded | `true / true` | `true / true` |
| container click packets sent | `1` | `1` |
| gönderilen state-id / slot / button / input id | `2 / 36 / 0 / 0` | `2 / 36 / 0 / 0` |
| changed-slot prediction | `slot36=empty` | `slot36=empty` |
| carried prediction | `item1,count1,added81 hash=0x9662f221,removed0` | `item1,count1,added81 hash=0x9662f221,removed0` |
| initial container content | `state=1, items=46, all empty, carried=empty` | `state=1, items=46, all empty, carried=empty` |
| item hazırlık slot observation | `state=2, slot36=count1,item1,positive81` | `state=2, slot36=count1,item1,positive81` |
| click sonrası incremental slot update | `0` | `0` |
| click sonrası cursor update | `0` | `0` |
| additional full content observations | `0` | `0` |
| chunk packet / decoded / batch finished | `157 / 157 / 51` | `157 / 157 / 51` |
| received packet | `7685` | `6885` |

Her iki vanilla oturumunda doğru state-id ve doğru registry-aware break sound
component hash'iyle click uygulandı. Hazırlık state `2` slot gözlemi dışında slot
veya cursor düzeltmesi gelmedi; item hazırlığı dışındaki full content paket sayısı
`0` olarak kaldı. Server logunda geçerli koşular için protocol error veya timeout
gözlenmedi.

İlk `BreakSoundB26` koşusunda doğrudan HashOps ile üretilen Holder hash'i
`0x4d1698df` kullanıldığı için cursor correction geldi; registry-aware oracle ile
üretilen `0x9662f221` kullanılarak yukarıdaki iki temiz koşu tekrarlandı.

## Sonuç ve açık kapsam

İki bağımsız Mojang vanilla 26.2 oturumu, `minecraft:block.stone.break` holder
component hash'i carried `HashedStack` içinde doğru eşleştiğinde slot/cursor
correction paketlerini bastırdığını doğrular. Sözleşme ayrıntıları [survival
inventory break sound component correct prediction contract belgesindedir](survival-inventory-break-sound-component-correct-prediction-contract.md).

Diğer sound event'leri, direct holder varyantları, diğer component codec'leri,
yanlış veya kısmi component prediction, diğer click input'ları, değişen slot
haritasının varyantları ve özel menu akışları açık kalır.
