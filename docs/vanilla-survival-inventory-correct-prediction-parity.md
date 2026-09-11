# P13 — Vanilla survival inventory correct prediction doğrulaması

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
item replace entity <player> hotbar.0 with minecraft:stone 8
```

Geçerli iki probe koşusu:

```text
cargo run --offline --release --example survival_inventory_correct_prediction_probe -- 127.0.0.1:25566 InvPredict3
cargo run --offline --release --example survival_inventory_correct_prediction_probe -- 127.0.0.1:25566 InvPredict4
```

İlk `InvPredict2` denemesi, non-empty `HashedStack` optional presence byte'ı
encoder'da eksik olduğu için vanilla decoder tarafından reddedildi ve parity
sonucu olarak kullanılmadı. Düzeltilen probe state `2`, slot `36`, stone count
`8` changed-slot hash'i ve empty carried hash'i gönderdi.

## Sonuçlar

| Gözlem | InvPredict3 | InvPredict4 |
| --- | ---: | ---: |
| play login / player loaded | `true / true` | `true / true` |
| container click packets sent | `1` | `1` |
| gönderilen state-id / slot / button / input id | `2 / 36 / 0 / 0` | `2 / 36 / 0 / 0` |
| changed-slot prediction | `slot36=item1,count8,empty components` | `slot36=item1,count8,empty components` |
| initial container content | `state=1, items=46, all empty, carried=empty` | `state=1, items=46, all empty, carried=empty` |
| item hazırlık slot observation | `state=2, slot36=count8,item1` | `state=2, slot36=count8,item1` |
| incremental slot update | `state=3, slot36=empty` | `state=3, slot36=empty` |
| cursor update | `set_cursor_item, count8,item1, empty components` | `set_cursor_item, count8,item1, empty components` |
| additional full content observations | `0` | `0` |
| chunk packet / decoded / batch finished | `157 / 157 / 51` | `157 / 157 / 51` |
| received packet | `1059` | `3515` |

Her iki geçerli vanilla oturumunda doğru prediction ile stale full resync yolu
çalışmadı. Slot `36` state `3` ile boşaltıldı, cursor count `8` ayrı
`set_cursor_item` paketiyle bildirildi ve item hazırlığı dışındaki full content
paket sayısı `0` kaldı. Server logunda geçerli koşular için protocol error veya
timeout gözlenmedi.

## Sonuç ve açık kapsam

İki bağımsız Mojang vanilla 26.2 oturumu, doğru state-id ve component'siz
`HashedStack` prediction'ının pickup sonrası incremental slot/cursor yayınını
doğrular. Sözleşme ayrıntıları [survival inventory correct prediction contract
belgesindedir](survival-inventory-correct-prediction-contract.md).

Component hash değerleri, yanlış veya kısmi prediction, diğer click input'ları,
stale state varyantları ve özel menu akışları açık kalır.
