# P13 — Survival inventory drag distribution state-id correction sözleşmesi — [server-conformance] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, carried item ile player inventory üzerinde iki slota yapılan vanilla
quick-craft dağıtımını ve client'ın eski container state-id göndermesi
durumunda server'ın her adım sonrası authoritative full content düzeltmesini
sınar. Probe, normal survival item stack'inin pickup, drag başlangıcı, iki slot
seçimi ve drag bitişi boyunca server davranışını kablo seviyesinde görünür
kılar.

## Java referansı

`GameProtocols.java` serverbound dispatch tablosunda
`ServerboundContainerClickPacket` 18 numaralı play paketidir. Packet alanları
container id, state id, slot short'u, button byte'ı, `ContainerInput`, değişen
slotların kısa indeksli map'i ve `HashedStack` carried değeridir.

`ContainerInput.PICKUP` kimliği `0`, `ContainerInput.QUICK_CRAFT` kimliği `5`'tir.
Quick-craft button'ının alt iki biti header olarak okunur: `0` başlangıç, `1`
seçili slota ekleme, `2` dağıtımı bitirme; üst bitler quick-craft type'ını
belirler. Type `0`, carried stack'i seçili slot sayısına böler ve her slot için
floor sonucunu kullanır.

`AbstractContainerMenu` quick-craft bitişinde seçili slot sayısını ve carried
count'u kullanarak slot başına yerleştirilecek miktarı hesaplar. İki seçili
slot ve count `8` için her slot `4` alır; carried item tamamen tüketilir.
`ServerGamePacketListenerImpl.handleContainerClick` packet state-id'sini menu
state-id'siyle karşılaştırır, click'i uygular ve state-id farklıysa
`broadcastFullState` çağırır. `ServerPlayer` full content yayınında menu
state-id'sini artırır; `ClientboundContainerSetContentPacket` container id,
state id, 46 optional slot ve carried item'i taşır.

## Probe girdisi ve akışı

- Oyun modu: `SURVIVAL`
- Container: id `0`, player inventory menu
- Hazırlık: console ile hotbar index `0` / menu slotu `36` içine `minecraft:stone 8`
- Stone item registry ID: `1`
- Tüm client click packet'leri: state-id `0`, boş changed-slot map'i ve empty
  carried hash'i
- Quick-craft type: `0`
- Slot dizisi:
  1. `(slot 36, button 0, input 0)` — stack'i cursor'a pickup
  2. `(slot -999, button 0, input 5)` — quick-craft başlat
  3. `(slot 9, button 1, input 5)` — player main inventory ilk slotunu seç
  4. `(slot 36, button 1, input 5)` — hotbar ilk slotunu seç
  5. `(slot -999, button 2, input 5)` — dağıtımı bitir

Probe, dünya hazır olduktan ve item hazırlığı için bekledikten sonra bu beş
paketi arka arkaya yollar; ardından full content düzeltmelerini toplar.

Beklenen invariant'lar şunlardır:

- Başlangıç content'i container `0`, state `1`, 46 slot ve carried için boş
  değer taşır.
- Console item hazırlığı menu slotu `36` için state `2`, count `8` ve item ID
  `1` slot gözlemi üretir.
- Server beş eski state-id'li click'in her birini uygular ve sırasıyla state
  `3`, `4`, `5`, `6`, `7` full content düzeltmeleri yollar.
- State `3`'te pickup sonrası carried count `8` olur; drag başlangıcı ve slot
  seçimlerinde carried count `8` korunur.
- Son state `7` content'inde menu slotları `9` ve `36` count `4`, item ID `1`
  taşır; carried boştur.
- Rust probe beş `container_click` paketi gönderir ve gönderilen state-id `0`
  olarak gözlenir.

## Sınır

Bu dilim iki seçili player inventory slotuna yapılan type `0` survival drag
dağıtımını ve stale state full resync sırasını kapsar. Doğru state-id ile
client prediction/hash map'i, diğer quick-craft type/header'ları, slotların
yerleştirme sınırları, double-click, stack sınırı, creative menu ve özel
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
item replace entity <player> hotbar.0 with minecraft:stone 8
```

Geçerli iki probe koşusu:

```text
cargo run --offline --release --example survival_inventory_drag_distribution_state_correction_probe -- 127.0.0.1:25566 InvDragDist2
cargo run --offline --release --example survival_inventory_drag_distribution_state_correction_probe -- 127.0.0.1:25566 InvDragDist3
```

İlk `InvDragDist1` denemesi, probe'un 30 saniyelik beklemesi nedeniyle vanilla
keepalive zaman aşımına uğradı ve parity sonucu olarak kullanılmadı. Bekleme 20
saniyeye indirildikten sonraki iki koşu aşağıdaki temiz sonuçları verdi.

## Sonuçlar

| Gözlem | InvDragDist2 | InvDragDist3 |
| --- | ---: | ---: |
| play login / player loaded | `true / true` | `true / true` |
| container click packets sent | `5` | `5` |
| gönderilen state-id | `0` | `0` |
| gönderilen slot/button/input dizisi | `36/0/0`, `-999/0/5`, `9/1/5`, `36/1/5`, `-999/2/5` | `36/0/0`, `-999/0/5`, `9/1/5`, `36/1/5`, `-999/2/5` |
| initial container content | `state=1, items=46, all empty, carried=empty` | `state=1, items=46, all empty, carried=empty` |
| item hazırlık slot observation | `state=2, slot36=count8,item1` | `state=2, slot36=count8,item1` |
| ara full resync'ler | `state=3..6, carried=count8,item1` | `state=3..6, carried=count8,item1` |
| son authoritative full resync | `state=7, slot9=count4,item1, slot36=count4,item1, carried=empty` | `state=7, slot9=count4,item1, slot36=count4,item1, carried=empty` |
| chunk packet / decoded / batch finished | `157 / 157 / 51` | `157 / 157 / 51` |
| received packet | `2212` | `3137` |

Her iki vanilla oturumunda beş stale state-id `0` click'i uygulandı; ilk click
stack'i cursor'a aldı, üç quick-craft paketi carried stack'i korudu ve son
paket count `8` stack'ini iki seçili slota floor bölümüyle `4 + 4` dağıttı.
Vanilla son full content state `7` ile iki slotu ve boş cursor'ı birlikte
bildirdi. Server logunda bu koşular için protocol error veya timeout gözlenmedi.

## Sonuç ve açık kapsam

İki bağımsız Mojang vanilla 26.2 oturumu, survival type `0` iki-slot drag
dağıtımının packet sırasını, stale state-id sonrası beş full resync artışını ve
son slot/cursor içeriğini doğrular. Sözleşme ayrıntıları [survival inventory
drag distribution state-id correction contract belgesindedir](#sözleşme).

Doğru state-id ile client'ın hashed prediction'ı, diğer quick-craft
type/header'ları, yanlış state'in diğer varyantları, double-click, stack sınırı
ve özel menu akışları açık kalır.
