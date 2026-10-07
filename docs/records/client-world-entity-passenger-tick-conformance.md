# P08.aw-entity-passenger-tick — [server-conformance] Passenger tick sırası

Bu kayıt, `ClientWorld` tick'inde araç ve passenger entity'lerinin Java
`ClientLevel.tickNonPassenger/tickPassenger` sırasına alınmasını kapsar. Kanıt
sınıfı `server-conformance`'dır: Rust probe'u Mojang vanilla 26.2 dedicated
server'a bağlanır. Java istemcisiyle aynı girdide state karşılaştırılmadığı
için bu kayıt `client-parity` değildir.

## Java kaynak incelemesi

`ClientLevel.java:461-491`, tick listesindeki entity'lerden passenger olmayan
ve donmamış kökleri seçer. Kök entity için eski transform kaydedilir, tick
sayacı artırılır ve entity tick'i çalıştırılır; ardından yolcular aynı araç
ilişkisi hâlâ geçerliyse recursive olarak işlenir.

`ClientLevel.java:479-490`, passenger'ın kaldırılmadığı ve hâlâ aynı araca
bağlı olduğu durumda Player veya ticking entity için eski transform/tick
sayacı adımlarını çalıştırıp `rideTick` çağırır. Passenger ağacının altındaki
entity'ler aynı sırayla devam eder. `Entity.java:2380-2390`, `rideTick` için
hareketi önce sıfırlar, ardından entity tick'ini ve araç üzerindeki rider
konumlandırmasını yürütür. `Entity.java:1812-1825`, eski konum/rotasyon
snapshot'ının tick başlangıcında alınan state olduğunu belirler.

Snowball'ın `ThrowableProjectile.java:46-75` tick gövdesi gravity, inertia,
konum ve rotation hesaplarını yapar. Bu nedenle passenger olarak tick edilen
bir snowball, normal üst seviye projectile tick'inden önce mevcut yatay
hareketini kaybeder. Bu gözlenebilir fark P08.aw için seçildi.

## Sözleşme

- `ClientWorld::tick`, vehicle ilişkisi olmayan ticking entity'leri kök olarak
  bir kez işler.
- Bir araç işlendiğinde, packet state'inde hâlâ o araca bağlı olan Player veya
  ticking passenger bir kez recursive olarak işlenir.
- Passenger tick'i, entity'nin `rideTick` davranışına uygun olarak mevcut
  movement'i sıfırlayarak başlar; desteklenen snowball tick'i bundan sonra
  gravity/inertia hesaplar.
- Passenger'ın tick count'u kök döngüsünde ikinci kez artırılmaz.
- Araç ve passenger tick count'ları aynı world tick'inde birer artar;
  nested passenger listesi vehicle sırasını izler.
- Passenger'ın `positionRider` ile araç geometrisine göre yeni konumu bu
  dilimde uygulanmaz; bu nedenle kayıt tick sırası ve movement reset'iyle
  sınırlıdır.

## Rust doğrulaması

`ClientWorld::tick`, ilişkisi olmayan entity'leri kök olarak toplar ve
`tick_entity_tree` ile passenger ağacını recursive yürütür. Passenger
entity'sinde movement sıfırlanır; mevcut snowball tick'i bu state üzerinden
çalışır. LocalPlayer ayrı state'te tutulduğu için aynı recursive yol local
oyuncu passenger'ını da tick listesine alır.

`passenger_tick_uses_ride_tick_motion_reset_before_projectile_tick` unit testi
loaded air section içinde hareketli snowball'ı araç entity'sine bağlar.
Araç ve passenger tick count'larının birer arttığını, snowball'ın yatay
movement'inin sıfırlandığını, Java gravity/inertia sonucunu ve yatay konumun
normal projectile adımıyla değişmediğini kontrol eder.

## Vanilla server-conformance doğrulaması

**Tarih:** 2026-09-14

**Referans:** Java Edition 26.2 / protokol 776 / Java 25

**Sunucu:** Mojang vanilla 26.2 dedicated server, offline, seed `776`,
creative test modu, `view-distance=5`, `simulation-distance=5`,
`127.0.0.1:25568`

Rust probe'u açıkken observer yakınında no-gravity oak boat ve yatay
hareketli snowball oluşturuldu; snowball boat'a bindirildi. Probe, uygulanan
passenger packet'inden sonra tek bir `ClientWorld` tick'i çalıştırdı:

```text
cargo run --locked --offline --example world_entity_passenger_tick_probe -- 127.0.0.1:25568 P08TickObs26
summon minecraft:oak_boat 14.5 70 -25.5 {Tags:["P08VT26"],NoGravity:1b}
summon minecraft:snowball 14.5 70 -25.5 {Tags:["P08PT27"],NoGravity:1b,Motion:[0.125d,0.0d,0.0d]}
ride @e[type=minecraft:snowball,tag=P08PT27,limit=1,sort=nearest] mount @e[type=minecraft:oak_boat,tag=P08VT26,limit=1,sort=nearest]
```

Vanilla konsolu snowball'ın boat'a bindiğini bildirdi. Rust probe çıktısı:

| Gözlem | Sonuç |
| --- | ---: |
| `play_login` | `true` |
| `player_loaded_sent` | `true` |
| `vehicle_id` | `6382` |
| `passenger_id` | `6388` |
| `passenger_type_id` | `120` |
| `vehicle_tick_count` | `0→1` |
| `passenger_tick_count` | `0→1` |
| `passenger_position` | `(21.866455078125,70.0,-25.5)→(21.866455078125,70.0,-25.5)` |
| `passenger_motion` | `(0.05060123298541175,0.0,0.0)→(0.0,0.0,0.0)` |
| `passenger_tick_observation_count` | `1` |
| `world_entity_count` | `48` |
| `world_chunk_count` | `157` |
| `received_packet_count` | `19525` |

Canlı akışta araç ve passenger entity'leri world kopyasında mevcut olduğu
için tick gözlemi uygulandı. Passenger'ın yatay movement'inin sıfırlanması,
Java `rideTick` sırasının Rust world tick'ine taşındığını gösterir. Bu vanilla
kabulü wire/world tick akışını gösterir; Java istemcisiyle tam parity iddiası
değildir.

## Açık kapsam

`positionRider` ile araç geometrisine göre passenger konumu [P08.bg dahil ilgili
attachment kayıtlarında](client-world-entity-passenger-generic-position-conformance.md),
boat kontrolü ve head/yaw yan etkileri, donmuş entity filtresi, kaldırılmış
entity davranışı,
diğer entity tick gövdeleri, generic motion/physics, collision, passenger
tick sonrası transform history ve Java istemcisiyle `client-parity`
karşılaştırması açık kalır.
