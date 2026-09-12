# P13 — Vanilla survival inventory cat collar component correct prediction doğrulaması

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
item replace entity <player> hotbar.0 with minecraft:stone[cat/collar=red]
```

Java oracle, `CAT_COLLAR` type ID `108`, `DyeColor.RED` color ID `14` ve
component hash `0x8c6452aa` verdi. Geçerli iki probe koşusu:

```text
cargo run --offline --release --example survival_inventory_cat_collar_component_correct_prediction_probe -- 127.0.0.1:25566 CatCollarC26
cargo run --offline --release --example survival_inventory_cat_collar_component_correct_prediction_probe -- 127.0.0.1:25566 CatCollarD26
```

Probe, preparation slotunda type `108` gözlemini bulduktan sonra o gözlemin
state-id değerini click packet'ine taşıdı. Carried prediction type `108`, hash
`0x8c6452aa`, removed set boş olarak gönderildi.

## Sonuçlar

| Gözlem | CatCollarC26 | CatCollarD26 |
| --- | ---: | ---: |
| play login / player loaded | `true / true` | `true / true` |
| container click packets sent | `1` | `1` |
| gönderilen state-id / slot / button / input id | `2 / 36 / 0 / 0` | `3 / 36 / 0 / 0` |
| changed-slot prediction | `slot36=empty` | `slot36=empty` |
| carried prediction | `item1,count1,added108 hash=0x8c6452aa,removed0` | `item1,count1,added108 hash=0x8c6452aa,removed0` |
| initial container content | `state=1, items=46, all empty, carried=empty` | `state=1, items=46, all empty, carried=empty` |
| item preparation slot observation | `state=2, slot36=count1,item1,positive108` | `state=2, slot36=count1,item1,positive85; state=3,count1,item1,positive108` |
| click sonrası incremental slot update | `0` | `0` |
| click sonrası cursor update | `0` | `0` |
| additional full content observations | `0` | `0` |
| chunk packet / decoded / batch finished | `157 / 157 / 51` | `157 / 157 / 51` |
| received packet | `9080` | `11002` |

Her iki vanilla oturumunda preparation slotunda görülen son state-id ve doğru
cat collar component hash'iyle click uygulandı. `CatCollarD26` koşusunda command
öncesi eski `wolf/collar` state `2` görüldü; command sonrası type `108` state `3`
gözlemi geldi ve probe state `3`'ü kullandı. Slot veya cursor düzeltmesi gelmedi;
item preparation dışındaki full content paket sayısı `0` olarak kaldı. Geçerli
koşuların server logunda protocol error veya timeout gözlenmedi.

## Sonuç ve açık kapsam

İki bağımsız Mojang vanilla 26.2 oturumu, `cat/collar=red` component hash'i
carried `HashedStack` içinde doğru eşleştiğinde slot/cursor correction paketlerini
bastırdığını doğrular. Sözleşme ayrıntıları [survival inventory cat collar
component correct prediction contract belgesindedir](survival-inventory-cat-collar-component-correct-prediction-contract.md).

Diğer entity component'leri, diğer dye renkleri, diğer component codec'leri,
yanlış veya kısmi component prediction, diğer click input'ları, değişen slot
haritasının varyantları ve özel menu akışları açık kalır.
