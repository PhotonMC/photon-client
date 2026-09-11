# P13 — Vanilla survival inventory ahead component correct prediction doğrulaması

**Tarih:** 2026-09-11

**Referans:** Java Edition 26.2 / protokol 776 / Java 25

**Sunucu:** Mojang vanilla 26.2 dedicated server, offline yerel koşu

**Sunucu SHA-1:** `823e2250d24b3ddac457a60c92a6a941943fcd6a`

## Çalıştırma

Sunucu `/tmp/voxel-vanilla-26.2-test` altında, `online-mode=false`, seed `776`,
view/simulation distance `5`, `gamemode=survival`, `force-gamemode=true`,
peaceful ve spawn protection `0` ayarlarıyla çalıştı.

Her oturumda player bağlandıktan sonra vanilla server console'undan şu hazırlık
komutu gönderildi:

```text
item replace entity <player> hotbar.0 with minecraft:milk_bucket[minecraft:custom_model_data={floats:[1.0f]}]
```

Java oracle, bu component değeri için hash'i `0x77a4a24f` verdi. Geçerli iki
probe koşusu:

```text
cargo run --offline --release --example survival_inventory_ahead_component_correct_prediction_probe -- 127.0.0.1:25566 InvAheadComp2
cargo run --offline --release --example survival_inventory_ahead_component_correct_prediction_probe -- 127.0.0.1:25566 InvAheadComp3
```

Probe state `99` gönderdi. Server hazırlık komutundan sonra menu state-id `2`
olduğu için bu packet ahead idi; changed-slot ve carried component hash'leri
pickup sonrası gerçek durumla eşleşecek şekilde gönderildi.

## Sonuçlar

| Gözlem | InvAheadComp2 | InvAheadComp3 |
| --- | ---: | ---: |
| play login / player loaded | `true / true` | `true / true` |
| container click packets sent | `1` | `1` |
| gönderilen state-id / slot / button / input id | `99 / 36 / 0 / 0` | `99 / 36 / 0 / 0` |
| changed-slot prediction | `slot36=empty` | `slot36=empty` |
| carried prediction | `item1046,count1,added17 hash=0x77a4a24f,removed0` | `item1046,count1,added17 hash=0x77a4a24f,removed0` |
| initial container content | `state=1, items=46, all empty, carried=empty` | `state=1, items=46, all empty, carried=empty` |
| item hazırlık slot observation | `state=2, slot36=count1,item1046,positive17` | `state=2, slot36=count1,item1046,positive17` |
| click sonrası full content | `state=3, slot36=empty, carried=count1,item1046,positive17` | `state=3, slot36=empty, carried=count1,item1046,positive17` |
| click sonrası incremental slot update | `0` | `0` |
| click sonrası cursor update | `0` | `0` |
| additional full content observations | `1` | `1` |
| chunk packet / decoded / batch finished | `157 / 157 / 51` | `157 / 157 / 51` |
| received packet | `1372` | `3463` |

Her iki vanilla oturumunda component hash'i doğru olmasına rağmen ahead state-id
mismatch full resync'i seçti. Server state `3` ile component'li milk bucket'ı
carried içerikte gönderdi; artımlı slot veya cursor düzeltmesi gelmedi. Server
logunda geçerli koşular için protocol error veya timeout gözlenmedi.

## Sonuç ve açık kapsam

İki bağımsız Mojang vanilla 26.2 oturumu, ahead state-id kontrolünün component
hash eşleşmesinden önce full resync kararı verdiğini doğrular. Sözleşme ayrıntıları
[survival inventory ahead component correct prediction contract belgesindedir](survival-inventory-ahead-component-correct-prediction-contract.md).

Diğer component codec'leri, yanlış component hash'leri, diğer click input'ları ve
özel container akışları açık kalır.
