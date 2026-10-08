# P13 — Survival inventory correct prediction sözleşmesi — [server-conformance] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, client'ın inventory click packet'inde vanilla'nın beklediği post-click
slot/cursor hash'lerini ve doğru menu state-id'sini göndermesi durumunda
server'ın full resync veya ek slot/cursor düzeltmesi yayınlamamasını sınar.

## Java referansı

`GameProtocols.java` serverbound dispatch tablosunda
`ServerboundContainerClickPacket` 18 numaralı play paketidir. `HashedStack`
non-empty değer için optional presence byte'ı, item holder'ı, count'u ve
`HashedPatchMap` component patch'ini taşır. Empty hash yalnızca optional empty
değeridir. Stone stack'inin component patch'i boş olduğundan prediction içinde
added component map'i ve removed component set'i boş olur.

`ServerPlayer` component hash'lerini registry serialization context ile CRC32C
üzerinden üretir. Empty component patch'inde hash alanı olmadığı için bu probe
component hash değerine ihtiyaç duymaz. `RemoteSlot.receive` packet'teki hash'i
server'ın mevcut remote beklentisi olarak kaydeder.

`MultiPlayerGameMode.handleContainerInput`, click'i client menüsünde uyguladıktan
sonra değişen slotların ve carried değerinin yeni durumundan `HashedStack`
oluşturur. Bu yüzden prediction, click öncesi kaynak stack'ini değil click
sonrası slot/cursor durumunu taşır. `ServerGamePacketListenerImpl.handleContainerClick`,
state-id packet ile menu state-id'si eşitse bu hash'leri remote beklenti olarak
kaydeder ve `broadcastChanges` yolunu kullanır. Pickup click'i slotu boşaltıp
carried stack'i oluşturduğunda iki hash de gerçek server durumuyla eşleşirse
ek slot veya cursor paketi yayınlanmaz. Hash eşleşmezse server'ın
`ClientboundContainerSetSlotPacket` ve `ClientboundSetCursorItemPacket` ile
düzeltme yapması ayrı bir varyanttır.

## Probe girdisi ve akışı

- Oyun modu: `SURVIVAL`
- Container: id `0`, player inventory menu
- Hazırlık: hotbar index `0` / menu slotu `36` içine `minecraft:stone 8`
- Stone item registry ID: `1`
- Menu state-id: `2` (initial content state `1` sonrasındaki hazırlık state'i)
- Click: slot `36`, button `0`, `PICKUP` input id `0`
- Changed-slot prediction: map içinde slot `36`, pickup sonrası boş hash
- Carried prediction: pickup sonrası item id `1`, count `8`, boş component
  map/set içeren non-empty hash

Probe, item hazırlığı sonrası doğru state-id ve prediction ile tek
`container_click` paketi yollar; ardından slot/cursor güncellemelerini toplar.

Beklenen invariant'lar şunlardır:

- Başlangıç content'i container `0`, state `1`, 46 slot ve carried için boş
  değer taşır.
- Console item hazırlığı menu slotu `36` için state `2`, count `8` ve item ID
  `1` slot gözlemi üretir.
- Server click'i mismatch saymaz ve pickup davranışını uygular.
- Prediction gerçek post-click durumla eşleştiği için hazırlık gözlemi dışında
  slot `36` için `container_set_slot` gönderilmez.
- Prediction gerçek post-click carried durumla eşleştiği için `set_cursor_item`
  gönderilmez.
- Item hazırlığı dışındaki `container_set_content` gözlemi oluşmaz; full resync
  yerine artımlı yayın yapılır.
- Rust probe bir `container_click` paketi yollar ve gönderilen state-id `2`
  olarak gözlenir.

## Sınır

Bu dilim component'siz stone stack için doğru state-id, pickup sonrası
changed-slot hash'i, non-empty carried hash'i ve eşleşen prediction'ın ek
paketleri bastırmasını kapsar. Component hash değerleri, yanlış veya kısmi
prediction, diğer click input'ları, stale state varyantları ve özel container'lar
sonraki P13 dilimleridir.


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
cargo run --offline --release --example survival_inventory_correct_prediction_probe -- 127.0.0.1:25566 InvPredict6
cargo run --offline --release --example survival_inventory_correct_prediction_probe -- 127.0.0.1:25566 InvPredict7
```

İlk `InvPredict2` denemesi, non-empty `HashedStack` optional presence byte'ı
encoder'da eksik olduğu için vanilla decoder tarafından reddedildi ve parity
sonucu olarak kullanılmadı. `InvPredict3` ve `InvPredict4` decoder'dan geçen
ama click öncesi kaynak stack'ini changed-slot hash'i olarak taşıyan keşif
koşullarıydı; doğru prediction kanıtı olarak kullanılmadı. `InvPredict5`
koşusunda item hazırlama console komutu gönderilmediği için başlangıç slotu
boş kaldı ve bu koşu da dışarıda bırakıldı.

Düzeltilen probe state `2`, pickup sonrası slot `36` için empty hash ve carried
stack için item id `1`, count `8`, boş component map/set içeren non-empty hash
gönderdi.

## Sonuçlar

| Gözlem | InvPredict6 | InvPredict7 |
| --- | ---: | ---: |
| play login / player loaded | `true / true` | `true / true` |
| container click packets sent | `1` | `1` |
| gönderilen state-id / slot / button / input id | `2 / 36 / 0 / 0` | `2 / 36 / 0 / 0` |
| changed-slot prediction | `slot36=empty` | `slot36=empty` |
| carried prediction | `item1,count8,empty components` | `item1,count8,empty components` |
| initial container content | `state=1, items=46, all empty, carried=empty` | `state=1, items=46, all empty, carried=empty` |
| item hazırlık slot observation | `state=2, slot36=count8,item1` | `state=2, slot36=count8,item1` |
| click sonrası incremental slot update | `0` | `0` |
| click sonrası cursor update | `0` | `0` |
| additional full content observations | `0` | `0` |
| chunk packet / decoded / batch finished | `157 / 157 / 51` | `157 / 157 / 51` |
| received packet | `970` | `2501` |

Her iki geçerli vanilla oturumunda doğru post-click prediction ile stale full
resync veya incremental correction yolu çalışmadı. Hazırlık komutunun ürettiği
slot `36` state `2` gözlemi dışında slot ve cursor paketi gelmedi; item
hazırlığı dışındaki full content paket sayısı `0` kaldı. Server logunda geçerli
koşular için protocol error veya timeout gözlenmedi.

## Sonuç ve açık kapsam

İki bağımsız Mojang vanilla 26.2 oturumu, doğru state-id ve component'siz
`HashedStack` post-click prediction'ının server düzeltme paketlerini bastırdığını
doğrular. Sözleşme ayrıntıları [survival inventory correct prediction contract
belgesindedir](#sözleşme).

Component hash değerleri, yanlış veya kısmi prediction, diğer click input'ları,
stale state varyantları ve özel menu akışları açık kalır.
