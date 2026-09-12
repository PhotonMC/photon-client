# P13 — Survival inventory swap state-id correction sözleşmesi — [server-conformance] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, survival oyuncunun hotbar slotundaki stack'i player inventory menu
slotuna `SWAP` tıklamasıyla taşımasını ve client'ın eski container state-id
göndermesi durumunda server'ın authoritative full content düzeltmesini sınar.
Probe, normal istemcinin changed-slot hash tahminini taklit etmez; server'ın
swap ve state-id correction dallarını boş prediction alanlarıyla görünür kılar.

## Java referansı

`GameProtocols.java` serverbound dispatch tablosunda
`ServerboundContainerClickPacket` 18 numaralı play paketidir. Packet alanları
container id, state id, slot short'u, button byte'ı, `ContainerInput`, değişen
slotların kısa indeksli map'i ve `HashedStack` carried değeridir.
`ContainerInput.SWAP` kimliği `2`'dir; boş `HashedStack` optional codec'te tek
boolean `false` olarak taşınır.

`AbstractContainerMenu` içindeki `SWAP` dalı button `0..8` değerini oyuncunun
hotbar inventory index'i olarak kullanır. Target menu slotu boş ve source
stack dolu olduğunda source hotbar slotu temizlenir, stack target slota yazılır.
Player inventory menu'sünde hotbar index `0`, menu slotu `36`'ya karşılık gelir;
bu probe target olarak ilk main inventory slotu olan menu slotu `9`'u seçer.

`ServerGamePacketListenerImpl.handleContainerClick` packet state-id'sini menu
state-id'siyle karşılaştırır, geçerli click'i uygular, sonra state-id farklıysa
`broadcastFullState` çağırır. `ServerPlayer` full content yayınında menu
state-id'sini artırır; `ClientboundContainerSetContentPacket` container id,
state id, 46 optional slot ve carried item'i taşır.

## Probe girdisi ve akışı

- Oyun modu: `SURVIVAL`
- Sunucu hazırlığı, oyuncu oyuna girdikten sonra:
  `item replace entity <player> hotbar.0 with minecraft:stone 8`
- Container: id `0`, player inventory menu
- Target slot: `9` (main inventory slot 0)
- Button: `0` (hotbar inventory index 0)
- Container input: `SWAP`, id `2`
- Gönderilen state-id: `0` (hazırlık güncellemesinin state `2` değerinden eski)
- Gönderilen changed-slot map'i: boş
- Gönderilen carried hash: empty
- Item replacement komutu click'ten önce başarıyla tamamlanır; Rust probe dünya
  hazır olduktan sonra 12 saniye bekleyip tek `container_click` paketi yollar.
  Ardından 2.5 saniyelik drain ile düzeltme paketlerini toplar.

Beklenen invariant'lar şunlardır:

- Başlangıç content'i container `0`, state `1`, 46 slot ve slot 9, slot 36 ile
  carried için boş değer taşır.
- Console hazırlığı slot 36 için state `2`, count `8`, item registry `1`
  (stone), boş component patch'i taşır.
- Rust bir `container_click` paketi yollar; packet state-id'si `0`, slot `9`,
  button `0` ve input id `2`'dir.
- Server swap işlemini uygular; tam düzeltme content'i state `3` ile menu slot
  9'da count `8`, item `1`, slot 36'da boş ve carried'da boş değer taşır.
- Bu senaryoda block ACK'i, effect veya projectile gözlemi beklenmez.

## Sınır

Bu dilim server state-id uyuşmazlığı, survival player-inventory swap ve full
slot/cursor resync'ini kapsar. Doğru state-id ile normal client prediction/hash
map'i, pickup button varyantları, throw, drag, double-click, stack sınırı,
creative menu ve özel container'lar sonraki P13 dilimleridir.


## Vanilla server-conformance doğrulaması

**Tarih:** 2026-09-11

**Referans:** Java Edition 26.2 / protokol 776 / Java 25

**Sunucu:** Mojang vanilla 26.2 dedicated server, offline yerel koşu

**Sunucu SHA-1:** `823e2250d24b3ddac457a60c92a6a941943fcd6a`

## Çalıştırma

Sunucu `/tmp/voxel-vanilla-26.2-test` altında, `online-mode=false`, seed `776`,
view/simulation distance `5`, `gamemode=survival`, `force-gamemode=true`,
peaceful ve spawn protection `0` ayarlarıyla çalıştı. Her geçerli koşuda console
item replacement başarı satırı görülmeden sonuç kabul edilmedi.

```text
cargo run --offline --release --example survival_inventory_swap_state_correction_probe -- 127.0.0.1:25566 InvSwap1
cargo run --offline --release --example survival_inventory_swap_state_correction_probe -- 127.0.0.1:25566 InvSwap4

item replace entity <player> hotbar.0 with minecraft:stone 8
```

Probe, player inventory menu state `0` iken menu slotu 9'a button `0` ve
`SWAP` input id `2` ile tıklama yollar; state-id'yi `0`, changed-slot map'ini
ve carried hash'i boş tutar. Böylece server'ın hotbar slotu 36'dan main
inventory slotu 9'a taşıyıp full resync yapması ayırt edilir.

## Sonuçlar

| Gözlem | InvSwap1 | InvSwap4 |
| --- | ---: | ---: |
| play login / player loaded | `true / true` | `true / true` |
| container click packets sent | `1` | `1` |
| sent container state-id / input id | `0 / 2` | `0 / 2` |
| initial container content | `state=1, items=46, slot9=empty, slot36=empty, carried=empty` | `state=1, items=46, slot9=empty, slot36=empty, carried=empty` |
| preparation slot | `state=2, slot=36, count=8, item=1, components=[]` | `state=2, slot=36, count=8, item=1, components=[]` |
| authoritative full resync content | `state=3, items=46, slot9=count8/item1, slot36=empty, carried=empty` | `state=3, items=46, slot9=count8/item1, slot36=empty, carried=empty` |
| chunk packet / decoded / batch finished | `157 / 157 / 51` | `157 / 157 / 51` |
| received packet | `1009` | `1670` |

Her iki geçerli console item replacement işlemi başarıyla tamamlandı. Server
slot 36'daki 8 taş stack'ini menu slotu 9'a taşıdı ve eski state-id nedeniyle
state `3` tam content düzeltmesi gönderdi. Vanilla logunda protocol error,
action warning veya timeout gözlenmedi; iki oturumun karşılaştırılan inventory
çıktıları aynıdır.

İki ara koşul (`InvSwap2`, `InvSwap3`) item replacement komutunun probe'un 12
saniyelik click beklemesinin sonunda işlenmesi nedeniyle geçerli parity koşulu
olarak alınmadı. Bu koşullarda click boş slot üzerinde çalıştı; sonuçlar
authoritative hazırlık gerçekleşmeden yapılan timing yarışını gösterdi.

## Sonuç ve açık kapsam

İki bağımsız geçerli vanilla oturumu, `SWAP` state-id uyuşmazlığında taşımanın
uygulanıp slot 9'daki authoritative stack ile full resync edildiğini doğrular.
Sözleşme ayrıntıları [survival inventory swap state correction contract
belgesindedir](#sözleşme).

Doğru state-id ile client'ın hashed changed-slot/cursor tahmini, pickup button
varyantları, throw, drag, double-click, stack sınırı ve özel menu akışları açık
kalır.
