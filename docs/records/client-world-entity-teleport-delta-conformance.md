# P08.au-entity-teleport-motion — [server-conformance] Teleport delta ve relative motion

Bu kayıt, `ClientboundTeleportEntityPacket` içindeki relative position,
rotation ve delta movement alanlarının `ClientWorld` entity state'ine
uygulanmasını kapsar. Kanıt sınıfı `server-conformance`'dır: Rust probe'u
Mojang vanilla 26.2 dedicated server'a bağlanır. Java istemcisiyle aynı
girdide state karşılaştırılmadığı için bu kayıt `client-parity` değildir.

## Java kaynak incelemesi

`ClientboundTeleportEntityPacket.java:12-39`, entity ID'sinden sonra
`PositionMoveRotation`, `Relative.SET_STREAM_CODEC` ile relative maskesi ve
`onGround` alanlarını taşır. `Relative.java:10-19,50-114`, X/Y/Z konum
bitlerini `0/1/2`, Y/X rotation bitlerini `3/4`, delta X/Y/Z bitlerini
`5/6/7` ve `ROTATE_DELTA` bitini `8` olarak kodlar.

`PositionMoveRotation.java:40-67`, relative konum ve rotation alanlarını
mevcut state'e ekler, pitch'i `[-90,90]` aralığına clamp eder. Delta alanında
ilgili bit set ise packet delta'sı mevcut harekete eklenir; bit yoksa packet
değeri hareketi değiştirir. `ROTATE_DELTA` set olduğunda mevcut hareket önce
`xRot`, sonra `yRot` ile `Vec3.xRot/yRot` sırasına göre döndürülür.
`Vec3.java:232-247` kaynak çarpma/toplama sırasını, `Mth.java:50-55` ise
Java sin/cos lookup quantization'ını belirler.

`ClientPacketListener.java:666-698`, entity mevcutsa teleport'u işler ve
relative position bulunduğunda küçük hareketi interpolation yoluna sokar;
ardından `onGround` state'ini yazar. `ClientPacketListener.java:806-826`,
uygulanan `PositionMoveRotation` sonucunu entity'ye verir. `Entity.java:3739-3743`
`setDeltaMovement` için yalnızca finite `Vec3` değerlerini kabul eder.

Vanilla komutunun gerçek relative packet üretimi
`TeleportCommand.java:224-228,252-266` ve `Entity.java:3139-3151` dallarından
gelir. `~` koordinatları position ve direction relative bitlerini birlikte
kurar. Kontrolcü passenger'a gönderilen teleport transition bu maskeyi ve
transition delta'sını taşır; kontrolcü olmayan riding player için packet
mutlak/boş maskeli ayrı bir fallback yoluna gider.

## Sözleşme

- Mevcut entity'nin konumu, rotation'ı ve hareketi packet alanlarının
  absolute/relative çözümünün başlangıç state'idir.
- X/Y/Z bitleri ilgili packet konumunu mevcut konuma ekler. Y_ROT/X_ROT
  bitleri rotation'a ekleme yapar; X_ROT sonucu `-90` ile `90` arasında tutulur.
- DELTA_X/Y/Z bitleri mevcut hareket bileşenine packet delta'sını ekler.
  İlgili bit yoksa o bileşen packet delta'sı ile değiştirilir.
- ROTATE_DELTA set ise mevcut hareket, source rotation ile hesaplanan target
  rotation farkıyla önce X sonra Y ekseninde döndürülür.
- Hesaplanan hareket bileşenlerinden biri finite değilse Java setter'ı mevcut
  hareketi korur.
- Bu davranış yalnız `TELEPORT_ENTITY` dispatch'inde uygulanır. Entity
  Position Sync packet'inin delta alanları bu kaydın kapsamına taşınmaz.
- Entity map'inde olmayan ID için packet observation korunur, world
  application sonucu `false` olur.

## Rust doğrulaması

