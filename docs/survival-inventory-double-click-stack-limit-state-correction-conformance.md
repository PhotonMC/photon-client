# P13 — Survival inventory double-click stack limit state-id correction sözleşmesi — [server-conformance] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, vanilla `PICKUP_ALL` taramasının carried item'ın maksimum stack
boyutunda durmasını ve client'ın eski container state-id göndermesi durumunda
server'ın authoritative full content düzeltmesini sınar. Probe, ikinci stack'in
tamamının değil yalnızca cursor kapasitesi kadarının alınmasını görünür kılar.

## Java referansı

`GameProtocols.java` serverbound dispatch tablosunda
`ServerboundContainerClickPacket` 18 numaralı play paketidir. Packet alanları
container id, state id, slot short'u, button byte'ı, `ContainerInput`, değişen
slotların kısa indeksli map'i ve `HashedStack` carried değeridir.

`ContainerInput.PICKUP` kimliği `0`, `ContainerInput.PICKUP_ALL` kimliği `6`'dır.
`AbstractContainerMenu` `PICKUP_ALL` dalında carried count'u
`getMaxStackSize()` ile karşılaştırır. `target.safeTake`, carried'da kalan
kapasiteyi üst sınır alır; böylece count `60` taşıyan cursor, aynı item'dan
count `8` olan source slotun yalnızca `4` item'ını alır ve maksimum count `64`'
te kalır.

`ServerGamePacketListenerImpl.handleContainerClick` packet state-id'sini menu
state-id'siyle karşılaştırır, click'i uygular ve state-id farklıysa
`broadcastFullState` çağırır. `ServerPlayer` full content yayınında menu
state-id'sini artırır; `ClientboundContainerSetContentPacket` container id,
state id, 46 optional slot ve carried item'i taşır.

## Probe girdisi ve akışı

- Oyun modu: `SURVIVAL`
- Container: id `0`, player inventory menu
- Hazırlık: hotbar index `0` / menu slotu `36` içine stone count `60`
- Hazırlık: main inventory index `0` / menu slotu `9` içine stone count `8`
- Stone item registry ID: `1`
- Tüm client click packet'leri: state-id `0`, boş changed-slot map'i ve empty
  carried hash'i
- Slot dizisi:
  1. `(slot 36, button 0, input 0)` — 60'lı stack'i cursor'a pickup
  2. `(slot 10, button 0, input 6)` — boş hedef üzerinden `PICKUP_ALL`

Slot `10`, `PICKUP_ALL` için boş hedef koşulunu sağlar; button `0` ile ileri
tarama slot `9`'daki stack'e ulaşır. Probe, dünya hazır olduktan ve iki item
hazırlığı için bekledikten sonra bu iki paketi yollar; ardından full content
düzeltmelerini toplar.

Beklenen invariant'lar şunlardır:

- Başlangıç content'i container `0`, state `1`, 46 slot ve carried için boş
  değer taşır.
- Console item hazırlıkları state `2`'de slot `36` için count `60`, state `3`'te
  slot `9` için count `8` ve item ID `1` gözlemleri üretir.
- İlk eski state-id'li click pickup sonrası state `4` full content'inde slot
  `36` boştur, slot `9` count `8` taşır ve carried count `60` olur.
- İkinci eski state-id'li click sonrası state `5` full content'inde slot `9`
  count `4`, slot `36` boş ve carried count `64` olur.
- Rust probe iki `container_click` paketi gönderir ve gönderilen state-id `0`
  olarak gözlenir.

## Sınır

Bu dilim ileri `PICKUP_ALL` taramasında carried stack kapasitesinin dolmasını
kapsar. Button `1` ile geriye tarama, farklı item/component eşleşmeleri, doğru
state-id ile client prediction/hash map'i, yanlış state'in diğer varyantları,
creative menu ve özel container'lar sonraki P13 dilimleridir.


## Vanilla server-conformance doğrulaması

**Tarih:** 2026-09-11

**Referans:** Java Edition 26.2 / protokol 776 / Java 25

**Sunucu:** Mojang vanilla 26.2 dedicated server, offline yerel koşu

**Sunucu SHA-1:** `823e2250d24b3ddac457a60c92a6a941943fcd6a`

## Çalıştırma

Sunucu `/tmp/voxel-vanilla-26.2-test` altında, `online-mode=false`, seed `776`,
view/simulation distance `5`, `gamemode=survival`, `force-gamemode=true`,
peaceful ve spawn protection `0` ayarlarıyla çalıştı.

Her oturumda player bağlandıktan sonra vanilla server console'undan şu iki
hazırlık komutu gönderildi:

```text
item replace entity <player> hotbar.0 with minecraft:stone 60
item replace entity <player> inventory.0 with minecraft:stone 8
```

Geçerli iki probe koşusu:

```text
cargo run --offline --release --example survival_inventory_double_click_stack_limit_state_correction_probe -- 127.0.0.1:25566 InvDoubleLimit2
cargo run --offline --release --example survival_inventory_double_click_stack_limit_state_correction_probe -- 127.0.0.1:25566 InvDoubleLimit3
```

Probe, player inventory menu state `1` iken hotbar slotu `36`'ya count `60`, main
inventory slotu `9`'a count `8` hazırlanmasını bekledi. Ardından state-id `0`,
boş changed-slot map'i ve empty carried hash'i ile pickup ve boş slot `10`
üzerinden `PICKUP_ALL` click'lerini gönderdi.

## Sonuçlar

| Gözlem | InvDoubleLimit2 | InvDoubleLimit3 |
| --- | ---: | ---: |
| play login / player loaded | `true / true` | `true / true` |
| container click packets sent | `2` | `2` |
| gönderilen state-id / slot-button-input dizisi | `0 / 36-0-0, 10-0-6` | `0 / 36-0-0, 10-0-6` |
| initial container content | `state=1, items=46, all empty, carried=empty` | `state=1, items=46, all empty, carried=empty` |
| item hazırlık slot observations | `state=2, slot36=count60,item1; state=3, slot9=count8,item1` | `state=2, slot36=count60,item1; state=3, slot9=count8,item1` |
| pickup full resync | `state=4, slot36=empty, slot9=count8,item1, carried=count60,item1` | `state=4, slot36=empty, slot9=count8,item1, carried=count60,item1` |
| stack limit full resync | `state=5, slot9=count4,item1, slot36=empty, carried=count64,item1` | `state=5, slot9=count4,item1, slot36=empty, carried=count64,item1` |
| chunk packet / decoded / batch finished | `157 / 157 / 51` | `157 / 157 / 51` |
| received packet | `999` | `2009` |

Her iki vanilla oturumunda `PICKUP_ALL` cursor kapasitesine ulaştı; count `60`
taşıyan cursor, slot `9`'daki count `8` stack'ten yalnızca `4` aldı. Son full
content state `5`'te cursor count `64`, slot `9` count `4` oldu. Server logunda
koşular için protocol error veya timeout gözlenmedi.

## Sonuç ve açık kapsam

İki bağımsız Mojang vanilla 26.2 oturumu, survival button `0` `PICKUP_ALL`
akışının max stack sınırını ve stale state-id sonrası slot/cursor full resync'ini
doğrular. Sözleşme ayrıntıları [survival inventory double-click stack limit
state-id correction contract belgesindedir](#sözleşme).

Button `1` ile geriye tarama, farklı item/component eşleşmeleri, doğru state-id
ile client'ın hashed prediction'ı, yanlış state'in diğer varyantları ve özel
menu akışları açık kalır.
