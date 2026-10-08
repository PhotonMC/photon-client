# P13 — Survival inventory throw-all state-id correction sözleşmesi — [server-conformance] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, survival oyuncunun player inventory slotundaki stack'in tamamını
düşürmesini ve client'ın eski container state-id göndermesi durumunda server'ın
authoritative full content düzeltmesini sınar. Probe, normal istemcinin
changed-slot hash tahminini taklit etmez; server'ın throw-all ve state-id
correction dallarını boş prediction alanlarıyla görünür kılar.

## Java referansı

`GameProtocols.java` serverbound dispatch tablosunda
`ServerboundContainerClickPacket` 18 numaralı play paketidir. Packet alanları
container id, state id, slot short'u, button byte'ı, `ContainerInput`, değişen
slotların kısa indeksli map'i ve `HashedStack` carried değeridir.
`ContainerInput.THROW` kimliği `4`'tür; boş `HashedStack` optional codec'te tek
boolean `false` olarak taşınır.

`AbstractContainerMenu` throw dalında carried boşken button `1` için miktarı
slotun mevcut stack count'u olarak seçer, bu stack'i `player.drop` yoluna verir
ve aynı item slotta kaldığı sürece döngüyü kontrol eder. Count `8` stack bu
senaryoda tek drop ile tamamen boşalır. Button `0` ile tek item düşürme ayrı
bir dilimde doğrulanmıştır.

`ServerGamePacketListenerImpl.handleContainerClick` packet state-id'sini menu
state-id'siyle karşılaştırır, geçerli click'i ve drop yan etkisini uygular,
sonra state-id farklıysa `broadcastFullState` çağırır. `ServerPlayer` full
content yayınında menu state-id'sini artırır; `ClientboundContainerSetContentPacket`
container id, state id, 46 optional slot ve carried item'i taşır. Düşen item için
ayrıca clientbound `add_entity` akışı gözlenir; bu probe item metadata paketini
decode etmez.

## Probe girdisi ve akışı

- Oyun modu: `SURVIVAL`
- Sunucu hazırlığı, oyuncu oyuna girdikten sonra:
  `item replace entity <player> hotbar.0 with minecraft:stone 8`
- Container: id `0`, player inventory menu
- Tıklanan slot: `36` (hotbar slot 0)
- Button: `1` (stack'in tamamı)
- Container input: `THROW`, id `4`
- Gönderilen state-id: `0` (hazırlık güncellemesinin state `2` değerinden eski)
- Gönderilen changed-slot map'i: boş
- Gönderilen carried hash'i: empty
- Item replacement komutu click'ten önce başarıyla tamamlanır; Rust probe dünya
  hazır olduktan sonra 30 saniye bekleyip tek `container_click` paketi yollar.
  Ardından 2.5 saniyelik drain ile slot, content ve add-entity paketlerini
  toplar.

Beklenen invariant'lar şunlardır:

- Başlangıç content'i container `0`, state `1`, 46 slot ve slot 36 ile carried
  için boş değer taşır.
- Console hazırlığı slot 36 için state `2`, count `8`, item registry `1`
  (stone), boş component patch'i taşır.
- Rust bir `container_click` paketi yollar; packet state-id'si `0`, slot `36`,
  button `1` ve input id `4`'tür.
- Server throw-all işlemini uygular; tam düzeltme content'i state `3` ile slot
  36'da boş, carried'da boş değer taşır.
- Düşen stack için player konumuna yakın bir `add_entity` gözlemi oluşur.
- Bu senaryoda block ACK'i veya effect gözlemi beklenmez.

## Sınır

Bu dilim server state-id uyuşmazlığı, survival throw-all ve full slot/cursor
resync'ini kapsar. Doğru state-id ile client'ın hashed changed-slot/cursor
tahmini, drag, double-click, stack sınırı, creative menu ve özel container'lar
sonraki P13 dilimleridir.


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
cargo run --offline --release --example survival_inventory_throw_all_state_correction_probe -- 127.0.0.1:25566 InvThrowAll3
cargo run --offline --release --example survival_inventory_throw_all_state_correction_probe -- 127.0.0.1:25566 InvThrowAll4

item replace entity <player> hotbar.0 with minecraft:stone 8
```

Probe, player inventory menu state `0` iken slot 36'ya button `1` ve `THROW`
input id `4` ile tıklama yollar; state-id'yi `0`, changed-slot map'ini ve
carried hash'i boş tutar. Böylece server'ın 8 taşlık stack'i tamamen düşürüp
slotu boşaltması ve full resync yapması ayırt edilir.

## Sonuçlar

| Gözlem | InvThrowAll3 | InvThrowAll4 |
| --- | ---: | ---: |
| play login / player loaded | `true / true` | `true / true` |
| container click packets sent | `1` | `1` |
| sent container state-id / button / input id | `0 / 1 / 4` | `0 / 1 / 4` |
| initial container content | `state=1, items=46, slot36=empty, carried=empty` | `state=1, items=46, slot36=empty, carried=empty` |
| preparation slot | `state=2, slot=36, count=8, item=1, components=[]` | `state=2, slot=36, count=8, item=1, components=[]` |
| authoritative full resync content | `state=3, items=46, slot36=empty, carried=empty` | `state=3, items=46, slot36=empty, carried=empty` |
| dropped-item add entity | `1 × entity_type_id=71 near player` | `1 × entity_type_id=71 near player` |
| chunk packet / decoded / batch finished | `157 / 157 / 51` | `157 / 157 / 51` |
| received packet | `4743` | `3837` |

Her iki geçerli console item replacement işlemi başarıyla tamamlandı. Server
8 taşlık stack'in tamamını tek drop olarak düşürdü, slot 36'yı boşalttı ve eski
state-id nedeniyle state `3` tam content düzeltmesi gönderdi. Her iki oturumda
oyuncu konumuna yakın tek `entity_type_id=71` add-entity gözlemi alındı. Probe
add-entity metadata'sındaki gerçek item stack component'lerini decode etmedi;
bu birleşik kayıttaki inventory sonucu slot content üzerinden doğrulandı.

Vanilla logunda protocol error, action warning veya timeout gözlenmedi; iki
oturumun karşılaştırılan inventory çıktıları aynıdır.

## Sonuç ve açık kapsam

İki bağımsız geçerli vanilla oturumu, throw-all state-id uyuşmazlığında tüm
stack drop işleminin uygulanıp slot ve cursor içeriğinin authoritative full
resync ile düzeltildiğini doğrular. Sözleşme ayrıntıları [survival inventory
throw-all state correction contract belgesindedir](#sözleşme).

Doğru state-id ile client'ın hashed changed-slot/cursor tahmini, drag,
double-click, stack sınırı ve özel menu akışları açık kalır.
