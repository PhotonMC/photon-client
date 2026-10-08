# P13 — Survival inventory wrong changed-slot prediction sözleşmesi — [server-conformance] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, client'ın doğru menu state-id'siyle gönderdiği inventory click
packet'inde değişen slot için yanlış post-click hash, carried için doğru
post-click hash taşıdığı durumda vanilla server'ın yalnızca slot düzeltmesi
göndermesini sınar.

## Java referansı

`MultiPlayerGameMode.handleContainerInput`, click'i client menüsünde
uyguladıktan sonra önceki ve sonraki slotları karşılaştırır; değişen slot için
sonraki değerin `HashedStack` hash'ini, carried için de click sonrası değerin
hash'ini packet'e koyar. Bu probe slot hash'ini bilerek click öncesi stone stack'i
olarak gönderir.

`ServerGamePacketListenerImpl.handleContainerClick`, eşleşen state-id ile
gelen changed-slot ve carried hash'lerini remote beklentiye kaydeder, click'i
uygular ve `broadcastChanges` çağırır. Slotun remote beklentisi stone stack'i
olarak kaldığı için gerçek click sonrası empty slot ile eşleşmez; carried
beklentisi gerçek stone stack'iyle eşleştiği için cursor düzeltmesi gerekmez.
`AbstractContainerMenu` bu durumda slot için state-id artıran
`ClientboundContainerSetSlotPacket` gönderir.

`HashedStack` non-empty değer için optional presence byte'ı, item holder'ı,
count'u ve `HashedPatchMap` component patch'ini taşır. Component'siz stone
stack'inde added component map'i ve removed component set'i boştur.

## Probe girdisi ve akışı

- Oyun modu: `SURVIVAL`
- Container: id `0`, player inventory menu
- Hazırlık: hotbar index `0` / menu slotu `36` içine `minecraft:stone 8`
- Stone item registry ID: `1`
- Menu state-id: `2` (initial content state `1` sonrasındaki hazırlık state'i)
- Click: slot `36`, button `0`, `PICKUP` input id `0`
- Changed-slot prediction: slot `36` için click öncesi item id `1`, count `8`,
  boş component map/set içeren non-empty hash
- Carried prediction: click sonrası item id `1`, count `8`, boş component
  map/set içeren non-empty hash

Probe, item hazırlığı sonrası tek `container_click` paketi yollar; ardından
slot, cursor ve full content güncellemelerini toplar.

Beklenen invariant'lar şunlardır:

- Başlangıç content'i container `0`, state `1`, 46 slot ve carried için boş
  değer taşır.
- Console item hazırlığı menu slotu `36` için state `2`, count `8` ve item ID
  `1` slot gözlemi üretir.
- Server click'i mismatch saymaz ve pickup davranışını uygular.
- Yanlış changed-slot hash'i nedeniyle slot `36` için state `3` ile empty
  `container_set_slot` gönderilir.
- Carried hash'i doğru olduğu için `set_cursor_item` gönderilmez.
- Item hazırlığı dışındaki `container_set_content` gözlemi oluşmaz; full resync
  yolu çalışmaz.
- Rust probe bir `container_click` paketi yollar ve gönderilen state-id `2`
  olarak gözlenir.

## Sınır

Bu dilim component'siz stone stack için yanlış changed-slot hash'inin tek slot
düzeltmesine yol açmasını kapsar. Yanlış carried hash'i, iki hash'in birlikte
yanlış olması, component hash'leri, kısmi changed-slot map'i, diğer click
input'ları, stale state varyantları ve özel container'lar sonraki P13
dilimleridir.


## Vanilla server-conformance doğrulaması

**Tarih:** 2026-09-11

**Referans:** Java Edition 26.2 / protokol 776 / Java 25

**Sunucu:** Mojang vanilla 26.2 dedicated server, offline yerel koşu

**Sunucu SHA-1:** `823e2250d24b3ddac457a60c92a6a941943fcd6a`

## Çalıştırma

Sunucu `/tmp/photon-vanilla-26.2-test` altında, `online-mode=false`, seed `776`,
view/simulation distance `5`, `gamemode=survival`, `force-gamemode=true`,
peaceful ve spawn protection `0` ayarlarıyla çalıştı.

Her oturumda player bağlandıktan sonra vanilla server console'undan şu hazırlık
komutu gönderildi:

```text
item replace entity <player> hotbar.0 with minecraft:stone 8
```

Geçerli iki probe koşusu:

```text
cargo run --offline --release --example survival_inventory_wrong_slot_prediction_probe -- 127.0.0.1:25566 InvWrongSlot2
cargo run --offline --release --example survival_inventory_wrong_slot_prediction_probe -- 127.0.0.1:25566 InvWrongSlot3
```

Probe state `2`, slot `36`, changed-slot hash'i olarak click öncesi stone
stack'i ve carried hash'i olarak click sonrası stone stack'ini gönderdi.

## Sonuçlar

| Gözlem | InvWrongSlot2 | InvWrongSlot3 |
| --- | ---: | ---: |
| play login / player loaded | `true / true` | `true / true` |
| container click packets sent | `1` | `1` |
| gönderilen state-id / slot / button / input id | `2 / 36 / 0 / 0` | `2 / 36 / 0 / 0` |
| changed-slot prediction | `slot36=item1,count8,empty components` | `slot36=item1,count8,empty components` |
| carried prediction | `item1,count8,empty components` | `item1,count8,empty components` |
| initial container content | `state=1, items=46, all empty, carried=empty` | `state=1, items=46, all empty, carried=empty` |
| item hazırlık slot observation | `state=2, slot36=count8,item1` | `state=2, slot36=count8,item1` |
| click sonrası slot düzeltmesi | `state=3, slot36=empty` | `state=3, slot36=empty` |
| click sonrası cursor update | `0` | `0` |
| additional full content observations | `0` | `0` |
| chunk packet / decoded / batch finished | `157 / 157 / 51` | `157 / 157 / 51` |
| received packet | `1078` | `1246` |

Her iki vanilla oturumunda state-id doğru olduğu için click uygulandı ve server
full resync göndermedi. Changed-slot remote hash'i click öncesi stone stack'i
olarak kaldığından yalnızca state `3` slot düzeltmesi geldi. Carried hash'i
click sonrası stone stack'iyle eşleştiği için cursor paketi gelmedi. Server
logunda geçerli koşular için protocol error veya timeout gözlenmedi.

## Sonuç ve açık kapsam

İki bağımsız Mojang vanilla 26.2 oturumu, yanlış changed-slot hash'inin doğru
carried prediction ile birlikte yalnızca authoritative slot düzeltmesi
ürettiğini doğrular. Sözleşme ayrıntıları [survival inventory wrong changed-slot
prediction contract belgesindedir](#sözleşme).

Yanlış carried hash'i, iki hash'in birlikte yanlış olması, component hash'leri,
kısmi changed-slot map'i, diğer click input'ları ve özel menu akışları açık
kalır.
