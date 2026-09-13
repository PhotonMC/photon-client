# P08.t — [server-conformance] ClientWorld tick sayaçları ve ticking uygunluğu

Bu kayıt, vanilla 26.2 play oturumundan oluşturulan `ClientWorld` kopyasının
world tick çağrısını gözlenebilir sayaçlara uygulamasını kapsar. Kanıt sınıfı
`server-conformance`'tır: Rust istemcisi Mojang vanilla sunucusundan aldığı
dünya durumunu kurar ve sonra bu durumu tick eder. Java istemcisiyle aynı
girdide state karşılaştırması yapılmadığı için bu kayıt `client-parity`
değildir.

## Kaynak incelemesi

Java Edition 26.2 / protokol 776 / Java 25 kaynak incelemesinde şu dallar
esas alındı:

- `client/multiplayer/ClientLevel.java:450-477`, `tickEntities` içindeki
  `tickNonPassenger` çağrısında tick edilebilir entity'nin eski konum/rotasyon
  bilgisini kaydettiği, `tickCount` değerini artırdığı ve entity tick'ini
  yürüttüğü görülür.
- `world/entity/Entity.java:247,1816-1835,3955-3972`, `tickCount` alanını,
  eski transform güncellemesini ve varsayılan `isAlwaysTicking=false`
  davranışını tanımlar.
- `world/entity/player/Player.java:1931-1934`, oyuncuların her tick'te
  değerlendirilen `isAlwaysTicking=true` dalını açar.
- `world/level/entity/TransientEntitySectionManager.java:28-65,95-140`, section
  ticking listeleri ile her zaman tick edilen entity'lerin ayrı uygunluk
  yolunu gösterir.
- `client/Minecraft.java:1791-1799`, unpaused tick sırasındaki renderer,
  entity ve block entity çağrılarını tanımlar.

## Sözleşme

- `ClientWorld::tick` her çağrıda world game time değerini bir artırır.
- Mevcut local player state'i her world tick'inde `tick_count` değerini bir
  artırır. Yeni veya respawn ile yeniden oluşturulan player state'i `0`'dan
  başlar.
- Remote entity yeni oluşturulduğunda `tick_count=0` olur ve yalnızca mevcut
  `ticking` uygunluğu true iken bir artırılır. Chunk unload bu uygunluğu
  kapatır; chunk yeniden yüklendiğinde sonraki tick'te sayaç devam eder.
- Sayaç taşmaları Rust'ta `wrapping_add` ile tutulur. Type `120` snowball için
  loaded air-cell free-flight entity tick gövdesi [P08.aq kaydında](client-world-entity-snowball-tick-conformance.md)
  sınırlı olarak uygulanır; diğer entity tick gövdeleri bu dilimin dışındadır.
  Remote Player position interpolation [P08.ar kaydında](client-world-entity-player-interpolation-conformance.md)
  ayrı olarak ilerletilmiştir.

## Rust doğrulaması

`src/world.rs` içindeki
`tick_counters_follow_local_player_and_entity_ticking_eligibility` testi
şunları aynı state kopyasında kontrol eder: local player'ın üç ardışık tick'te
ilerlemesi, yüklü remote entity'nin ilerlemesi, chunk unload sırasında
durması, yeniden yüklemeden sonra devam etmesi ve player respawn'ında sıfıra
dönmesi. Snowball gravity/drag ve transform history testi ayrıca
[P08.aq kaydında](client-world-entity-snowball-tick-conformance.md) tutulur.

```text
cargo test --locked --offline --all-targets --quiet
```

Sonuç: `85 passed; 0 failed`.

## Vanilla server-conformance doğrulaması

**Tarih:** 2026-09-13

**Referans:** Java Edition 26.2 / protokol 776 / Java 25

**Sunucu:** Mojang vanilla 26.2 dedicated server, offline yerel koşu

**Sunucu SHA-1:** `823e2250d24b3ddac457a60c92a6a941943fcd6a`

Sunucu `/tmp/voxel-vanilla-26.2-world` altında `127.0.0.1:25568`, seed `776`,
creative/peaceful, view/simulation distance `5` ve boş sunucuda 60 saniye
pause ayarıyla çalıştırıldı. JAR ve dünya depoya alınmadı.

```text
cargo run --locked --offline -- 127.0.0.1:25568 MainTickNow 20 20
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
local_player_tick_count=20
rendered_frames=0
```

Bu koşu, ana binary'nin vanilla sunucusundan dünya kopyası kurabildiğini ve
bağlantı sonrasında local player tick sayacını çalıştırılan 20 world tick ile
aynı değere getirdiğini gösterir. Sunucu logunda bu oturum için protocol error
oluşmadı.

## Açık kapsam

Bu kayıt genel `Entity.tick()` davranışını, snowball dışı motion entegrasyonunu,
collision'ı, eski konum/rotasyon interpolasyonunun tüm genel yollarını,
removed/passenger/frozen filtrelerini, always-ticking remote entity manager'ını
ve entity lifecycle'ını kapatmaz. Snowball free-flight alt dalı [P08.aq](client-world-entity-snowball-tick-conformance.md),
remote Player position interpolation [P08.ar](client-world-entity-player-interpolation-conformance.md)
ile sınırlı olarak ilerletilmiştir.
Java istemcisiyle karşılaştırmalı `client-parity` kanıtı da henüz yoktur.
