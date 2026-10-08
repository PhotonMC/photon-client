# P13 — Survival inventory empty drag cancellation state-id correction sözleşmesi — [server-conformance] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

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


## Vanilla server-conformance doğrulaması

**Tarih:** 2026-09-11

**Referans:** Java Edition 26.2 / protokol 776 / Java 25

**Sunucu:** Mojang vanilla 26.2 dedicated server, offline yerel koşu

**Sunucu SHA-1:** `823e2250d24b3ddac457a60c92a6a941943fcd6a`

## Çalıştırma

Sunucu `/tmp/photon-vanilla-26.2-test` altında, `online-mode=false`, seed `776`,
view/simulation distance `5`, `gamemode=survival`, `force-gamemode=true`,
peaceful ve spawn protection `0` ayarlarıyla çalıştı. Bu senaryoda item hazırlığı
yapılmadı; başlangıç player inventory'sinin boş content'i doğrudan kullanıldı.

```text
cargo run --offline --release --example survival_inventory_drag_empty_cancel_state_correction_probe -- 127.0.0.1:25566 InvDrag1
cargo run --offline --release --example survival_inventory_drag_empty_cancel_state_correction_probe -- 127.0.0.1:25566 InvDrag2
```

Probe, player inventory menu state `1` iken slot `-999`, button `0` ve
`QUICK_CRAFT` input id `5` ile drag başlangıcı yollar; state-id'yi `0`,
changed-slot map'ini ve carried hash'i boş tutar. Carried boş olduğu için
vanilla quick-craft durumunu sıfırlar ve stale state nedeniyle no-op full resync
yapar.

## Sonuçlar

| Gözlem | InvDrag1 | InvDrag2 |
| --- | ---: | ---: |
| play login / player loaded | `true / true` | `true / true` |
| container click packets sent | `1` | `1` |
| sent container state-id / slot / button / input id | `0 / -999 / 0 / 5` | `0 / -999 / 0 / 5` |
| initial container content | `state=1, items=46, all empty, carried=empty` | `state=1, items=46, all empty, carried=empty` |
| container slot observations | `0` | `0` |
| authoritative full resync content | `state=2, items=46, all empty, carried=empty` | `state=2, items=46, all empty, carried=empty` |
| chunk packet / decoded / batch finished | `157 / 157 / 51` | `157 / 157 / 51` |
| received packet | `396` | `360` |

Her iki vanilla oturumunda quick-craft başlangıcı boş carried nedeniyle iptal
edildi; slot veya cursor item'i oluşmadı ve stale state-id nedeniyle state `2`
full content düzeltmesi gönderildi. Vanilla logunda protocol error, action
warning veya timeout gözlenmedi; karşılaştırılan inventory çıktıları aynıdır.

## Sonuç ve açık kapsam

İki bağımsız vanilla oturumu, empty-carried `QUICK_CRAFT` başlangıcının no-op
kalıp authoritative full resync ile state `2`'ye düzeltildiğini doğrular.
Sözleşme ayrıntıları [survival inventory empty drag cancellation state correction
contract belgesindedir](#sözleşme).

Carried item ile gerçek drag dağıtımı, drag header/add/end sırası, doğru state-id
ile client'ın hashed prediction'ı, double-click, stack sınırı ve özel menu
akışları açık kalır.
