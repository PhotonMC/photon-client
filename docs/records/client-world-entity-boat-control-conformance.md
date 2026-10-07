# P08.bh-entity-boat-control — [server-conformance] Boat input and `deltaRotation`

Bu kayıt, client-authoritative boat tick'inde dört input bayrağının
`deltaRotation`, heading, hareket ve paddle state'ine bağlanmasını kapsar.
Kanıt sınıfı `server-conformance`'dır: Rust world kopyası Mojang vanilla 26.2
sunucusundan gelen entity/passenger state'i üzerinde çalıştırıldı. Java
istemcisiyle aynı oturum state'i karşılaştırılmadığı için bu kayıt
`client-parity` değildir.

## Java kaynak incelemesi

`AbstractBoat.java:60-69`, boat'ın `deltaRotation` ile sol/sağ/yukarı/aşağı
input alanlarını ayrı state olarak tuttuğunu gösterir. `764-768` içindeki
setter dört bayrağı kopyalar; `LocalPlayer.java:975-978` bunu yolcu oyuncunun
klavye snapshot'ından her tick besler.

`AbstractBoat.java:586-620` içindeki client-authoritative control dalı şu
işlem sırasını kullanır:

- sol input `deltaRotation` değerini bir azaltır, sağ input bir artırır;
  ikisi birlikteyse iki işlem birbirini götürür;
- yalnızca dönüş input'u varken ivmeye `0.005F` eklenir;
- boat yaw'ı güncel `deltaRotation` ile artırılır;
- ileri için `0.04F`, geri için `-0.005F` ivme eklenir;
- `Mth.sin(-yaw * DEG_TO_RAD)` ve `Mth.cos(yaw * DEG_TO_RAD)` ile bulunan
  f32 ivmeler mevcut `Vec3` hareketine eklenir;
- paddle sırası `(right && !left || forward, left && !right || forward)`
  olarak yazılır.

`AbstractBoat.java:631-642` bu `deltaRotation` değerini `positionRider`
sırasında `CAN_TURN_IN_BOATS` dışındaki passenger'ın yaw ve head yaw'ına
ekler, sonra clamp uygular. Doğrudan passenger dönüşündeki
`onPassengerTurned` callback'i (`677-688`) delta'yı ikinci kez eklemez.

Bu dilim yalnız control arithmetic ve passenger tick bağlantısını kapatır.
`floatBoat` status/friction, collision-aware `move`, paddle animasyonu/sesi ve
serverbound movement paketleri ayrı davranışlardır.

## Java oracle ve Rust sözleşmesi

Oracle, sabitlenen client JAR'ındaki `Mth.sin`/`Mth.cos` yollarını doğrudan
çağıran Java 25 yardımcı programıyla çalıştırıldı. Başlangıç değerleri her
satırda `deltaRotation=0`, boat yaw `10.0F`, movement
`(0.125D,0.0D,-0.25D)` idi. Aşağıdaki hareket bitleri, Java'nın önce f32
ivme çarpımını, sonra `Vec3` double genişletmesini korur.

| Input `(left,right,forward,backward)` | `deltaRotation` f32 | boat yaw f32 | movement `(x,z)` raw f64 bits | paddles `(left,right)` |
| --- | ---: | ---: | --- | --- |
| `(true,false,false,false)` | `bf800000` | `41100000` | `(3fbfccc081000000,bfcf5e2d19000000)` | `(false,true)` |
| `(false,true,false,false)` | `3f800000` | `41300000` | `(3fbfc17da4400000,bfcf5f2b33000000)` | `(true,false)` |
| `(false,false,true,false)` | `00000000` | `41200000` | `(3fbe38e6d8000000,bfcaf52ee0000000)` | `(true,true)` |
| `(false,false,false,true)` | `00000000` | `41200000` | `(3fc01c7192800000,bfd050ad12000000)` | `(false,false)` |
| `(true,true,false,false)` | `00000000` | `41200000` | `(3fc0000000000000,bfd0000000000000)` | `(false,false)` |
| `(true,false,true,false)` | `bf800000` | `41100000` | `(3fbe660408000000,bfcaf168c8000000)` | `(true,true)` |

