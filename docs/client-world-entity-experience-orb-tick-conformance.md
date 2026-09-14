# P08.bm-entity-experience-orb-tick — [server-conformance] Experience Orb client tick

Bu kayıt, vanilla sunucusunun oluşturduğu Experience Orb entity'sinin yüklü,
çarpışmasız ve fluidsiz hava hücresindeki ilk iki `ClientWorld` tick'ini
kapsar. Kanıt sınıfı `server-conformance`'dır: Rust probe Mojang vanilla 26.2
dedicated server'dan gelen AddEntity state'ini alır ve aynı state üzerinde
Rust world tick'lerini yürütür. Java client ile aynı girdide durum
karşılaştırması yapılmadığı için bu kayıt `client-parity` değildir.

## Java kaynak incelemesi

`EntityTypes.java:440-443` Experience Orb entity'sini registry sırasındaki
protocol type ID `49` ile tanımlar. `ClientLevel.java:467-472`, uygun root
entity için eski transform'un saklanmasını, tick sayacının artırılmasını ve
entity `tick()` gövdesinin çağrılmasını belirler. `Entity.java:247,253`,
`tickCount` ve ilk tick state'ini taşır.

`ExperienceOrb.java:96-152`, client tarafındaki ilk tick'te yalnızca
`firstTick` state'ini kapatır; sonraki tick'te genel entity tick'inden sonra
çarpışma yoksa gravity uygular, hareket eder ve hava sürtünmesini hareketten
sonra uygular. `ExperienceOrb.java:154-156`, hava sürtünmesini `0.98F` olarak
belirler; `ExperienceOrb.java:96-99` varsayılan gravity değerini `0.03` olarak
belirler. `Entity.java:1510-1529`, `NoGravity` açıkken bu gravity değerinin
sıfırlanmasını tanımlar.

## Sözleşme

- Yalnız Experience Orb protocol type ID `49` için bu entity-specific client
  tick yolu uygulanır.
- AddEntity sonrası ilk client world tick'i `firstTick` koruması nedeniyle
  konumu ve hareketi değiştirmez; tick sayacı yine ClientLevel tarafından
  artırılır.
- Sonraki tick'te yüklü, boş ve fluidsiz hava hücresinde `NoGravity=false`
  ise önce hareketin Y bileşeninden `0.03` çıkarılır.
- Entity, gravity uygulanmış hareket vektörüyle konum alır; sonra üç hareket
  bileşeni Java f32 `0.98F` değerinin f64 işlemine genişletilmiş karşılığıyla
  çarpılır. Bu dalda rotasyon değişmez.
- Yakındaki oyuncuya yönelme, su/lava hareketi, block/entity collision,
  ground friction, vertical bounce, merge, age/lifetime, pickup/effect,
  removal ve renderer state'i bu kaydın dışındadır.

## Rust doğrulaması

`ClientEntity::first_tick`, Experience Orb'un istemciye özgü ilk tick
korumasını taşır. `ClientWorld::tick`, ikinci tick'te yalnız geçiş hücreleri
vanilla air state ID `0` olduğunda Experience Orb hareket yolunu çalıştırır.
`ClientEntity::tick_experience_orb`, gravity → position → air friction sırasını
uygular.

Unit testi ilk tick'te state'in korunmasını, ikinci tick'te position/motion
sonucunu, önceki position geçmişini ve iki tick sayısını kontrol eder. Kalıcı
probe, AddEntity başlangıç state'ini ve iki world tick'inin ayrı sonuçlarını
kesin eşitlikle karşılaştırır.

## Vanilla server-conformance doğrulaması

**Tarih:** 2026-09-14

**Referans:** Java Edition 26.2 / protokol 776 / Java 25

**Sunucu:** Mojang vanilla 26.2 dedicated server, offline, seed `776`,
creative test modu, `view-distance=5`, `simulation-distance=5`,
`127.0.0.1:25568`

**Sunucu SHA-1:** `823e2250d24b3ddac457a60c92a6a941943fcd6a`

Probe bağlantısı açıkken oyuncunun 8 blok dışına, yüklü hava hücresine şu orb
eklendi:

```text
execute as P08XOrb105 at @s run summon minecraft:experience_orb ~ ~15 ~ {Value:1}
```

Çalıştırma:

```text
cargo run --locked --offline --example world_entity_experience_orb_tick_probe -- 127.0.0.1:25568 P08XOrb105
```

Canlı sonuç:

```text
connection=ok
play_login=true
player_loaded_sent=true
experience_orb_add_entity_id=25757
experience_orb_add_position=(24.5, 77.0, -38.5)
experience_orb_add_motion=(0.0, 0.0, 0.0)
experience_orb_after_first_tick_position=(24.5, 77.0, -38.5)
experience_orb_after_first_tick_motion=(0.0, 0.0, 0.0)
experience_orb_after_second_tick_position=(24.5, 76.97, -38.5)
experience_orb_after_second_tick_motion=(0.0, -0.02940000057220459, 0.0)
experience_orb_tick_count=2
world_entity_count=82
world_chunk_count=157
received_packet_count=15162
```

Canlı AddEntity movement'i sıfırdı. İlk tick sonucu başlangıç state'iyle aynı
kaldı; ikinci tick'te position Y değeri `0.03` azaldı ve hareket Y değeri
`(-0.03) * 0.98F` genişletilmiş sonucu olan
`-0.02940000057220459` değerine geldi. Entity ID, dünya entity sayısı ve
packet sayısı canlı oturum state'ine bağlıdır.

## Açık kapsam

Experience Orb'un oyuncuya yaklaşma dalı, su/lava hareketi, block/entity
collision, ground friction/bounce, merge ve lifetime/pickup lifecycle'ı,
fluid/block effect'leri, XP value/collect state'i, renderer ve Java client
parity'si açık kalır. P08'in `client-parity` kapanışı da henüz yoktur.
