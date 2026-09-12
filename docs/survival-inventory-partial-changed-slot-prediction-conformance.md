# P13 — Survival inventory partial changed-slot prediction sözleşmesi — [server-conformance] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, inventory pickup packet'inde değişen slotun `HashedStack`
haritasından tamamen çıkarıldığı durumda vanilla server'ın slotu düzeltmesini,
doğru carried prediction'ı ise korumasını sınar.

## Java referansı

`ServerGamePacketListenerImpl.handleContainerClick`, state-id eşleştiğinde
packet içindeki changed-slot map girdilerini tek tek remote slot beklentisi
olarak kaydeder. Haritada bir slot yoksa o slot için yeni bir remote beklenti
alınmaz; önceki beklenti korunur.

`AbstractContainerMenu.broadcastChanges`, pickup sonrası gerçek slot durumunu
korunan remote beklentiyle karşılaştırır. Kaynak slot gerçek durumda empty
olduğu için slot `36` için state-id taşıyan `container_set_slot` düzeltmesi
gönderilir. Carried hash'i gerçek stone stack ile eşleştiği için cursor
düzeltmesi gönderilmez. State-id doğru olduğundan full resync yolu çalışmaz.

## Probe girdisi ve akışı

- Oyun modu: `SURVIVAL`
- Container: id `0`, player inventory menu
- Hazırlık: hotbar index `0` / menu slotu `36` içine `minecraft:stone 8`
- Stone item registry ID: `1`
- Menu state-id: `2` (initial content state `1` sonrasındaki hazırlık state'i)
- Click: slot `36`, button `0`, `PICKUP` input id `0`
- Changed-slot prediction: boş map; slot `36` için hash girdisi yok
- Carried prediction: item ID `1`, count `8`, boş added/removed component
  map/set içeren non-empty hash

Probe, item hazırlığı sonrası tek `container_click` paketi yollar; ardından
slot, cursor ve full content güncellemelerini toplar.

Beklenen invariant'lar şunlardır:

- Başlangıç content'i container `0`, state `1`, 46 slot ve carried için boş
  değer taşır.
- Console item hazırlığı slot `36` için state `2`, item ID `1` ve count `8`
  gözlemi üretir.
- Server click'i state-id mismatch saymaz ve pickup davranışını uygular.
- Changed-slot map'i slot `36` içermediği için server'ın önceki remote slot
  beklentisi korunur ve gerçek empty slot için state `3` correction gelir.
- Carried prediction doğru olduğu için `set_cursor_item` correction gelmez.
- Item hazırlığı dışındaki `container_set_content` gözlemi oluşmaz; full resync
  yolu çalışmaz.

## Sınır

Bu dilim değişen slot hash girdisinin eksik olmasını ve doğru carried hash'iyle
bağımsız slot/cursor düzeltmesini kapsar. Birden fazla eksik slot, yanlış veya
kısmi carried component prediction, diğer click input'ları ve özel container'lar
sonraki P13 dilimleridir.


## Vanilla server-conformance doğrulaması

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
cargo run --offline --release --example survival_inventory_partial_changed_slot_prediction_probe -- 127.0.0.1:25566 InvPart2
cargo run --offline --release --example survival_inventory_partial_changed_slot_prediction_probe -- 127.0.0.1:25566 InvPart3
```

Probe state `2`, slot `36` için changed-slot hash map'ini boş gönderdi; carried
prediction ise item ID `1`, count `8` ve boş component map/set içeren doğru
non-empty hash olarak gönderildi.

## Sonuçlar

| Gözlem | InvPart2 | InvPart3 |
| --- | ---: | ---: |
| play login / player loaded | `true / true` | `true / true` |
| container click packets sent | `1` | `1` |
| gönderilen state-id / slot / button / input id | `2 / 36 / 0 / 0` | `2 / 36 / 0 / 0` |
| changed-slot prediction | `boş map` | `boş map` |
| carried prediction | `item1,count8,empty components` | `item1,count8,empty components` |
| initial container content | `state=1, items=46, all empty, carried=empty` | `state=1, items=46, all empty, carried=empty` |
| item hazırlık slot observation | `state=2, slot36=count8,item1` | `state=2, slot36=count8,item1` |
| click sonrası slot düzeltmesi | `state=3, slot36=empty` | `state=3, slot36=empty` |
| click sonrası cursor update | `0` | `0` |
| additional full content observations | `0` | `0` |
| chunk packet / decoded / batch finished | `157 / 157 / 51` | `157 / 157 / 51` |
| received packet | `1424` | `1442` |

Her iki vanilla oturumunda state-id doğru olduğu için pickup uygulandı. Changed
slot map'inde slot `36` bulunmadığından önceki remote beklenti korundu ve gerçek
empty slot için yalnızca state `3` `container_set_slot` düzeltmesi geldi. Doğru
carried hash'i cursor düzeltmesini bastırdı; item hazırlığı dışındaki full
content paket sayısı `0` kaldı. Server logunda geçerli koşular için protocol
error veya timeout gözlenmedi.

## Sonuç ve açık kapsam

İki bağımsız Mojang vanilla 26.2 oturumu, changed-slot hash girdisi eksik olsa
da carried prediction'ın bağımsız değerlendirildiğini ve yalnızca slot
düzeltmesi üretildiğini doğrular. Sözleşme ayrıntıları [survival inventory
partial changed-slot prediction contract belgesindedir](#sözleşme).

Birden fazla eksik slot, yanlış veya kısmi carried component prediction, diğer
click input'ları ve özel container akışları açık kalır.
