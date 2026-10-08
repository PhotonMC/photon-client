# P13 — Survival inventory secondary pickup state-id correction sözleşmesi — [server-conformance] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, survival oyuncunun player inventory hotbar stack'inin yarısını
secondary pickup tıklamasıyla cursor'a almasını ve client'ın eski container
state-id göndermesi durumunda server'ın authoritative full content düzeltmesini
sınar. Probe, normal istemcinin changed-slot hash tahminini taklit etmez;
server'ın secondary pickup ve state-id correction dallarını boş prediction
alanlarıyla görünür kılar.

## Java referansı

`GameProtocols.java` serverbound dispatch tablosunda
`ServerboundContainerClickPacket` 18 numaralı play paketidir. Packet alanları
container id, state id, slot short'u, button byte'ı, `ContainerInput`, değişen
slotların kısa indeksli map'i ve `HashedStack` carried değeridir.
`ContainerInput.PICKUP` kimliği `0`'dır; secondary davranış packet input'ı
değiştirmeden button `1` ile seçilir. Boş `HashedStack` optional codec'te tek
boolean `false` olarak taşınır.

`AbstractContainerMenu` pickup dalında button `0` primary, button `1`
secondary olarak ayrılır. Carried boşken secondary pickup miktarı
`(clicked count + 1) / 2`'dir; count `8` stack için 4 item cursor'a alınır ve
slotta 4 item bırakılır. `ServerGamePacketListenerImpl.handleContainerClick`
packet state-id'sini menu state-id'siyle karşılaştırır, geçerli click'i uygular,
sonra state-id farklıysa `broadcastFullState` çağırır. `ServerPlayer` full
content yayınında menu state-id'sini artırır; `ClientboundContainerSetContentPacket`
container id, state id, 46 optional slot ve carried item'i taşır.

## Probe girdisi ve akışı

- Oyun modu: `SURVIVAL`
- Sunucu hazırlığı, oyuncu oyuna girdikten sonra:
  `item replace entity <player> hotbar.0 with minecraft:stone 8`
- Container: id `0`, player inventory menu
- Tıklanan slot: `36` (hotbar slot 0)
- Button: `1` (secondary pickup)
- Container input: `PICKUP`, id `0`
- Gönderilen state-id: `0` (hazırlık güncellemesinin state `2` değerinden eski)
- Gönderilen changed-slot map'i: boş
- Gönderilen carried hash: empty
- Item replacement komutu click'ten önce başarıyla tamamlanır; Rust probe dünya
  hazır olduktan sonra 20 saniye bekleyip tek `container_click` paketi yollar.
  Ardından 2.5 saniyelik drain ile düzeltme paketlerini toplar.

Beklenen invariant'lar şunlardır:

- Başlangıç content'i container `0`, state `1`, 46 slot ve slot 36 ile carried
  için boş değer taşır.
- Console hazırlığı slot 36 için state `2`, count `8`, item registry `1`
  (stone), boş component patch'i taşır.
- Rust bir `container_click` paketi yollar; packet state-id'si `0`, slot `36`,
  button `1` ve input id `0`'dır.
- Server secondary pickup işlemini uygular; tam düzeltme content'i state `3`
  ile slot 36'da count `4`, item `1`, carried'da count `4`, item `1` taşır.
- Bu senaryoda block ACK'i, effect veya projectile gözlemi beklenmez.

## Sınır

Bu dilim server state-id uyuşmazlığı, survival secondary pickup ve full
slot/cursor resync'ini kapsar. Doğru state-id ile normal client prediction/hash
map'i, primary pickup'ın diğer stack koşulları, throw, drag, double-click, stack
sınırı, creative menu ve özel container'lar sonraki P13 dilimleridir.


## Vanilla server-conformance doğrulaması

**Tarih:** 2026-09-11

**Referans:** Java Edition 26.2 / protokol 776 / Java 25

**Sunucu:** Mojang vanilla 26.2 dedicated server, offline yerel koşu

**Sunucu SHA-1:** `823e2250d24b3ddac457a60c92a6a941943fcd6a`

## Çalıştırma

Sunucu `/tmp/photon-vanilla-26.2-test` altında, `online-mode=false`, seed `776`,
view/simulation distance `5`, `gamemode=survival`, `force-gamemode=true`,
peaceful ve spawn protection `0` ayarlarıyla çalıştı. Her geçerli koşuda console
item replacement başarı satırı görülmeden sonuç kabul edilmedi.

```text
cargo run --offline --release --example survival_inventory_secondary_pickup_state_correction_probe -- 127.0.0.1:25566 InvHalf1
cargo run --offline --release --example survival_inventory_secondary_pickup_state_correction_probe -- 127.0.0.1:25566 InvHalf4

item replace entity <player> hotbar.0 with minecraft:stone 8
```

Probe, player inventory menu state `0` iken slot 36'ya button `1` ve `PICKUP`
input id `0` ile tıklama yollar; state-id'yi `0`, changed-slot map'ini ve
carried hash'i boş tutar. Böylece server'ın 8 taşlık stack'in 4 item'ini
cursor'a alıp slotta 4 bırakması ve full resync yapması ayırt edilir.

## Sonuçlar

| Gözlem | InvHalf1 | InvHalf4 |
| --- | ---: | ---: |
| play login / player loaded | `true / true` | `true / true` |
| container click packets sent | `1` | `1` |
| sent container state-id / button / input id | `0 / 1 / 0` | `0 / 1 / 0` |
| initial container content | `state=1, items=46, slot36=empty, carried=empty` | `state=1, items=46, slot36=empty, carried=empty` |
| preparation slot | `state=2, slot=36, count=8, item=1, components=[]` | `state=2, slot=36, count=8, item=1, components=[]` |
| authoritative full resync content | `state=3, items=46, slot36=count4/item1, carried=count4/item1` | `state=3, items=46, slot36=count4/item1, carried=count4/item1` |
| chunk packet / decoded / batch finished | `157 / 157 / 51` | `157 / 157 / 51` |
| received packet | `1116` | `1524` |

Her iki geçerli console item replacement işlemi başarıyla tamamlandı. Server
8 taşlık hotbar stack'inin 4'ünü carried item yaptı, slot 36'da 4 bıraktı ve
eski state-id nedeniyle state `3` tam content düzeltmesi gönderdi. Vanilla
logunda protocol error, action warning veya timeout gözlenmedi; iki oturumun
karşılaştırılan inventory çıktıları aynıdır.

İki ara koşul (`InvHalf2`, `InvHalf3`) item replacement komutunun probe click
beklemesine çok yakın işlenmesi nedeniyle geçerli parity koşulu olarak alınmadı.
Bu koşullarda item değişiminin ve click düzeltmesinin state sırası karıştı;
sonuçlar hazırlık zamanlamasının kontrol edilmesi gerektiğini gösterdi.

## Sonuç ve açık kapsam

İki bağımsız geçerli vanilla oturumu, secondary pickup state-id uyuşmazlığında
yarım-stack işleminin uygulanıp slot ve cursor içeriğinin authoritative full
resync ile düzeltildiğini doğrular. Sözleşme ayrıntıları [survival inventory
secondary pickup state correction contract belgesindedir](#sözleşme).

Doğru state-id ile client'ın hashed changed-slot/cursor tahmini, diğer pickup
stack koşulları, throw, drag, double-click, stack sınırı ve özel menu akışları
açık kalır.
