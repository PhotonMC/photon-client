# P08.aq-snowball-tick — [server-conformance] Snowball free-flight world tick

Bu kayıt, vanilla sunucusunun oluşturduğu snowball entity'sinin `ClientWorld`
tick'i içinde açık ve çarpışmasız bir hücreden ilerlemesini kapsar. Kanıt sınıfı
`server-conformance`'dır: Rust probe Mojang vanilla 26.2 dedicated server'a
bağlanır ve server'ın gönderdiği AddEntity state'ini Rust world tick'ine verir.
Java istemcisiyle aynı girdide frame/state karşılaştırması yapılmadığı için bu
kayıt `client-parity` değildir.

## Java kaynak incelemesi

`world/item/SnowballItem.java:23-47`, başarılı `use` çağrısında server level'a
`Snowball` projectile'ı ekler; `Projectile.java:99-108`, `ThrowableProjectile`
sınıfının temel projectile tick zincirini Entity tick'ine bağlar.

`client/multiplayer/ClientPacketListener.java:565-590` AddEntity packet'inden
entity'yi oluşturup `recreateFromPacket` ile konum, rotation ve başlangıç
movement state'ini kurar. `client/multiplayer/ClientLevel.java:450-477`,
tick'e uygun entity için eski transform'u kaydeder, `tickCount` değerini artırır
ve entity'nin `tick()` gövdesini çalıştırır.

`world/entity/projectile/ThrowableProjectile.java:46-65,91-113`, snowball'ın
her tick'te önce gravity'yi, sonra hava sürtünmesini uyguladığını ve hareket
sonucunda MISS varsa yeni konumu `position + deltaMovement` olarak seçtiğini
gösterir. `world/entity/Entity.java:1510-1529`, `NoGravity` true olduğunda
gravity'nin sıfıra indiğini tanımlar. `world/entity/projectile/Projectile.java:329-346`,
hareket vektöründen hedef pitch/yaw'ı üretip eski rotasyona doğru `0.2F` ile
yaklaştırır; `world/phys/Vec3.java:188-190` yatay uzaklığı f64 karekök ile
hesaplar. `util/Mth.java:391-438,550-552`, atan2 ve f32 lerp işlemlerinin
Java sırasını sağlar.

## Sözleşme

- Yalnız vanilla snowball entity type ID `120` bu dilimde projectile tick'i
  yürütür. Genel entity'ler ve diğer projectile türleri bu davranıştan
  etkilenmez.
- Tick başında eski konum ve rotation saklanır, ardından entity tick sayacı
  bir artırılır.
- Snowball için `NoGravity=false` ise hareketin Y bileşenine `-0.03` eklenir;
  sonra üç bileşenin tamamı Java `0.99F` hava sürtünmesiyle çarpılır. Bu f32
  sabit f64 Vec3 çarpımına yükseltildiği için Rust tarafında `f64::from(0.99F)`
  karşılığı korunur.
- Açık MISS dalında yeni konum, sürtünme sonrası hareketin mevcut konuma
  eklenmesidir. Aynı hareketten hedef pitch/yaw hesaplanır ve eski rotation'a
  Java `lerpRotation` sırasıyla `0.2F` yaklaşılır.
- Rust world kopyasında block-state verisi bulunmayan veya current/target
  hücrelerden biri vanilla air state ID `0` olmayan yol bu dar dalda
  ilerletilmez. Block clip, entity hit, su, portal, impact/removal ve gerçek
  collision sonucu sonraki kapsamdır; bu kayıt yalnız loaded air-cell free-flight
  koşulunu kapatır.

## Rust doğrulaması

`ClientWorld::tick`, yalnız type ID `120` olan ve yüklü air-cell yoluna sahip
entity'lerde snowball gravity/drag/position/rotation adımını yürütür.
`ClientEntity` eski transform, motion ve current rotation state'ini korur;
`no_gravity` metadata'sı gravity dalına bağlanır.

`ticking_snowballs_applies_java_free_flight_gravity_drag_and_history` unit testi
normal gravity ve `NoGravity` dallarını, f32 `0.99` drag widening'ini, önceki
konumu, tick sayacını ve projectile rotation yaklaşımını kontrol eder.
`world_entity_snowball_tick_probe`, vanilla AddEntity içindeki başlangıç
movement state'ini aldıktan sonra tek world tick uygular ve Java işlem sırası
ile hesaplanan konum/hareket değerlerini kesin eşitlikle karşılaştırır.

## Vanilla server-conformance doğrulaması

**Tarih:** 2026-09-14

**Referans:** Java Edition 26.2 / protokol 776 / Java 25

**Sunucu:** Mojang vanilla 26.2 dedicated server, offline, seed `776`,
creative test modu, `view-distance=5`, `simulation-distance=5`,
`127.0.0.1:25568`

Probe, creative hotbar'a snowball koyup `use_item` gönderdi. Snowball AddEntity
paketi uygulandıktan hemen sonra Rust world kopyasında tek tick çalıştırıldı.
Başarılı vanilla koşularından ikisi:

```text
cargo run --locked --offline --example world_entity_snowball_tick_probe -- 127.0.0.1:25568 P08SnowTick4

connection=ok
play_login=true
player_loaded_sent=true
snowball_add_entity_id=3135
snowball_add_position=(22.5, 81.52000000327826, -36.5)
snowball_add_motion=(-0.006470121467374801, 0.00781297686626381, 1.487395470914973)
snowball_after_tick_position=(22.493594579685595, 81.49803485016426, -35.02747846960927)
snowball_after_tick_motion=(-0.00640542031440494, -0.02196515311399077, 1.472521530390732)
snowball_before_tick_rotation=(-1.40625, 0.0, 0.0)
snowball_after_tick_rotation=(-1.1748424, -0.17092533, 0.0)
snowball_tick_count=1
world_entity_count=37
world_chunk_count=129
received_packet_count=1329
```

```text
cargo run --locked --offline --example world_entity_snowball_tick_probe -- 127.0.0.1:25568 P08SnowTick5

connection=ok
play_login=true
player_loaded_sent=true
snowball_add_entity_id=3249
snowball_add_position=(26.5, 81.52000000327826, -36.5)
snowball_add_motion=(-0.019776597692730213, 0.004516877250808626, 1.5112006348043705)
snowball_after_tick_position=(26.480421168095592, 81.49477171151354, -35.00391135713174)
snowball_after_tick_motion=(-0.019578831904407245, -0.025228291764725456, 1.4960886428682592)
snowball_before_tick_rotation=(-1.40625, 0.0, 0.0)
snowball_after_tick_rotation=(-1.2749364, -0.19318335, 0.0)
snowball_tick_count=1
world_entity_count=48
world_chunk_count=129
received_packet_count=1637
```

Her iki koşuda da AddEntity type ID `120` olarak uygulandı; probe'un beklenen
movement ve position değerleri Java gravity/drag/MISS sırasıyla birebir eşleşti.
Entity ID, spawn koordinatı, mevcut mob sayısı ve packet sayısı canlı dünya
durumuna bağlıdır. Bu sonuç yalnız Rust world kopyasının vanilla packet state'i
üzerindeki dar snowball free-flight adımını gösterir; Java client parity'si
ve genel projectile fiziği iddiası değildir.

## Açık kapsam

Block/entity collision ve hit sonucu, projectile impact/removal, su inertia'sı,
diğer projectile ve living entity tick gövdeleri, passenger ilişkisi, genel
motion integration, network interpolation, renderer ve oynanabilir istemci
bağlantısı açık kalır. P08'in `client-parity` kapanışı da henüz yoktur.
