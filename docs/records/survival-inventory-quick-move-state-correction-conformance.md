# P13 — Survival inventory quick-move state-id correction sözleşmesi — [server-conformance] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, survival oyuncunun hotbar slotundaki stack'e shift-click karşılığı
olan `QUICK_MOVE` tıklamasını göndermesini ve eski container state-id ile gelen
client tahmini karşısında server'ın taşıma sonucunu full content düzeltmesiyle
bildirmesini sınar. Probe, normal istemcinin changed-slot tahminini taklit
etmez; boş prediction alanlarıyla server'ın authoritative quick-move ve
correction yolunu görünür kılar.

## Java referansı

`GameProtocols.java` serverbound dispatch tablosunda
`ServerboundContainerClickPacket` 18 numaralı play paketidir. Packet alanları
container id, state id, slot short'u, button byte'ı, `ContainerInput`, değişen
slotların kısa indeksli map'i ve `HashedStack` carried değeridir.
`ContainerInput.QUICK_MOVE` kimliği `1`'dir; boş `HashedStack` optional codec'te
tek boolean `false` olarak taşınır.

`ServerGamePacketListenerImpl.handleContainerClick` aynı container id ve
geçerli slot koşullarında state-id uyuşmazlığı olsa bile
`AbstractContainerMenu.clicked` çağrısını çalıştırır. `ContainerMenu` slot 36
hotbar stack'i için `quickMoveStack` çağrısında hedef aralığı menu slotları
`9..35` olarak seçer; ilk boş main inventory slotu menu slotu `9`'dur. Stack
slot 36'dan slot 9'a taşındıktan sonra eski state-id nedeniyle
`broadcastFullState` çalışır ve `ServerPlayer` artmış state-id ile 46 slotu ve
boş carried değerini `ClientboundContainerSetContentPacket` olarak yollar.

## Probe girdisi ve akışı

- Oyun modu: `SURVIVAL`
- Sunucu hazırlığı, oyuncu oyuna girdikten sonra:
  `item replace entity <player> hotbar.0 with minecraft:stone 8`
- Container: id `0`, player inventory menu
- Tıklanan slot: `36` (hotbar slot 0)
- Button: `0`
- Container input: `QUICK_MOVE`, id `1`
- Gönderilen state-id: `0` (hazırlık güncellemesinin state `2` değerinden eski)
- Gönderilen changed-slot map'i: boş
- Gönderilen carried hash: empty
- Rust probe, login/chunk/player-loaded akışından sonra 12 saniye bekler ve
  tek `container_click` paketi yollar; ardından 2.5 saniyelik drain ile
  düzeltme paketlerini toplar.

Beklenen invariant'lar şunlardır:

- Başlangıç content'i container `0`, state `1`, 46 slot, menu slot 9 ve slot 36
  için boş değerler taşır.
- Console hazırlığı slot 36 için state `2`, count `8`, item registry `1`
  (stone), boş component patch'i taşır.
- Rust bir `container_click` paketi yollar, input id `1` ve packet state-id'si
  `0`'dır.
- Server quick-move işlemini uygular; tam düzeltme content'i state `3` ile
  menu slot 9'da count `8`, item `1`, slot 36'da boş ve carried'da boş değer
  taşır.
- Bu senaryoda block ACK'i, effect veya projectile gözlemi beklenmez.

## Sınır

Bu dilim server state-id uyuşmazlığı, survival quick-move ve full inventory
resync'ini kapsar. Doğru state-id ile normal client prediction/hash map'i,
primary/secondary pickup, swap, throw, drag, double-click, stack sınırı,
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
cargo run --offline --release --example survival_inventory_quick_move_state_correction_probe -- 127.0.0.1:25566 InvQuick1
cargo run --offline --release --example survival_inventory_quick_move_state_correction_probe -- 127.0.0.1:25566 InvQuick2

item replace entity <player> hotbar.0 with minecraft:stone 8
```

Probe, player inventory menu state `0` iken slot 36'ya button `0` ve
`QUICK_MOVE` input id `1` ile tıklama yollar; state-id'yi `0`, changed-slot
map'ini ve carried hash'i boş tutar. Böylece server'ın stack'i ilk boş main
inventory slotu olan menu slotu 9'a taşıyıp full resync yapması ayırt edilir.

## Sonuçlar

| Gözlem | InvQuick1 | InvQuick2 |
| --- | ---: | ---: |
| play login / player loaded | `true / true` | `true / true` |
| container click packets sent | `1` | `1` |
| sent container state-id / input id | `0 / 1` | `0 / 1` |
| initial container content | `state=1, items=46, slot9=empty, slot36=empty, carried=empty` | `state=1, items=46, slot9=empty, slot36=empty, carried=empty` |
| preparation slot | `state=2, slot=36, count=8, item=1, components=[]` | `state=2, slot=36, count=8, item=1, components=[]` |
| authoritative full resync content | `state=3, items=46, slot9=count8/item1, slot36=empty, carried=empty` | `state=3, items=46, slot9=count8/item1, slot36=empty, carried=empty` |
| chunk packet / decoded / batch finished | `157 / 157 / 51` | `157 / 157 / 51` |
| received packet | `1216` | `1512` |

Her iki console item replacement işlemi başarıyla tamamlandı. Server slot 36'daki
8 taş stack'ini slot 9'a taşıdı ve eski state-id nedeniyle state `3` tam content
düzeltmesi gönderdi. Vanilla logunda protocol error, action warning veya
timeout gözlenmedi; iki oturumun karşılaştırılan inventory çıktıları aynıdır.

## Sonuç ve açık kapsam

İki bağımsız vanilla oturumu, `QUICK_MOVE` state-id uyuşmazlığında shift-click
taşımasının uygulanıp slot 9'daki authoritative stack ile full resync edildiğini
doğrular. Sözleşme ayrıntıları [survival inventory quick-move state correction
contract belgesindedir](#sözleşme).

Doğru state-id ile client'ın hashed changed-slot/cursor tahmini, primary ve
secondary pickup, swap, throw, drag, double-click, stack sınırı ve özel menu
akışları açık kalır.
