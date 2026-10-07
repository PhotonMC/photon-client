# P08.as-player-snap — [server-conformance] Remote Player large-position snap

Bu kayıt, uzak bir Player konum güncellemesi Java'nın interpolation eşiğini
aştığında `ClientWorld` state'inin aynı packet uygulamasında hedefe snap
edilmesini kapsar. Kanıt sınıfı `server-conformance`'dır: Rust probe'ları
Mojang vanilla 26.2 dedicated server'a bağlanır. Java istemcisiyle aynı
girdide frame/state karşılaştırması yapılmadığı için bu kayıt
`client-parity` değildir.

## Java kaynak incelemesi

`client/multiplayer/ClientPacketListener.java:639-657`, entity position sync
packet'inde mevcut raw position ile packet hedefi arasındaki squared distance
`4096.0` değerini aşarsa `snapTo`, entity ticking durumunda ve eşik altında
kalırsa `moveOrInterpolateTo` seçer. Bu yol, konum ve body rotation'ı aynı
packet uygulamasında değerlendirir.

`client/multiplayer/ClientPacketListener.java:806-823`, teleport ve relative
position packet'lerinde önce `PositionMoveRotation.of(entity)` ile aktif
interpolation hedefini de hesaba katan mutlak state'i kurar. Uzaklık eşiği
aşılırsa doğrudan position/rotation setter'larını ve snap history düzeltmesini
uygular; eşik altında kalırsa interpolation handler'ına geçer.

`world/entity/Entity.java:1808-1814`, `snapTo` konumu ve body rotation'ı
ayarlayıp eski position/rotation state'ini yeni değerle eşitler. Bu, büyük
hareketin render history'sinde eski konuma doğru bir ara frame bırakmamasını
sağlar.

## Sözleşme

- Player entity type ID `156` için PositionSync hedefi ile mevcut raw position
  arasındaki squared distance `4096.0` değerinden büyükse interpolation başlatılmaz.
- Büyük güncellemede current position ve body rotation packet hedefi olur;
  previous position/body rotation aynı hedefe ayarlanır. Java `snapTo`, önceden
  aktif bir `InterpolationHandler` state'ini kendi içinde iptal etmez; aktif
  handler olmayan bu canlı senaryoda kalan adım sayısı `0` gözlenmiştir. Bu
  ayrım [P08.as.1 client-parity kaydında](client-world-entity-player-snap-parity-conformance.md)
  ölçülür.
- Eşik değeri aşılmayan ticking Player güncellemesi üç adımlı handler yolunda
  kalır. Eşik karşılaştırması strict `>` şeklindedir; tam `4096.0` değeri
  interpolation tarafındadır.
- Bu kayıt, uzak bir remote Player'ın server tarafından gönderilen mutlak
  PositionSync güncellemesini kapsar. Local instance authority, passenger
  konumlandırması, collision ve render extraction bu state'in dışındadır.

## Rust doğrulaması

`EntityPositionUpdateMode::PositionSync`, packet'in konum ve body rotation
uygulamasını tek world işlemi altında yürütür. Player için raw position
mesafesi eşiği kontrol edilir; snap yolunda current/previous position ve body
rotation birlikte hedefe yazılır. Önceden aktif Player interpolation state'i
Java `snapTo` semantiğiyle korunur; aktif state yoksa kalan adım `0` olur. Eşik
altı path'i P08.ar'daki üç tick testini korur.

`remote_player_position_sync_snaps_when_distance_exceeds_java_threshold`
unit testi `1.0` konumundan `66.0` konumuna geçişte snap history'sini ve
interpolation adımının sıfır olduğunu kontrol eder; ardından tam `64.0` blok
farkın interpolation başlattığını da doğrular.

## Vanilla server-conformance doğrulaması

**Tarih:** 2026-09-14

**Referans:** Java Edition 26.2 / protokol 776 / Java 25

**Sunucu:** Mojang vanilla 26.2 dedicated server, offline, seed `776`,
creative test modu, `view-distance=5`, `simulation-distance=5`,
`127.0.0.1:25568`

İki Rust probe bağlantısı aynı vanilla oturumunda açıldı. Sunucu konsolunda
observer ve mover önce sabit test koordinatlarına alındı; ardından mover
`x=90` konumuna taşındı. Vanilla bu command akışında
`ENTITY_POSITION_SYNC` gönderdi. Observer probe'u remote Player'ın ilk
`4096.0` üstü hedefinde snap state'ini kaydetti:

```text
cargo run --locked --offline --example world_remote_player_position_snap_probe -- 127.0.0.1:25568 P08SnapObs11 P08SnapMv11
tp P08SnapObs11 20 80 -36
tp P08SnapMv11 25 80 -36
tp P08SnapMv11 90 80 -36
```

Başarılı çıktı:

```text
connection=ok
play_login=true
player_loaded_sent=true
mover_movement_packets_sent=1
remote_player_position_snap_packet_kind=position_sync
remote_player_id=4243
remote_player_position_snap_before_position=(12.5, 68.0, -26.5)
remote_player_position_snap_target_position=(90.5, 80.0, -35.5)
remote_player_position_snap_before_rotation=(0.0, 0.0)
remote_player_position_snap_target_rotation=(0.0, 0.0)
remote_player_position_snap_applied_position=(90.5, 80.0, -35.5)
remote_player_position_snap_applied_rotation=(0.0, 0.0, 0.0)
remote_player_position_snap_previous_position=(90.5, 80.0, -35.5)
remote_player_position_snap_previous_rotation=(0.0, 0.0, 0.0)
remote_player_position_snap_interpolation_steps=0
entity_position_sync_packet_count=20
entity_teleport_packet_count=0
world_entity_count=70
world_chunk_count=157
received_packet_count=4939
```

Hedef ile önceki position arasındaki squared distance `6309.0`'dur. Rust
state'i packet uygulamasında hedefe ulaştı, previous history'yi aynı hedefe
eşitledi ve başlangıçta aktif handler olmadığı için kalan interpolation adımı
`0` kaldı. Önceden aktif handler'ın snap sonrasındaki Java state'i [P08.as.1
client-parity kaydında](client-world-entity-player-snap-parity-conformance.md)
ayrıca doğrulanır. Bu kayıt Java istemcisiyle frame parity iddiası değildir.
`TELEPORT_ENTITY` dalının canlı vanilla üretimi bu koşuda oluşmadı; kod ve
relative çözümü ayrı unit kapsamındadır.

## Açık kapsam

Teleport packet'inin canlı snap gözlemi, local player authority, passenger
history düzeltmesi, collision gate'i, hızlı ardışık packet codec base'i,
external movement delta'sı, generic LivingEntity türleri, render partial tick
ve Java istemcisiyle ek `client-parity` karşılaştırmaları açık kalır.