Rust `BoatInput` bu dört bayrağı taşır. `ClientWorld::set_boat_input`, pinned
boat type'larında input snapshot'ını saklar; world tick'i passenger bulunan
boat'ta Java control adımını yürütür. `ClientEntity` delta, paddle ve motion
sonuçlarını sorgulanabilir state olarak tutar. Passenger tick'i delta'yı
living passenger'ın current yaw/head yaw'ına ekler, ardından mevcut boat clamp
ve body/head dispatch yollarını çalıştırır.

`boat_control_applies_java_input_delta_and_paddles` tek parametrik testte altı
ayırt edici input satırını raw f64/f32 bitleriyle karşılaştırır.
`boat_control_delta_rotation_reaches_passenger_tick` aynı tick içinde sağ
input'ının boat yaw'ını ve Zombie passenger'ın current/head/body state'ini
güncellediğini kontrol eder.

## Vanilla server-conformance doğrulaması

**Tarih:** 2026-09-14

**Referans:** Java Edition 26.2 / protokol 776 / Java 25

**Sunucu:** Mojang vanilla 26.2 dedicated server, offline, seed `776`,
creative test modu, `view-distance=5`, `simulation-distance=5`,
`127.0.0.1:25568`

Observer oyuncusu yanında no-gravity oak boat oluşturuldu ve oyuncu boat'ın
controlling passenger'ı yapıldı. İlişki packet'inin Rust world kopyasına
ulaştığı kontrol edildikten sonra probe sağ input snapshot'ı verip bir world
tick çalıştırdı.
Fixture komutları:

```text
execute as P08Control94 at @s run summon minecraft:oak_boat ~5 ~ ~ {Tags:["P08BHV94"],NoGravity:1b,Rotation:[0.0f,0.0f]}
ride P08Control94 mount @e[type=minecraft:oak_boat,tag=P08BHV94,limit=1,sort=nearest]
```

Çalıştırma:

```text
cargo run --locked --offline --example world_entity_passenger_boat_control_probe -- 127.0.0.1:25568 P08Control94
```

Gözlenen sonuç:

```text
connection=ok
play_login=true
player_loaded_sent=true
vehicle_id=22298
vehicle_type_id=89
before_vehicle_rotation=(0.0, 0.0, 0.0)
after_vehicle_rotation=(1.0, 0.0, 0.0)
delta_rotation=1
paddle_state=(true, false)
local_player_id=22165
world_entity_count=174
world_chunk_count=157
received_packet_count=10966
```

Oak boat type `89` ve local player ID `22165` ile sıfır başlangıç heading'i
doğrulandı. Sağ input'ı bir control tick'inde `deltaRotation=1`, boat yaw'ı
`1` ve paddle state'i `(true,false)` üretti.
Bu gözlem Rust world kopyasının gerçek vanilla entity/passenger akışına
uyguladığı kaynak türevi dalı gösterir; Java istemcisinin local prediction,
frame veya paket gönderimiyle tam eşitliği göstermez.

## Açık kapsam

Boat status'una bağlı `deltaRotation` friction'ı, su/karada `floatBoat`,
collision-aware hareket ve world position ilerlemesi, `ServerboundPaddleBoat`
ve `ServerboundMoveVehicle` paketleri, gerçek pencere/input callback'i,
paddle model/ses, diğer vehicle türleri ve Java istemcisiyle `client-parity`
karşılaştırması açık kalır. Passenger seat ve default attachment yolları
[P08.ax](client-world-entity-passenger-position-conformance.md),
[P08.bg](client-world-entity-passenger-generic-position-conformance.md),
boat clamp ve direct turn yolları [P08.ba](client-world-entity-passenger-rotation-conformance.md)
ile [P08.be](client-world-entity-passenger-turn-conformance.md) kayıtlarında
izlenir.
