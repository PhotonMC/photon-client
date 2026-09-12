# P13 — Vanilla survival inventory sheep color component correct prediction doğrulaması

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
item replace entity <player> hotbar.0 with minecraft:stone[sheep/color=red]
```

Java oracle, `SHEEP_COLOR` type ID `109`, `DyeColor.RED` color ID `14` ve
component hash `0x8c6452aa` verdi. Geçerli iki probe koşusu:

```text
cargo run --offline --release --example survival_inventory_sheep_color_component_correct_prediction_probe -- 127.0.0.1:25566 SheepColorC26
cargo run --offline --release --example survival_inventory_sheep_color_component_correct_prediction_probe -- 127.0.0.1:25566 SheepColorD26
```

Probe, preparation slotunda type `109` gözlemini bulduktan sonra o gözlemin
state-id değerini click packet'ine taşıdı. Carried prediction type `109`, hash
`0x8c6452aa`, removed set boş olarak gönderildi.

## Sonuçlar

| Gözlem | SheepColorC26 | SheepColorD26 |
| --- | ---: | ---: |
| play login / player loaded | `true / true` | `true / true` |
| container click packets sent | `1` | `1` |
| gönderilen state-id / slot / button / input id | `2 / 36 / 0 / 0` | `2 / 36 / 0 / 0` |
| changed-slot prediction | `slot36=empty` | `slot36=empty` |
| carried prediction | `item1,count1,added109 hash=0x8c6452aa,removed0` | `item1,count1,added109 hash=0x8c6452aa,removed0` |
| initial container content | `state=1, items=46, all empty, carried=empty` | `state=1, items=46, all empty, carried=empty` |
| item preparation slot observation | `state=2, slot36=count1,item1,positive109` | `state=2, slot36=count1,item1,positive109` |
| click sonrası incremental slot update | `0` | `0` |
| click sonrası cursor update | `0` | `0` |
| additional full content observations | `0` | `0` |
| chunk packet / decoded / batch finished | `157 / 157 / 51` | `157 / 157 / 51` |
| received packet | `11165` | `8483` |

Her iki vanilla oturumunda preparation slotunda görülen state `2` ve doğru sheep
color component hash'iyle click uygulandı. Slot veya cursor düzeltmesi gelmedi;
item preparation dışındaki full content paket sayısı `0` olarak kaldı. Geçerli
koşuların server logunda protocol error veya timeout gözlenmedi.

## Sonuç ve açık kapsam

İki bağımsız Mojang vanilla 26.2 oturumu, `sheep/color=red` component hash'i
carried `HashedStack` içinde doğru eşleştiğinde slot/cursor correction paketlerini
bastırdığını doğrular. Sözleşme ayrıntıları [survival inventory sheep color
component correct prediction contract belgesindedir](survival-inventory-sheep-color-component-correct-prediction-contract.md).

Diğer entity component'leri, diğer dye renkleri, diğer component codec'leri,
yanlış veya kısmi component prediction, diğer click input'ları, değişen slot
haritasının varyantları ve özel menu akışları açık kalır.
