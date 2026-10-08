# P13 — Survival inventory pickup state-id correction sözleşmesi — [server-conformance] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, survival oyuncunun hotbar slotundaki stack'i pickup tıklamasıyla
almasını ve client'ın eski container state-id göndermesi durumunda server'ın
authoritative slot/cursor tam düzeltmesini sınar. Probe, normal istemcinin
öngörüsünü temsil etmek için değil, server state-id uyuşmazlığı ve correction
dalını kontrollü olarak görünür kılmak için boş `changedSlots` ve boş carried
hash gönderir.

## Java referansı

`GameProtocols.java` serverbound dispatch tablosunda
`ServerboundContainerClickPacket` 18 numaralı play paketidir. Packet alanları
sırasıyla container id, state id, slot short'u, button byte'ı, `ContainerInput`,
değişen slotların kısa indeksli map'i ve `HashedStack` carried değeridir.
`ContainerInput.PICKUP` kimliği `0`'dır; boş `HashedStack` optional codec'te tek
boolean `false` olarak taşınır.

`ServerGamePacketListenerImpl.handleContainerClick` aynı container id ve
geçerli slot koşullarında state-id'yi mevcut menu state-id ile karşılaştırır,
ardından `AbstractContainerMenu.clicked` çağrısını yine çalıştırır. Pickup
button `0`, carried boşken slot 36'daki stack'in tamamını carried'a alır ve
slotu boşaltır. Paket state-id'si farklıysa listener değişiklikleri uyguladıktan
sonra `broadcastFullState` çağırır; `ServerPlayer` bu tam yayında menu state-id
değerini artırarak tüm 46 slotu ve carried item'i
`ClientboundContainerSetContentPacket` ile yollar.

## Probe girdisi ve akışı

- Oyun modu: `SURVIVAL`
- Sunucu hazırlığı, oyuncu oyuna girdikten sonra:
  `item replace entity <player> hotbar.0 with minecraft:stone 8`
- Container: id `0`, player inventory menu
- Tıklanan slot: `36` (hotbar slot 0)
- Button: `0` (primary pickup)
- Container input: `PICKUP`, id `0`
- Gönderilen state-id: `0` (hazırlık güncellemesinin state `2` değerinden eski)
- Gönderilen changed-slot map'i: boş
- Gönderilen carried hash: empty
- Rust probe, login/chunk/player-loaded akışından sonra 12 saniye bekler ve
  tek `container_click` paketi yollar; ardından 2.5 saniyelik drain ile
  düzeltme paketlerini toplar.

Beklenen invariant'lar şunlardır:

- Başlangıç content'i container `0`, state `1`, 46 slot ve boş slot 36 taşır.
- Console hazırlığı slot 36 için state `2`, count `8`, item registry `1`
  (stone), boş component patch'i taşır.
- Rust bir `container_click` paketi yollar ve packet state-id'si `0`'dır.
- Server tıklamayı uygular; tam düzeltme content'i state `3` ile 46 slot taşır,
  slot 36 boştur ve carried item count `8`, registry `1`'dir.
- Bu senaryoda block ACK'i, effect veya projectile gözlemi beklenmez.

## Sınır

Bu dilim server state-id uyuşmazlığı, primary pickup ve full slot/cursor
resync'ini kapsar. Doğru state-id ile normal client prediction/hash map'i,
secondary pickup, quick-move, swap, throw, drag, double-click, stack sınırı,
creative menu ve özel container'lar sonraki P13 dilimleridir.


## Vanilla server-conformance doğrulaması

**Tarih:** 2026-09-11

**Referans:** Java Edition 26.2 / protokol 776 / Java 25

**Sunucu:** Mojang vanilla 26.2 dedicated server, offline yerel koşu

**Sunucu SHA-1:** `823e2250d24b3ddac457a60c92a6a941943fcd6a`

## Çalıştırma

Sunucu `/tmp/photon-vanilla-26.2-test` altında, `online-mode=false`, seed `776`,
view/simulation distance `5`, `gamemode=survival`, `force-gamemode=true`,
peaceful ve spawn protection `0` ayarlarıyla çalıştı. Her koşuda console item
replacement başarı satırı görülmeden sonuç geçerli sayılmadı.

```text
cargo run --offline --release --example survival_inventory_pickup_state_correction_probe -- 127.0.0.1:25566 InvCorr1
cargo run --offline --release --example survival_inventory_pickup_state_correction_probe -- 127.0.0.1:25566 InvCorr2

item replace entity <player> hotbar.0 with minecraft:stone 8
```

Probe, player inventory menu state `0` iken slot 36'ya primary pickup yollar;
state-id'yi kasıtlı olarak `0`, changed-slot map'ini ve carried hash'i boş
tutar. Bu kontrollü eski client tahmini server'ın tıklamayı uyguladıktan sonra
full resync dalına girdiğini ayırt eder.

## Sonuçlar

| Gözlem | InvCorr1 | InvCorr2 |
| --- | ---: | ---: |
| play login / player loaded | `true / true` | `true / true` |
| container click packets sent | `1` | `1` |
| sent container state-id | `0` | `0` |
| initial container content | `state=1, items=46, slot36=empty, carried=empty` | `state=1, items=46, slot36=empty, carried=empty` |
| preparation slot | `state=2, slot=36, count=8, item=1, components=[]` | `state=2, slot=36, count=8, item=1, components=[]` |
| authoritative full resync content | `state=3, items=46, slot36=empty, carried=count8/item1` | `state=3, items=46, slot36=empty, carried=count8/item1` |
| chunk packet / decoded / batch finished | `157 / 157 / 51` | `157 / 157 / 51` |
| received packet | `3064` | `2772` |

Her iki console item replacement işlemi başarıyla tamamlandı. Server slot 36'yı
boşaltıp 8 taşlık carried stack oluşturdu ve eski state-id nedeniyle state `3`
tam content düzeltmesi gönderdi. Vanilla logunda protocol error, action warning
veya timeout gözlenmedi; iki oturumun karşılaştırılan inventory çıktıları
aynıdır.

## Sonuç ve açık kapsam

İki bağımsız vanilla oturumu, `container_click` state-id uyuşmazlığında pickup
işleminin uygulanıp ardından slot ve cursor içeriğinin authoritative full
resync ile düzeltildiğini doğrular. Sözleşme ayrıntıları [survival inventory
pickup state correction contract belgesindedir](#sözleşme).

Doğru state-id ile client'ın hashed changed-slot/cursor tahmini, secondary
pickup, quick-move, swap, throw, drag, double-click, stack sınırı ve özel menu
akışları açık kalır.