`ClientWorld::apply_entity_teleport`, packet'in relative position ve rotation
alanlarını mevcut entity state'ine çözer; `resolve_entity_teleport_motion`
delta bitlerini ve `ROTATE_DELTA` dönüş sırasını uygular. `set_motion_if_finite`
Java'nın `setDeltaMovement` finite kontrolünü korur. Probe teleport packet'ini
bu uygulamadan geçirir ve packet öncesi/uygulama sonrası hareketi observation'a
yazar.

`entity_teleport_applies_relative_delta_motion_and_rotation` unit testi tüm
position/rotation/delta bitleriyle ekleme yolunu, X sonra Y hareket dönüşünü ve
non-finite delta'da eski hareketin korunmasını kontrol eder.

## Vanilla server-conformance doğrulaması

**Tarih:** 2026-09-14

**Referans:** Java Edition 26.2 / protokol 776 / Java 25

**Sunucu:** Mojang vanilla 26.2 dedicated server, offline, seed `776`,
creative test modu, `view-distance=5`, `simulation-distance=5`,
`127.0.0.1:25568`

Vanilla'nın kontrolcü passenger dalını çalıştırmak için observer oyuncusu bir
oak boat'a bindirildi. Boat, observer'ın bulunduğu noktaya yakın oluşturuldu
ve göreli koordinatlarla teleport edildi:

```text
cargo run --locked --offline --example world_entity_teleport_delta_probe -- 127.0.0.1:25568 P08Boat25
summon minecraft:oak_boat 24.5 62 -38.5 {Tags:["P08Boat25"],NoGravity:1b,Motion:[0.125d,0.0d,0.0d]}
ride P08Boat25 mount @e[type=minecraft:oak_boat,tag=P08Boat25,limit=1,sort=nearest]
execute as P08Boat25 at @s run tp @e[type=minecraft:oak_boat,tag=P08Boat25,limit=1,sort=nearest] ~1 ~2 ~-3
```

Vanilla konsolu boat teleport'ini bildirdi. Rust probe'u `flags=255` ile
position, rotation ve üç delta relative bitinin birlikte geldiğini; delta
payload'ının sıfır olduğunu ve uygulama sonrası hareketin değişmediğini
gözledi:

| Gözlem | Sonuç |
| --- | ---: |
| `play_login` | `true` |
| `player_loaded_sent` | `true` |
| `entity_id` | `6121` |
| `teleport_position_payload` | `(1.0, 1.5874999999999986, -3.0)` |
| `teleport_delta_payload` | `(0.0, 0.0, 0.0)` |
| `teleport_relative_flags` | `255` |
| `motion_before` | `(0.0, 0.0, 0.0)` |
| `applied_motion` | `(0.0, 0.0, 0.0)` |
| `applied_position` | `(26.521728515625, 63.5875, -41.5)` |
| `applied_rotation` | `(0.0, 0.0, 0.0)` |
| `world_entity_count` | `47` |
| `world_chunk_count` | `157` |
| `received_packet_count` | `20397` |

Canlı packet, Rust world kopyasında uygulandı. Bu oturum zero-delta relative
packet yolunu doğrular; non-zero delta, rotation ve non-finite setter dalları
Java kaynaklı unit testte kontrol edilir. Vanilla kabulü wire ve uygulama
akışını gösterir; Java istemcisiyle tam parity iddiası değildir.

## Açık kapsam

Generic entity tick/fizik, yerçekimi, collision ve motion integration;
`PositionMoveRotation.of` içindeki known-movement/passenger ayrıntıları;
araç kontrolü, passenger konumlandırması, hızlı ardışık teleport'lar, render
partial tick state'i ve Java istemcisiyle `client-parity` karşılaştırması açık
kalır. Konum uygulamasının temel dalları [P08.e position kaydında](client-world-entity-position-conformance.md),
remote Player interpolation [P08.ar kaydında](client-world-entity-player-interpolation-conformance.md),
snap [P08.as kaydında](client-world-entity-player-snap-conformance.md) ve
motion interpolation [P08.at kaydında](client-world-entity-player-motion-conformance.md)
izlenir.
