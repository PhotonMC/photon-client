# P08.ae — [server-conformance] Entity freeze progress semantic state

Bu kayıt, `ClientboundSetEntityDataPacket` içindeki Java entity freeze progress
alanının Rust `ClientWorld` state'ine semantik olarak uygulanmasını kapsar.
Kanıt sınıfı `server-conformance`'dır: Rust packet akışı Mojang vanilla 26.2
dedicated server'a bağlanır. Java istemcisiyle aynı girdide state
karşılaştırılmadığı için bu kayıt `client-parity` değildir.

## Java kaynak incelemesi

`Entity.java:271,320`, `DATA_TICKS_FROZEN` alanını entity data dizisinde ID `7`
olarak tanımlar ve varsayılanını `0` yapar. `Entity.java:2812-2833`, alanın
okuma/yazma yollarını, `isFreezing()` için pozitif eşik, `isFullyFrozen()` için
gereken tick eşiği ve varsayılan `getTicksRequiredToFreeze()` değerini `140`
olarak verir. `Entity.java:3881-3884`, pozitif freeze state'inin sorgusunu
tekrar kullanır.

`EntityDataSerializers.java:53-54,148-166`, `INT` serializer'ının ID `1`
olduğunu gösterir. Bu serializer signed VarInt kullanır. Metadata kayıtlarının
ID, serializer, değer ve `255` sonlandırıcısı biçimi
`ClientboundSetEntityDataPacket.java:21-37` içindedir; dirty kayıt sırasını ve
yalnız gönderilen alanların uygulanmasını `SynchedEntityData.java:74-85,108-125`
belirtir.

Freeze state'i istemci sunumunda da kullanılır: `client/gui/Hud.java:293-295`
powder snow overlay yüzdesini, `client/gui/Hud.java:1486-1487` tam donmuş HUD
durumunu ve `client/renderer/entity/LivingEntityRenderer.java:273-276` tam
donmuş render state'ini okur. Server tarafındaki LivingEntity freeze azaltma
ve hasar dalları `world/entity/LivingEntity.java:3152-3160` içindedir.

## Sözleşme

- Geçerli metadata içinde ID `7` ve `INT` serializer ID `1` varsa signed VarInt
  değeri `ticks_frozen` state'ine yazılır.
- Varsayılan state `0`'dır; `ticks_frozen > 0` ise `is_freezing()` true olur.
- Temel Java entity için `percent_frozen`, `min(ticks_frozen, 140) / 140`
  hesabının f32 sonucudur; `ticks_frozen >= 140` ise `is_fully_frozen()` true
  olur.
- ID `7` tek başına veya daha önce desteklenen BYTE, BOOLEAN ve POSE kayıtlarıyla
  birlikte gönderilebilir; gönderilmeyen alanlar önceki değerlerini korur.
- Yeni `ClientEntity` ve yeni `ClientPlayerState` freeze state'ini `0` ile
  başlatır.
- Local player yeniden kurulmasında `dataToKeep` bit 2 varsa freeze state'i
  mevcut entity data ile birlikte korunur; bit 2 yoksa yeni oyuncu `0` ile
  başlar.

## Rust doğrulaması

`metadata_int_value`, ID `7` için INT VarInt kaydını ve önündeki desteklenen
metadata kayıtlarını tüketir. `ClientEntity` ve `ClientPlayerState` üzerinde
`ticks_frozen`, `percent_frozen`, `is_freezing` ve `is_fully_frozen` sorguları
bulunur.

World unit testi varsayılan, kısmi, tam, signed negatif ve sıfırlanan freeze
değerlerini; birleşik metadata içinde pose/shared flags korunmasını doğrular.
Local player respawn testi `dataToKeep` bit 2 ile state'in korunmasını ve boş
maskeyle varsayılana dönülmesini kontrol eder.

## Vanilla server-conformance doğrulaması

**Tarih:** 2026-09-13

**Referans:** Java Edition 26.2 / protokol 776 / Java 25

**Sunucu:** Mojang vanilla 26.2 dedicated server, offline, seed `776`,
creative test modu, `view-distance=5`, `simulation-distance=5`,
`127.0.0.1:25568`

Probe bağlantısı açıkken sabit bir konumda `TicksFrozen:70` ve `NoGravity:1b`
alanlarına sahip bir minecart oluşturuldu. Minecart, LivingEntity freeze azaltma
tick'ine girmediği için metadata değerini koruyan kontrollü bir entity'dir:

```text
cargo run --locked --offline --example world_entity_ticks_frozen_probe -- 127.0.0.1:25568 P08FreezeL1
summon minecraft:minecart 25 90 -36 {TicksFrozen:70,NoGravity:1b}
```

Vanilla sunucusu summon komutunu kabul etti. Probe çıktısı:

| Gözlem | Sonuç |
| --- | ---: |
| `play_login` | `true` |
| `player_loaded_sent` | `true` |
| `entity_metadata_packet_count` | `452` |
| `ticks_frozen` | `70` |
| `percent_frozen` | `0.5` |
| `is_fully_frozen` | `false` |
| `entity_type_id` | `85` |
| `entity_position` | `(25.5, 90.0, -35.5)` |
| `entity_metadata_length` | `7` |
| `world_entity_count` | `42` |
| `world_chunk_count` | `157` |
| `received_packet_count` | `10044` |

Minecart entity'nin metadata üzerinden bulunması ve ID `7` INT değerinin
freeze türevleriyle world map'inde sorgulanması, vanilla sunucunun freeze
metadata akışının Rust semantic state'e ulaştığını gösterir. Bu sonuç Java
istemcisiyle parity iddiası değildir.

## Açık kapsam

LivingEntity freeze yavaşlatması ve hasar tick'i, powder snow overlay/HUD,
donmuş entity render shake state'i, türün `getTicksRequiredToFreeze()` override'ı,
client tick içi freeze geçişleri, diğer metadata serializer'ları ve
`client-parity` karşılaştırması açık kalır.
