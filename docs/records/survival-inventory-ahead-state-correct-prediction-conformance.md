# P13 — Survival inventory ahead state correct prediction sözleşmesi — [server-conformance] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, inventory pickup packet'inde post-click slot ve carried hash'leri
doğru olsa bile state-id menu state-id'sinin ilerisinde olduğunda vanilla server'ın
full resync seçmesini sınar.

## Java referansı

`ServerGamePacketListenerImpl.handleContainerClick`, packet state-id'si menu
state-id'sine eşit değilse `fullResyncNeeded` değerini true yapar. Bu karşılaştırma
state-id'nin eski veya ileri olmasını ayırmaz. Server click'i yine uygular, remote
hash kayıtlarını alır ve sonunda `broadcastFullState` çağırır.

`AbstractContainerMenu.broadcastFullState`, pickup sonrası bütün slotları ve
carried item'ı yeni state-id ile yollar. Bu nedenle doğru post-click hash'leri,
ileri state-id'nin full resync kararını artımlı slot/cursor yayınına çeviremez.

## Probe girdisi ve akışı

- Oyun modu: `SURVIVAL`
- Container: id `0`, player inventory menu
- Hazırlık: hotbar index `0` / menu slotu `36` içine `minecraft:stone 8`
- Stone item registry ID: `1`
- Server menu state-id: `2`
- Gönderilen ahead state-id: `99`
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
- Server click'i uygular, fakat ahead state-id mismatch nedeniyle full resync
  dalını seçer.
- Pickup sonrası full content state `3` ile slot `36` empty, carried item `1`
  count `8` olur.
- Doğru hash'lere rağmen artımlı slot veya cursor düzeltmesi gönderilmez;
  full content gözlemi oluşur.

## Sınır

Bu dilim ahead state-id ile doğru slot/carried prediction'ın birlikte
gönderilmesini ve full resync önceliğini kapsar. Ahead state ile component'li
item, diğer click input'ları ve özel container'lar sonraki P13 dilimleridir.


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
cargo run --offline --release --example survival_inventory_ahead_state_correct_prediction_probe -- 127.0.0.1:25566 InvAhead2
cargo run --offline --release --example survival_inventory_ahead_state_correct_prediction_probe -- 127.0.0.1:25566 InvAhead3
```

Probe state `99` gönderdi. Server hazırlık komutundan sonra menu state-id `2`
olduğu için bu packet ahead idi; changed-slot ve carried hash'leri pickup
sonrası gerçek durumla eşleşecek şekilde gönderildi.

## Sonuçlar

| Gözlem | InvAhead2 | InvAhead3 |
| --- | ---: | ---: |
| play login / player loaded | `true / true` | `true / true` |
| container click packets sent | `1` | `1` |
| gönderilen state-id / slot / button / input id | `99 / 36 / 0 / 0` | `99 / 36 / 0 / 0` |
| changed-slot prediction | `slot36=empty` | `slot36=empty` |
| carried prediction | `item1,count8,empty components` | `item1,count8,empty components` |
| initial container content | `state=1, items=46, all empty, carried=empty` | `state=1, items=46, all empty, carried=empty` |
| item hazırlık slot observation | `state=2, slot36=count8,item1` | `state=2, slot36=count8,item1` |
| click sonrası full content | `state=3, slot36=empty, carried=count8,item1` | `state=3, slot36=empty, carried=count8,item1` |
| click sonrası incremental slot update | `0` | `0` |
| click sonrası cursor update | `0` | `0` |
| chunk packet / decoded / batch finished | `157 / 157 / 51` | `157 / 157 / 51` |
| received packet | `2586` | `1532` |

Her iki vanilla oturumunda click uygulandı ve ahead state-id mismatch full
resync'i seçti. Doğru changed-slot ve carried hash'leri bu dalı artımlı yayına
çevirmedi; server state `3` ile 46 slotu ve carried `stone x8` değerini full
content içinde gönderdi. Server logunda geçerli koşular için protocol error
veya timeout gözlenmedi.

## Sonuç ve açık kapsam

İki bağımsız Mojang vanilla 26.2 oturumu, ahead state-id'nin eski state-id ile
aynı full resync kararını verdiğini ve doğru hash prediction'larının bu kararı
değiştirmediğini doğrular. Sözleşme ayrıntıları [survival inventory ahead state
correct prediction contract belgesindedir](#sözleşme).

Ahead state ile component'li item, diğer click input'ları ve özel container
akışları açık kalır.
