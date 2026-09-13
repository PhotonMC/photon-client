# P08.u — [server-conformance] Entity tick transform geçmişi ve interpolation

Bu kayıt, `ClientWorld` içindeki local player ve ticking remote entity
transformlarının world tick sınırında geçmiş state'e alınmasını ve bu iki
state arasında Java sıralı interpolation yapılmasını kapsar. Kanıt sınıfı
`server-conformance`'tır: Rust istemcisi Mojang vanilla sunucusundan aldığı
dünya durumunu kurar ve ölçülebilir tick yüzeyini çalıştırır. Java istemcisiyle
aynı girdide state karşılaştırması yapılmadığı için bu kayıt `client-parity`
değildir.

## Java kaynak incelemesi

Java Edition 26.2 / protokol 776 / Java 25 kaynaklarında şu davranış dalları
incelendi:

- `client/multiplayer/ClientLevel.java:450-477`, `tickNonPassenger` içinde
  entity tick'inden önce `setOldPosAndRot()` çağrıldığını gösterir.
- `world/entity/Entity.java:1816-1844`, mevcut position ve body rotation
  değerlerini `xo/xOld`, `yo/yOld`, `zo/zOld`, `yRotO` ve `xRotO` geçmişine
  kopyalar. `snapTo` da yeni entity'nin ilk geçmiş değerlerini mevcut
  transform ile başlatır.
- `world/entity/Entity.java:1960-1964,2000-2004`, position interpolation'ı
  `Mth.lerp` ile, pitch'i linear lerp ve yaw'ı `Mth.rotLerp` ile okur; partial
  değer `1.0F` olduğunda rotation sorgusu doğrudan mevcut değeri döndürür.
- `world/entity/LivingEntity.java:240-241,498,3357`, head rotation için
  `yHeadRotO` geçmişini ve `Mth.rotLerp` yolunu ekler.
- `util/Mth.java:558-560,608-613`, double position lerp'inin ve wrapped
  rotation lerp'inin aritmetik sırasını tanımlar.

Packet listener'ın `moveOrInterpolateTo` ve snap ayrımı bu dilimin sınırıdır:
genel network hedefini çok adımlı interpolation state'ine çevirmiyoruz; remote
Player'ın dar üç tick'lik dalı [P08.ar kaydında](client-world-entity-player-interpolation-conformance.md)
ve büyük uzaklık snap dalı [P08.as kaydında](client-world-entity-player-snap-conformance.md)
ayrıca tutulur. Alınan
current transform'ı bir sonraki uygun world tick'inde history snapshot'ına
alıyoruz.

## Sözleşme

- Yeni local player veya remote entity state'i oluşturulurken geçmiş position
  ve rotation mevcut ilk transform ile aynıdır.
- `ClientWorld::tick`, local player için ve `ticking=true` remote entity için
  mevcut position/body/head rotation değerlerini önceki state'e kopyalar;
  ardından P08.t tick sayacını artırır.
- Network/application güncellemesi current state'i değiştirir ve previous
  state'i bir sonraki uygun tick'e kadar korur. Böylece aradaki sorgu, Java
  `getPosition(partialTick)` benzeri iki state'i görür.
- Position interpolation, Java `Mth.lerp` source order'ını korur ve Rust'a
  alınan `f32` partial tick'i double aritmetik için genişletir. Body yaw ve
  living head yaw wrapped `Mth.rotLerp`, pitch ise linear `Mth.lerp` kullanır.
- Chunk dışındaki remote entity tick edilmediği için history snapshot'ı da
  ilerlemez. Local player state'i chunk uygunluğundan bağımsız tick edilir.

## Rust doğrulaması

`src/world.rs` içindeki
`tick_captures_previous_transform_for_position_and_rotation_interpolation`
testi şu sıralamayı kontrol eder: ilk snapshot, tick sonrasında current
transform değişimi, position lerp, `350° → 10°` wrapped yaw/head lerp,
`partial_tick=1.0` doğrudan current rotation sonucu, chunk unload sırasında
history'nin korunması ve reload sonrası yeni snapshot.

```text
cargo test --locked --offline --all-targets --quiet
```

Sonuç: `86 passed; 0 failed`.

## Vanilla server-conformance doğrulaması

**Tarih:** 2026-09-13

**Referans:** Java Edition 26.2 / protokol 776 / Java 25

**Sunucu:** Mojang vanilla 26.2 dedicated server, offline yerel koşu

**Sunucu SHA-1:** `823e2250d24b3ddac457a60c92a6a941943fcd6a`

Sunucu `/tmp/voxel-vanilla-26.2-world` altında `127.0.0.1:25568`, seed `776`,
creative/peaceful, view/simulation distance `5` ve boş sunucuda 60 saniye
pause ayarıyla çalıştırıldı. JAR ve dünya depoya alınmadı.

```text
cargo run --locked --offline -- 127.0.0.1:25568 MainInterpNow 20 20
```

Çıktı:

```text
client_connection=ok
play_login=true
world_constructed=true
world_chunk_count=24
world_section_count=576
ticks_run=20
world_game_time=20
local_player_previous_position=(22.5, 80.0, -35.5)
local_player_interpolated_position_half=(22.5, 80.0, -35.5)
local_player_tick_count=20
rendered_frames=0
```

Bu koşu, vanilla sunucusundan alınan ilk local player transformunun world
state'ine ulaştığını, bağlantı sonrası 20 tick'in çalıştığını ve history/
interpolation sorgularının ana binary üzerinden erişilebilir olduğunu gösterir.
Canlı koşuda transform tick'ler arasında değiştirilmediği için ara konum
sonucu current konumla aynıdır; değişen transform ve açı sarmalama davranışı
unit testte ölçülür. Sunucu logunda bu oturum için protocol error oluşmadı.

## Açık kapsam

Bu kayıt `moveOrInterpolateTo` içindeki genel çok adımlı network interpolation'ı,
teleport snap'in geniş history düzeltmesini, gerçek entity tick/motion entegrasyonunu,
passenger transformlarını, LivingEntity body rotation semantic'ini ve render
frame/partial-tick zamanlamasını kapatmaz. Java istemcisiyle karşılaştırmalı
`client-parity` kanıtı da henüz yoktur. Remote Player'ın dar PositionSync snap
dalı [P08.as kaydında](client-world-entity-player-snap-conformance.md)
ayrıca ölçülmüştür.
