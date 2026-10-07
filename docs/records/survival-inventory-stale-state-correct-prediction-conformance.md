# P13 — Survival inventory stale state correct prediction sözleşmesi — [server-conformance] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, inventory pickup packet'inde post-click slot ve carried hash'leri
doğru olsa bile state-id eski olduğunda vanilla server'ın full resync seçmesini
sınar.

## Java referansı

`ServerGamePacketListenerImpl.handleContainerClick`, packet state-id'sini menu
state-id'siyle karşılaştırır. Değerler eşit değilse `fullResyncNeeded` true olur.
Server click'i yine `clicked` ile uygular, packet'teki remote hash'leri kaydeder
ve sonunda `broadcastFullState` çağırır. Bu dal hash eşleşmelerine bakarak
artımlı slot/cursor yayınını bastırmaz.

`AbstractContainerMenu.broadcastFullState`, pickup sonrası bütün slotları ve
carried item'ı yeni state-id ile birlikte yollar. Bu nedenle doğru post-click
prediction gönderilmiş olsa bile stale state-id tam content yayınının yerini
değiştirmez.

## Probe girdisi ve akışı

- Oyun modu: `SURVIVAL`
- Container: id `0`, player inventory menu
- Hazırlık: hotbar index `0` / menu slotu `36` içine `minecraft:stone 8`
- Stone item registry ID: `1`
- Server menu state-id: `2`
- Gönderilen stale state-id: `0`
- Click: slot `36`, button `0`, `PICKUP` input id `0`
- Changed-slot prediction: map içinde slot `36`, pickup sonrası boş hash
- Carried prediction: item ID `1`, count `8`, boş added/removed component
  map/set içeren doğru non-empty hash

Probe, item hazırlığı sonrası tek `container_click` paketi yollar; ardından
slot, cursor ve full content güncellemelerini toplar.

Beklenen invariant'lar şunlardır:

- Başlangıç content'i container `0`, state `1`, 46 slot ve carried için boş
  değer taşır.
- Console item hazırlığı slot `36` için state `2`, item ID `1` ve count `8`
  gözlemi üretir.
- Server click'i uygular, fakat state-id mismatch nedeniyle full resync dalını
  seçer.
- Pickup sonrası full content state `3` ile slot `36` empty, carried item `1`
  count `8` olur.
- Doğru hash'lere rağmen artımlı slot veya cursor düzeltmesi gönderilmez;
  full content gözlemi oluşur.

## Sınır

Bu dilim stale state-id ile doğru slot/carried prediction'ın birlikte
gönderilmesini ve full resync önceliğini kapsar. Ahead state-id'nin doğru hash
varyantı, stale state ile component'li item, diğer click input'ları ve özel
container'lar sonraki P13 dilimleridir.


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
cargo run --offline --release --example survival_inventory_stale_state_correct_prediction_probe -- 127.0.0.1:25566 InvStale2
cargo run --offline --release --example survival_inventory_stale_state_correct_prediction_probe -- 127.0.0.1:25566 InvStale3
```

Probe state `0` gönderdi. Server hazırlık komutundan sonra menu state-id `2`
olduğu için bu packet stale idi; changed-slot ve carried hash'leri ise pickup
sonrası gerçek durumla eşleşecek şekilde gönderildi.

## Sonuçlar

| Gözlem | InvStale2 | InvStale3 |
| --- | ---: | ---: |
| play login / player loaded | `true / true` | `true / true` |
| container click packets sent | `1` | `1` |
| gönderilen state-id / slot / button / input id | `0 / 36 / 0 / 0` | `0 / 36 / 0 / 0` |
| changed-slot prediction | `slot36=empty` | `slot36=empty` |
| carried prediction | `item1,count8,empty components` | `item1,count8,empty components` |
| initial container content | `state=1, items=46, all empty, carried=empty` | `state=1, items=46, all empty, carried=empty` |
| item hazırlık slot observation | `state=2, slot36=count8,item1` | `state=2, slot36=count8,item1` |
| click sonrası full content | `state=3, slot36=empty, carried=count8,item1` | `state=3, slot36=empty, carried=count8,item1` |
| click sonrası incremental slot update | `0` | `0` |
| click sonrası cursor update | `0` | `0` |
| chunk packet / decoded / batch finished | `157 / 157 / 51` | `157 / 157 / 51` |
| received packet | `2025` | `1606` |

Her iki vanilla oturumunda click uygulanmasına rağmen state-id mismatch full
resync'i seçti. Doğru changed-slot ve carried hash'leri bu dalı artımlı yayına
çevirmedi; server state `3` ile 46 slotu ve carried `stone x8` değerini tek full
content gözleminde gönderdi. Server logunda geçerli koşular için protocol error
veya timeout gözlenmedi.

## Sonuç ve açık kapsam

İki bağımsız Mojang vanilla 26.2 oturumu, state-id kontrolünün doğru hash
prediction'larından önce full resync kararını belirlediğini doğrular. Sözleşme
ayrıntıları [survival inventory stale state correct prediction contract
belgesindedir](#sözleşme).

Ahead state-id'nin doğru hash varyantı, stale state ile component'li item, diğer
click input'ları ve özel container akışları açık kalır.
