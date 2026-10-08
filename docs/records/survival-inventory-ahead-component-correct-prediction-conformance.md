# P13 — Survival inventory ahead component correct prediction sözleşmesi — [server-conformance] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, component taşıyan bir inventory item'ı pickup ederken component hash'i
doğru olsa bile state-id menu state-id'sinin ilerisinde olduğunda vanilla server'ın
component'li full resync seçmesini sınar.

## Java referansı

`ServerGamePacketListenerImpl.handleContainerClick`, packet state-id'si menu
state-id'sine eşit değilse full resync dalını seçer. Bu kontrol ileri ve eski
state-id'leri aynı mismatch olarak ele alır. Click ve remote hash kayıtları
yapıldıktan sonra `AbstractContainerMenu.broadcastFullState` çağrılır.

`ServerPlayer` component hash'lerini registry serialization context ile
`HashOps.CRC32C_INSTANCE` üzerinden üretir. `CustomModelData` içinde yalnızca
`floats=[1.0]` bulunan milk bucket için Java oracle hash'i `0x77a4a24f`'dir.
Bu hash'in doğru olması state mismatch nedeniyle full content yayınının yerini
değiştirmez.

## Probe girdisi ve akışı

- Oyun modu: `SURVIVAL`
- Container: id `0`, player inventory menu
- Hazırlık: hotbar index `0` / menu slotu `36` içine
  `minecraft:milk_bucket[minecraft:custom_model_data={floats:[1.0f]}]`
- Milk bucket item registry ID: `1046`
- `custom_model_data` component type ID: `17`
- Server menu state-id: `2`
- Gönderilen ahead state-id: `99`
- Click: slot `36`, button `0`, `PICKUP` input id `0`
- Changed-slot prediction: map içinde slot `36`, pickup sonrası boş hash
- Carried prediction: item ID `1046`, count `1`, added component type `17` ve
  Java oracle hash'i `0x77a4a24f`, removed set'i boş

Probe, item hazırlığı sonrası tek `container_click` paketi yollar; ardından
slot, cursor ve full content güncellemelerini toplar.

Beklenen invariant'lar şunlardır:

- Başlangıç content'i container `0`, state `1`, 46 slot ve carried için boş
  değer taşır.
- Console item hazırlığı slot `36` için state `2`, item ID `1046`, count `1`
  ve positive component ID `17` gözlemi üretir.
- Server click'i uygular, fakat ahead state-id mismatch nedeniyle full resync
  dalını seçer.
- Pickup sonrası full content state `3` ile slot `36` empty, carried item
  `1046`, count `1` ve positive component ID `17` olur.
- Doğru component hash'ine rağmen artımlı slot veya cursor düzeltmesi
  gönderilmez; full content gözlemi oluşur.

## Sınır

Bu dilim ahead state-id ile component'li doğru slot/carried prediction'ın
birlikte gönderilmesini ve full resync önceliğini kapsar. Diğer component
codec'leri, yanlış component hash'leri, diğer click input'ları ve özel
container'lar sonraki P13 dilimleridir.


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
[survival inventory ahead component correct prediction contract belgesindedir](#sözleşme).

Diğer component codec'leri, yanlış component hash'leri, diğer click input'ları ve
özel container akışları açık kalır.
