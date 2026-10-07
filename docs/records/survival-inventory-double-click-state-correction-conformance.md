# P13 — Survival inventory double-click state-id correction sözleşmesi — [server-conformance] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, carried stack ile player inventory içindeki aynı item stack'lerini
vanilla `PICKUP_ALL` davranışıyla toplama ve client'ın eski container state-id
göndermesi durumunda server'ın authoritative full content düzeltmesini sınar.
Probe, boş hedef slot koşulunu ve iki ayrı stack'in cursor'da birleşmesini
kablo seviyesinde görünür kılar.

## Java referansı

`GameProtocols.java` serverbound dispatch tablosunda
`ServerboundContainerClickPacket` 18 numaralı play paketidir. Packet alanları
container id, state id, slot short'u, button byte'ı, `ContainerInput`, değişen
slotların kısa indeksli map'i ve `HashedStack` carried değeridir.

`ContainerInput.PICKUP` kimliği `0`, `ContainerInput.PICKUP_ALL` kimliği `6`'dır.
`AbstractContainerMenu` `PICKUP_ALL` dalını yalnızca carried item boş değilken
ve tıklanan slot boş veya pickup'a kapalıyken işler. Button `0` slotları
başlangıçtan sona doğru tarar; aynı item ve component'lere sahip stack'lerden
carried max stack sınırına kadar item alır.

`ServerGamePacketListenerImpl.handleContainerClick` packet state-id'sini menu
state-id'siyle karşılaştırır, click'i uygular ve state-id farklıysa
`broadcastFullState` çağırır. `ServerPlayer` full content yayınında menu
state-id'sini artırır; `ClientboundContainerSetContentPacket` container id,
state id, 46 optional slot ve carried item'i taşır.

## Probe girdisi ve akışı

- Oyun modu: `SURVIVAL`
- Container: id `0`, player inventory menu
- Hazırlık: hotbar index `0` / menu slotu `36` içine stone count `8`
- Hazırlık: main inventory index `0` / menu slotu `9` içine stone count `3`
- Stone item registry ID: `1`
- Tüm client click packet'leri: state-id `0`, boş changed-slot map'i ve empty
  carried hash'i
- Slot dizisi:
  1. `(slot 36, button 0, input 0)` — 8'li stack'i cursor'a pickup
  2. `(slot 10, button 0, input 6)` — boş hedef üzerinden `PICKUP_ALL`

Slot `10`, `PICKUP_ALL` için boş hedef koşulunu sağlar; ileri tarama slot `9`'
daki 3 taş stack'ini cursor'a ekler. Probe, dünya hazır olduktan ve iki item
hazırlığı için bekledikten sonra bu iki paketi yollar; ardından full content
düzeltmelerini toplar.

Beklenen invariant'lar şunlardır:

- Başlangıç content'i container `0`, state `1`, 46 slot ve carried için boş
  değer taşır.
- Console item hazırlıkları state `2`'de slot `36` için count `8`, state `3`'te
  slot `9` için count `3` ve item ID `1` gözlemleri üretir.
- İlk eski state-id'li click pickup sonrası state `4` full content'inde slot
  `36` boştur, slot `9` count `3` taşır ve carried count `8` olur.
- İkinci eski state-id'li click sonrası state `5` full content'inde slot `9` ve
  slot `36` boştur; carried count `11` ve item ID `1` olur.
- Rust probe iki `container_click` paketi gönderir ve gönderilen state-id `0`
  olarak gözlenir.

## Sınır

Bu dilim button `0` ile ileri taranan survival `PICKUP_ALL` ve iki stack'in
cursor'da birleşmesini kapsar. Button `1` ile geriye tarama, cursor max stack
sınırı, doğru state-id ile client prediction/hash map'i, yanlış state'in diğer
varyantları, creative menu ve özel container'lar sonraki P13 dilimleridir.


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
item replace entity <player> hotbar.0 with minecraft:stone 8
item replace entity <player> inventory.0 with minecraft:stone 3
```

Geçerli iki probe koşusu:

```text
cargo run --offline --release --example survival_inventory_double_click_state_correction_probe -- 127.0.0.1:25566 InvDouble2
cargo run --offline --release --example survival_inventory_double_click_state_correction_probe -- 127.0.0.1:25566 InvDouble3
```

Probe, player inventory menu state `1` iken hotbar slotu `36`'ya count `8`, main
inventory slotu `9`'a count `3` hazırlanmasını bekledi. Ardından state-id `0`,
boş changed-slot map'i ve empty carried hash'i ile pickup ve boş slot `10`
üzerinden `PICKUP_ALL` click'lerini gönderdi.

## Sonuçlar

| Gözlem | InvDouble2 | InvDouble3 |
| --- | ---: | ---: |
| play login / player loaded | `true / true` | `true / true` |
| container click packets sent | `2` | `2` |
| gönderilen state-id / slot-button-input dizisi | `0 / 36-0-0, 10-0-6` | `0 / 36-0-0, 10-0-6` |
| initial container content | `state=1, items=46, all empty, carried=empty` | `state=1, items=46, all empty, carried=empty` |
| item hazırlık slot observations | `state=2, slot36=count8,item1; state=3, slot9=count3,item1` | `state=2, slot36=count8,item1; state=3, slot9=count3,item1` |
| pickup full resync | `state=4, slot36=empty, slot9=count3,item1, carried=count8,item1` | `state=4, slot36=empty, slot9=count3,item1, carried=count8,item1` |
| double-click full resync | `state=5, slot9=empty, slot36=empty, carried=count11,item1` | `state=5, slot9=empty, slot36=empty, carried=count11,item1` |
| chunk packet / decoded / batch finished | `157 / 157 / 51` | `157 / 157 / 51` |
| received packet | `1833` | `1529` |

Her iki vanilla oturumunda pickup sonrası cursor'daki 8 taş, boş hedef slot
`10` üzerinden ileri taranan slot `9`'daki 3 taş ile birleşti. Eski state-id
`0` nedeniyle her click sonrası full content state'i arttı; son state `5`'te
slot `9` ve `36` boş, carried count `11` oldu. Server logunda bu koşular için
protocol error veya timeout gözlenmedi.

## Sonuç ve açık kapsam

İki bağımsız Mojang vanilla 26.2 oturumu, survival button `0` `PICKUP_ALL`
akışının boş hedef koşulunu, item stack birleşmesini ve stale state-id sonrası
slot/cursor full resync'ini doğrular. Sözleşme ayrıntıları [survival inventory
double-click state-id correction contract belgesindedir](#sözleşme).

Button `1` ile geriye tarama, cursor max stack sınırı, doğru state-id ile
client'ın hashed prediction'ı, yanlış state'in diğer varyantları ve özel menu
akışları açık kalır.
