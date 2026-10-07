# P08.ap-player-head-rotation — [server-conformance] Remote Player head rotation interpolation

Bu kayıt, vanilla sunucusunun Player entity'si için gönderdiği `RotateHead`
packet'inin Rust `ClientWorld` kopyasında Java LivingEntity head interpolation
state'ine dönüştürülmesini kapsar. Kanıt sınıfı `server-conformance`'dır: iki
Rust probe bağlantısı Mojang vanilla 26.2 dedicated server'a bağlanır. Java
istemcisiyle aynı girdide frame/state karşılaştırması yapılmadığı için bu kayıt
`client-parity` değildir.

## Java kaynak incelemesi

`ClientboundRotateHeadPacket.java:12-50`, entity ID ile packed head byte'ını
taşır ve `Mth.unpackDegrees` ile dereceye çevirir. `ClientPacketListener.java:767-772`,
paketi mevcut entity için `lerpHeadTo(yaw, 3)` çağrısına verir; packet değeri
Player'ın head state'ine doğrudan snap edilmez.

`Entity.java:2568-2570`, genel entity için `lerpHeadTo` çağrısını doğrudan head
setter'ına bağlar. `LivingEntity.java:240-254,3053-3055,3312-3315,3994-3995`,
living entity'nin hedef head rotation ve kalan adım sayısını sakladığını, her
tick'te `rotLerp(1.0 / steps, current, target)` uygulayıp adımı azalttığını
gösterir. `LivingEntity.java:3357-3358,3380-3386`, render/view sorgusunun
old/current head state'i wrapped interpolation ile kullandığını doğrular.
`RemotePlayer.java:37-50`, remote Player tick yolunda aynı üç adımlı head
interpolation dalını yürütür.

`ClientPacketListener.java:793-803`, server'ın local oyuncuya gönderdiği
position correction'ı uygular; local oyuncunun head bakışı server'dan gelen
`RotateHead` packet'ine bağlı değildir. Bu nedenle canlı senaryo packet'i
gözlemci bağlantısında remote Player üzerinde üretir.

## Sözleşme

- Player entity type ID `156` için yeni `RotateHead` packet'i hedef head
  rotation'ı ve kalan `3` interpolation adımını kurar.
- Her world tick'i önce eski head state'ini yakalar, sonra kalan adım sayısına
  göre Java `rotLerp` hesabını uygular. Üçüncü tick sonunda current head hedef
  değere eşittir; 0/360 sınırında hesap kısa yolu değil wrapped derece farkı
  kullanır.
- Yeni bir head packet'i önceki hedefe ulaşmadan gelirse hedef ve üç adımlı
  sayaç yeniden kurulur.
- Player olmayan entity'lerde önceki P08.e raw head rotation davranışı korunur;
  bu kaydın network interpolation kapsamı Player/LivingEntity dalıdır.
- Local player head state'i ayrı current/previous/target alanlarında tutulur;
  gerçek local mouse/input bakış üretimi ve body-head dönüş sınırları bu kaydın
  dışındadır.

## Rust doğrulaması

`ClientEntity` Player type ID `156` için head hedefini ve kalan adımları tutar;
`ClientWorld::tick` entity eski transform snapshot'ından sonra Java ile aynı
wrapped interpolation adımını yürütür. `ClientPlayerState` aynı state'i local
oyuncu world kopyasında korur ve current/previous/interpolated head sorgularını
sağlar. AddEntity ve respawn keep-data yolları head history/target state'ini
başlangıçta ve yeniden oluşturmada korur.

Head probe adapter'ı mevcut Movement action'ını parametrik kullanır: observer
önce bağlanır, mover oyuncu üç farklı yaw (`90`, `180`, `270`) gönderir ve
bağlantıyı gözlem tamamlanana kadar açık tutar. Böylece yeni bir packet action
ve aynı davranışı çoğaltan ayrı bir kod yolu eklenmez.

## Vanilla server-conformance doğrulaması

**Tarih:** 2026-09-14

**Referans:** Java Edition 26.2 / protokol 776 / Java 25

**Sunucu:** Mojang vanilla 26.2 dedicated server, offline, seed `776`,
creative test modu, `view-distance=5`, `simulation-distance=5`,
`127.0.0.1:25568`

Observer bağlantısı mover bağlantısından önce açıldı; vanilla yeni Player
entity'sini mevcut tracking client'a gönderdi. Mover üç `MovePlayer.PosRot`
packet'i gönderdi ve observer'ın `RotateHead` kayıtlarını alması beklendi:

```text
cargo run --locked --offline --example world_remote_player_head_rotation_probe -- 127.0.0.1:25568 P08HeadObs7 P08HeadMove7
```

Başarılı Rust probe çıktısı:

```text
connection=ok
play_login=true
player_loaded_sent=true
mover_movement_packets_sent=3
observer_local_player_id=2541
remote_player_id=2602
remote_player_head_rotation_packet_count=3
remote_player_head_rotation_targets=[90.0, -180.0, -90.0]
remote_player_head_rotation_before_ticks=0
remote_player_head_rotation_target=-90
remote_player_head_rotation_after_three_ticks=-90
world_entity_count=43
world_chunk_count=157
received_packet_count=2751
```

Probe, üç `RotateHead` kaydının tamamını type ID `156` remote entity'sine
uyguladı; hedefler değişti ve son hedef `-90` üç `ClientWorld::tick` sonrasında
aynı değere ulaştı. `270` derece üçüncü yaw'ın packed signed byte açılımı
vanilla packet'inde `-90` olarak gözlendi. Entity ID, packet sayısı ve dünya
entity sayısı canlı oturuma bağlıdır.

## Açık kapsam

Local mouse/input'tan head bakışı üretimi, Player body rotation ile head
rotation arasındaki 50 derece sınırı, hareket/fizik entegrasyonu, passenger
durumu, gerçek client entity tick gövdesi, renderer/model/skin ve Java ile
aynı frame çıktısının karşılaştırılması açık kalır. Bu kayıt P08'i
`client-parity` ile kapatmaz.
