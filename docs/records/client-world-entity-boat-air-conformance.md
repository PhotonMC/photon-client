# P08.bj-entity-boat-air — [server-conformance] Local boat `IN_AIR` tick

Bu kayıt, yerel oyuncunun doğrudan kontrol ettiği boat entity'sinin havadaki
tek tick akışını kapsar: `floatBoat` içindeki hava drag/gravity adımı, client
control ivmesi ve ardından `move` konum adımı. Kanıt sınıfı
`server-conformance`'dır. Rust world kopyası Mojang vanilla 26.2 sunucusundan
gelen chunk, entity ve passenger state'i üzerinde çalıştırıldı; Java
istemcisiyle aynı oturum state'i karşılaştırılmadığı için bu kayıt
`client-parity` değildir.

## Java kaynak incelemesi

`AbstractBoat.java:210-251` client-authoritative boat tick'inde önce
`floatBoat`, sonra client tarafında `controlBoat`, ardından
`move(MoverType.SELF, getDeltaMovement())` çağrıldığını gösterir.
`383-399` status seçimini yapar: su ve kara koşulları yoksa boat
`IN_AIR` olur. `538-580` içindeki `floatBoat` bu dalda f32 `0.9F` yatay
friction uygular, gravity'yi y hareketine ekler ve `deltaRotation` değerini
aynı `0.9F` ile çarpar. `586-620` control dalı güncel yaw ile ileri ivmeyi
hesaplar; ileri input `0.04F` değerini `Mth.sin`/`Mth.cos` yönüne ekler.

`ClientLevel.java:471-490`, her root non-passenger entity'sini tick eder ve
passenger tick'inden önce vehicle'ın kendi tick'inin tamamlanmasını sağlar.
`Entity.java:3587-3603` local instance authority'yi controlling passenger'a
bağlar; `Player.java:1275-1283` local player dalını belirler.
`Entity.java:712-802` son hareketi collision-aware `move` yolundan geçirir.

Bu uygulama, henüz tam block shape/fluid sorgusu olmadığı için `IN_AIR`
durumunu yalnızca yüklenmiş ve açıkça havasız olmayan section'larda, boat'ın
örneklenen hacmindeki tüm bloklar vanilla air state'i olduğunda tanır. Su,
kara friction'ı, buoyancy, bubble column ve collision-aware çözüm bu dilimin
dışındadır.

## Java oracle ve Rust sözleşmesi

Pinned client JAR'ındaki `Mth.sin`/`Mth.cos` yollarını doğrudan kullanan Java
25 yardımcı programı iki kez çalıştırıldı; çıktılar `cmp -s` ile aynı bulundu.
Başlangıç float değerleri ve raw bit çıktıları aşağıdadır:

| Satır | Son hareket `(x,y,z)` raw f64 bitleri | `deltaRotation` f32 | Son konum `(x,y,z)` raw f64 bitleri |
| --- | --- | ---: | --- |
| `falling_low` | `(3fccccccc0000000,3fdd70a3d70a3d71,bfe5999990000000)` | `40666666` | `(4029733333000000,4020eb851eb851ec,c013b33332000000)` |
| `no_gravity_low` | `(3fccccccc0000000,3fe0000000000000,bfe5999990000000)` | `40666666` | `(4029733333000000,4021000000000000,c013b33332000000)` |
| `controlled_forward` | `(0000000000000000,0000000000000000,3fa47ae140000000)` | `00000000` | `(403c800000000000,4056400000000000,c03875c28f600000)` |

Rust `ClientWorld` yalnız local player'ı passenger listesinde taşıyan pinned
boat type'ında bu tick'i çalıştırır. `boat_is_in_air` yüklenmemiş veya bilinmeyen
block state'ini başarı saymaz. `ClientEntity` float adımında x/z hareketini,
gravity veya `NoGravity` y dalını ve `deltaRotation` drag'ını uygular; aynı
tick'te boat control input'ı bundan sonra çalışır ve son hareket mevcut
motion ile position'a eklenir.

`air_boat_float_applies_java_drag_gravity_and_position_step` falling ve
`NoGravity` satırlarını raw f64 bitleriyle karşılaştırır. Probe örneği ayrıca
ileri input sonrası motion, position, `deltaRotation` ve paddle state'ini
kontrol eder.

## Vanilla server-conformance doğrulaması

**Tarih:** 2026-09-14

**Referans:** Java Edition 26.2 / protokol 776 / Java 25

**Sunucu:** Mojang vanilla 26.2 dedicated server, offline, seed `776`,
creative test modu, `view-distance=5`, `simulation-distance=5`,
`127.0.0.1:25568`

Observer oyuncusunun yanında no-gravity oak boat oluşturuldu ve vanilla
`ride` komutuyla yerel oyuncuya bağlandı:

```text
execute as P08BoatAir87 at @s run summon minecraft:oak_boat ~5 ~20 ~ {Tags:["P08BJA87"],NoGravity:1b,Rotation:[0.0f,0.0f]}
ride P08BoatAir87 mount @e[type=minecraft:oak_boat,tag=P08BJA87,limit=1,sort=nearest]
```

Çalıştırma:

```text
cargo run --locked --offline --example world_entity_passenger_boat_air_probe -- 127.0.0.1:25568 P08BoatAir87
```

Gözlenen sonuç:

```text
connection=ok
play_login=true
player_loaded_sent=true
local_player_id=20131
vehicle_id=20238
vehicle_type_id=89
no_gravity=true
before_position=(28.5, 89.0, -24.5)
after_position=(28.5, 89.0, -24.46000000089407)
before_motion=(0.0, 0.0, 0.0)
after_motion=(0.0, 0.0, 0.03999999910593033)
before_delta_rotation=0
after_delta_rotation=0
world_entity_count=160
world_chunk_count=157
received_packet_count=25611
```

Sunucu başlangıç hareketini sıfır gönderdiği için probe client control
input'ı olarak ileri bayrağını uyguladı. `controlled_forward` Java oracle
satırıyla birlikte bu sonuç, vanilla packet akışından gelen local passenger
ilişkisi ve açık hava fixture'ı üzerinde Rust tick sırasını doğrular. Java
istemcisinin local prediction'ı, frame zamanlaması veya serverbound paddle ve
vehicle movement packet'leri bu kanıtta yoktur.

## Açık kapsam

Su ve kara status/friction dalları, buoyancy, collision-aware `Entity.move`,
fluid/shape çözümü, bubble column, paddle animasyonu/sesi,
`ServerboundPaddleBoat` ve `ServerboundMoveVehicle` paketleri, nested
controlling passenger, diğer vehicle türleri, gerçek pencere/input callback'i
ve Java istemcisiyle `client-parity` karşılaştırması açık kalır.
