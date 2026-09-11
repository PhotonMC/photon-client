# P13 — Survival inventory drag distribution state-id correction sözleşmesi

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
