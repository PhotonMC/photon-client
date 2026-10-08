# P13 — Survival inventory ahead state-id correction sözleşmesi — [server-conformance] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, player inventory click packet'i menu state-id'sinden büyük bir
state-id taşıdığında vanilla server'ın click'i yine uygulayıp authoritative
full content düzeltmesi göndermesini sınar. Önceki P13 probe'ları state `0` ile
menu state'inden eski packet'leri kullandı; bu probe mismatch'in diğer yönünü
izole eder.

## Java referansı

`GameProtocols.java` serverbound dispatch tablosunda
`ServerboundContainerClickPacket` 18 numaralı play paketidir. Packet alanları
container id, state id, slot short'u, button byte'ı, `ContainerInput`, değişen
slotların kısa indeksli map'i ve `HashedStack` carried değeridir.

`ServerGamePacketListenerImpl.handleContainerClick`, packet state-id'sini menu
state-id'siyle eşitlik üzerinden karşılaştırır. Eşit değilse remote update'leri
geçici olarak bastırır, `containerMenu.clicked` ile click davranışını uygular,
packet'in değişen slot/cursor tahminini kaydeder ve `broadcastFullState` ile
authoritative içeriği yollar. Bu dal state-id'nin menu değerinden küçük veya
büyük olmasını ayırmaz.

`ContainerInput.PICKUP` kimliği `0`'dır. Carried boşken button `0` ile menu slotu
`36`'daki stack alınır; slot boşalır ve stack cursor'a taşınır. `ServerPlayer`
full content yayınında menu state-id'sini artırır; `ClientboundContainerSetContentPacket`
container id, state id, 46 optional slot ve carried item'i taşır.

## Probe girdisi ve akışı

- Oyun modu: `SURVIVAL`
- Container: id `0`, player inventory menu
- Hazırlık: hotbar index `0` / menu slotu `36` içine `minecraft:stone 8`
- Stone item registry ID: `1`
- Gönderilen state-id: `99` (hazırlık sonrası menu state `2` değerinden büyük)
- Slot: `36`
- Button: `0`
- Container input: `PICKUP`, id `0`
- Gönderilen changed-slot map'i: boş
- Gönderilen carried hash'i: empty

Probe, dünya hazır olduktan ve item hazırlığı için bekledikten sonra tek
`container_click` paketi yollar; ardından full content düzeltmesini toplar.

Beklenen invariant'lar şunlardır:

- Başlangıç content'i container `0`, state `1`, 46 slot ve carried için boş
  değer taşır.
- Console item hazırlığı menu slotu `36` için state `2`, count `8` ve item ID
  `1` slot gözlemi üretir.
- State-id `99`, menu state `2` ile eşit olmadığı için mismatch branch'i seçilir;
  server pickup davranışını yine uygular.
- Authoritative correction state `3` ile slot `36` boş, carried count `8` ve
  item ID `1` taşır.
- Rust probe bir `container_click` paketi gönderir ve gönderilen state-id `99`
  olarak gözlenir.

## Sınır

Bu dilim state-id'nin menu değerinden büyük olduğu tekli survival pickup
durumunu kapsar. Non-empty changed-slot map'i, non-empty carried hash'i, aynı
pakette diğer input/slot varyantları, doğru state-id ile client prediction ve
özel container'lar sonraki P13 dilimleridir.


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
cargo run --offline --release --example survival_inventory_ahead_state_correction_probe -- 127.0.0.1:25566 InvAhead2
cargo run --offline --release --example survival_inventory_ahead_state_correction_probe -- 127.0.0.1:25566 InvAhead3
```

Probe, item hazırlığı sonrası menu state `2` iken state-id `99`, slot `36`,
button `0`, `PICKUP` input id `0`, boş changed-slot map'i ve empty carried
hash'i ile tek click gönderdi.

## Sonuçlar

| Gözlem | InvAhead2 | InvAhead3 |
| --- | ---: | ---: |
| play login / player loaded | `true / true` | `true / true` |
| container click packets sent | `1` | `1` |
| gönderilen state-id / slot / button / input id | `99 / 36 / 0 / 0` | `99 / 36 / 0 / 0` |
| initial container content | `state=1, items=46, all empty, carried=empty` | `state=1, items=46, all empty, carried=empty` |
| item hazırlık slot observation | `state=2, slot36=count8,item1` | `state=2, slot36=count8,item1` |
| authoritative full resync | `state=3, slot36=empty, carried=count8,item1` | `state=3, slot36=empty, carried=count8,item1` |
| chunk packet / decoded / batch finished | `157 / 157 / 51` | `157 / 157 / 51` |
| received packet | `2829` | `1566` |

Her iki vanilla oturumunda state-id `99`, menu state `2` ile eşleşmediği halde
pickup click'i uygulandı ve state `3` full content düzeltmesiyle slot boşalıp
8 taş cursor'a taşındı. Server logunda koşular için protocol error veya timeout
gözlenmedi.

## Sonuç ve açık kapsam

İki bağımsız Mojang vanilla 26.2 oturumu, menu state-id'sinden büyük bir
serverbound container state-id'nin de click'i uygulayıp authoritative full
resync'e girdiğini doğrular. Sözleşme ayrıntıları [survival inventory ahead
state-id correction contract belgesindedir](#sözleşme).

Non-empty changed-slot map'i, non-empty carried hash'i, diğer input/slot
varyantları, doğru state-id ile client'ın hashed prediction'ı ve özel menu
akışları açık kalır.
