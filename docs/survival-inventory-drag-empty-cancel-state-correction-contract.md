# P13 — Survival inventory empty drag cancellation state-id correction sözleşmesi

Bu dilim, carried item boşken survival player inventory üzerinde drag
başlangıcının vanilla tarafından iptal edilmesini ve client'ın eski container
state-id göndermesi durumunda server'ın authoritative no-op full content
düzeltmesini sınar. Probe, gerçek item dağıtımı yapmaz; drag durumunun boş
carried ile sıfırlanmasını görünür kılar.

## Java referansı

`GameProtocols.java` serverbound dispatch tablosunda
`ServerboundContainerClickPacket` 18 numaralı play paketidir. Packet alanları
container id, state id, slot short'u, button byte'ı, `ContainerInput`, değişen
slotların kısa indeksli map'i ve `HashedStack` carried değeridir.
`ContainerInput.QUICK_CRAFT` kimliği `5`'tir; button `0` quick-craft header `0`
ve type `0` başlangıcını ifade eder. Boş `HashedStack` optional codec'te tek
boolean `false` olarak taşınır.

`AbstractContainerMenu` quick-craft dalı header'ı okur. Başlangıç durumu için
header `0` kabul edildikten sonra carried item boşsa `resetQuickCraft` çağrılır;
slot listesine item eklenmez ve player inventory değişmez.
`ServerGamePacketListenerImpl.handleContainerClick` packet state-id'sini menu
state-id'siyle karşılaştırır, click'i uygular, sonra state-id farklıysa
`broadcastFullState` çağırır. `ServerPlayer` full content yayınında menu
state-id'sini artırır; `ClientboundContainerSetContentPacket` container id,
state id, 46 optional slot ve carried item'i taşır.

## Probe girdisi ve akışı

- Oyun modu: `SURVIVAL`
- Container: id `0`, player inventory menu
- Tıklanan slot: `-999` (menu dışı drag başlangıcı)
- Button: `0` (quick-craft header `0`, type `0`)
- Container input: `QUICK_CRAFT`, id `5`
- Gönderilen state-id: `0` (başlangıç menu state `1` değerinden eski)
- Gönderilen changed-slot map'i: boş
- Gönderilen carried hash'i: empty
- Rust probe item hazırlığı yapmadan dünya hazır olur olmaz tek
  `container_click` paketi yollar; ardından 2.5 saniyelik drain ile full
  content düzeltmesini toplar.

Beklenen invariant'lar şunlardır:

- Başlangıç content'i container `0`, state `1`, 46 slot ve carried için boş
  değer taşır.
- Rust bir `container_click` paketi yollar; packet state-id'si `0`, slot
  `-999`, button `0` ve input id `5`'tir.
- Server carried boş olduğu için quick-craft durumunu sıfırlar; slotlarda veya
  cursor'da item oluşmaz.
- Eski state-id nedeniyle tam düzeltme content'i state `2` ile 46 boş slot ve
  boş carried değer taşır.
- Bu senaryoda slot değişikliği, block ACK'i, effect veya projectile gözlemi
  beklenmez.

## Sınır

Bu dilim boş carried quick-craft başlangıcının iptalini ve no-op full resync'i
kapsar. Carried item ile gerçek drag dağıtımı, drag header/add/end sırası,
doğru state-id ile client prediction/hash map'i, double-click, stack sınırı,
creative menu ve özel container'lar sonraki P13 dilimleridir.
