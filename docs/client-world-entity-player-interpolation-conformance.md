# P08.ar-player-interpolation — [server-conformance] Remote Player position interpolation

Bu kayıt, vanilla sunucusunun remote Player için gönderdiği küçük hareket
paketinin `ClientWorld` içinde doğrudan konum değişimi yerine üç world tick'lik
hedef state'ine çevrilmesini kapsar. Kanıt sınıfı `server-conformance`'dır:
Rust probe'ları Mojang vanilla 26.2 dedicated server'a bağlanır. Java
istemcisiyle aynı girdide frame/state karşılaştırması yapılmadığı için bu kayıt
`client-parity` değildir.

## Java kaynak incelemesi

`world/entity/InterpolationHandler.java:10-115`, hedef position ve y/x
rotation'ı, kalan interpolation adımlarını ve son interpolation tick'inin
position/rotation bilgisini ayrı state'te tutar. Varsayılan hedef uzunluğu `3`
'tür. Her tick'te kalan adımın tersi alpha olarak kullanılır; position double
`Mth.lerp`, yaw wrapped `Mth.rotLerp`, pitch linear lerp ile current state'ten
hedefe yaklaştırılır.

`world/entity/LivingEntity.java:252,3042-3056,3306-3315`, LivingEntity'nin bu
handler'ı kullandığını ve `aiStep` içinde head interpolation'dan önce
çalıştırdığını gösterir. `client/player/RemotePlayer.java:43-62,72-76`, remote
Player tick'inde aynı interpolation sırasını ve network motion yolunu taşır.

`world/entity/Entity.java:2535-2565`, `moveOrInterpolateTo` çağrısının handler
varsa hedef kurduğunu, handler yoksa doğrudan setter kullandığını tanımlar.
`client/multiplayer/ClientPacketListener.java:639-754,806-823`, position sync,
teleport ve MoveEntity paketlerinde ticking ve uzaklık koşullarına göre bu
çağrının veya snap yolunun seçildiğini gösterir. Bu dilimin canlı senaryosu,
üç tick'lik handler dalını açan ticking remote Player'ın küçük MoveEntity
hareketiyle sınırlıdır.

`util/Mth.java:558-560,608-613`, position ve rotation lerplerinin işlem
sırasını; `world/entity/InterpolationHandler.java:81-107`, önceki tick'ten
beri oluşan delta'nın hedefe eklenmesini ve her adım sonunda previous state'in
güncellenmesini tanımlar.

## Sözleşme

- Player entity type ID `156` için küçük bir MoveEntity position güncellemesi
  current position'ı hemen değiştirmez; hedef position ve body rotation ayrı
  interpolation state'inde tutulur.
- Hedef kurulduğunda kalan adım sayısı `3` olur. World tick'leri sırayla
  `1/3`, `1/2` ve `1` alpha kullanır; üçüncü adımda hedefe ulaşır.
- Body yaw Java double `rotLerp` ile, pitch Java double linear `lerp` ile
  hesaplanır ve entity setter'larının derece sınırları uygulanır.
- Aynı hedef aktifken yeni bir interpolation başlatılmaz; hedef değişirse
  current state'ten yeni üç adımlı hedef kurulur.
- Bu kayıt, hareket paketinin ilk üç world tick'inde external movement olmayan
  yüklü air-cell senaryosunu kapsar. Collision gate'i, büyük uzaklıkta snap,
  teleport/position-sync dalının tamamı ve genel LivingEntity türleri açık
  kapsamdır.

## Rust doğrulaması

`ClientEntity` Player state'i için target position/rotation, previous
interpolation state'i ve kalan adım sayısını saklar. `ClientWorld::tick`,
remote Player entity'sinde tick sayacından sonra position interpolation'ı,
ardından mevcut head interpolation'ını yürütür. Packet adapter'ı relative
MoveEntity, PositionSync ve Teleport konum tabanını aktif Player hedefi üzerinden
çözer; Player dışı entity'ler önceki raw snap yolunu korur.

`remote_player_position_interpolation_follows_java_three_step_handler` unit
testi `(1,80,-35.5)` konumundan `(4,82,-34.5)` konumuna ve `350/-20` body
rotation'dan `10/20` değerine geçişte üç ara state'i, target state'i ve adım
sayısını kontrol eder.

## Vanilla server-conformance doğrulaması

**Tarih:** 2026-09-14

**Referans:** Java Edition 26.2 / protokol 776 / Java 25

**Sunucu:** Mojang vanilla 26.2 dedicated server, offline, seed `776`,
creative test modu, `view-distance=5`, `simulation-distance=5`,
`127.0.0.1:25568`

Observer probe'u önce bağlandı; mover probe'u aynı vanilla oturumunda üç
`MovePlayer.PosRot` paketi (`90`, `180`, `270` yaw) gönderdi. Adapter, remote
type ID `156` için ilk no-op hareketi atlayıp ilk gerçek position+rotation
hedefinde üç `ClientWorld::tick` çalıştırdı:

```text
cargo run --locked --offline --example world_remote_player_interpolation_probe -- 127.0.0.1:25568 P08InterpObs3 P08InterpMove3
```

Başarılı çıktı:

```text
connection=ok
play_login=true
player_loaded_sent=true
mover_movement_packets_sent=3
remote_player_id=3604
remote_player_interpolation_packet_count=1
remote_player_interpolation_before_position=(23.5, 80.0, -36.5)
remote_player_interpolation_target_position=(23.75, 80.0, -36.5)
remote_player_interpolation_after_tick_positions=[(23.583333333333332, 80.0, -36.5), (23.666666666666664, 80.0, -36.5), (23.75, 80.0, -36.5)]
remote_player_interpolation_before_rotation=(0.0, 0.0)
remote_player_interpolation_target_rotation=(90.0, 0.0)
remote_player_interpolation_after_tick_rotations=[(30.0, 0.0), (60.0, 0.0), (90.0, 0.0)]
remote_player_interpolation_tick_count=3
world_entity_count=41
world_chunk_count=157
received_packet_count=2538
```

Çıktı, vanilla'nın remote Player hareketini Rust world kopyasına aldığını,
target state'i koruduğunu ve üç tick sonunda position/yaw hedeflerine ulaştığını
gösterir. Entity ID ve packet sayıları canlı oturum durumuna bağlıdır; sonuç
Java istemcisiyle parity iddiası değildir.

## Açık kapsam

Genel LivingEntity ve araç/tekne interpolation türleri, packet position codec
base'inin hızlı arka arkaya güncellemelerde ayrıştırılması, collision kontrolü,
`4096.0` uzaklık snap dalı, teleport/position-sync özel history düzeltmesi,
external movement delta'sının gerçek collider ile doğrulanması, local player
input/movement, render partial tick ve Java istemcisiyle `client-parity`
karşılaştırması açık kalır.
