# P08.bl-entity-arrow-tick — [server-conformance] Arrow free-flight world tick

Bu kayıt, vanilla sunucusunun oluşturduğu normal Arrow entity'sinin yüklü ve
çarpışmasız hava hücresindeki tek `ClientWorld` tick'ini kapsar. Kanıt sınıfı
`server-conformance`'dır: Rust probe Mojang vanilla 26.2 dedicated server'dan
gelen AddEntity state'ini alır ve aynı state üzerinde Rust world tick'ini
uygular. Java client ile aynı girdide durum karşılaştırması yapılmadığı için
bu kayıt `client-parity` değildir.

## Java kaynak incelemesi

`EntityTypes.java:190-193` Arrow entity'sini kayıt sırasındaki protocol type
ID `6` ile tanımlar. `ClientLevel.java:450-477`, tick'e uygun root entity için
önce eski transform'un saklanmasını, tick sayacının artırılmasını ve entity
`tick()` gövdesinin yürütülmesini belirler.

`AbstractArrow.java:176-280`, fizik açık ve entity yerinde değilken hareket
ve çarpışma yolunu yürütür. MISS sonucu için Arrow önce tick başındaki hareket
vektörüyle konum alır; aynı hareketten rotation hedeflerini hesaplar; hava
inertia'sını `0.99F` ile uygular; `NoGravity` kapalıysa sonrasında `0.05`
gravity ekler. `AbstractArrow.java:323-349` bu inertia ve gravity sabitlerini
belirler. `Projectile.java:329-340`, pitch/yaw hedefinin mevcut rotation'a
`0.2F` wrapped lerp ile uygulanmasını tanımlar. `Entity.java:1510-1529`,
`NoGravity` durumunda gravity değerini sıfırlar.

## Sözleşme

- Yalnız Arrow protocol type ID `6` için bu dar free-flight tick'i uygulanır.
- Başlangıçta mevcut position ve motion'ın bulunduğu hücre ile bir sonraki
  position'ın hücresi yüklü vanilla air state ID `0` olmalıdır. Bu koşul,
  block clip sonucunun MISS olduğu küçük fixture'ı temsil eder.
- Tick başında eski position/rotation saklanır ve Arrow tick sayacı bir artırılır.
- Konum, inertia veya gravity uygulanmadan önceki hareket vektörüyle alınır.
- Pitch/yaw aynı hareket vektöründen hesaplanır ve mevcut rotation'a Java'nın
  `0.2F` rotation lerp sırasıyla yaklaşır; head rotation bu dalda değişmez.
- Konum adımından sonra üç motion bileşeni Java f32 `0.99F` değerinin f64
  Vec3 işlemine genişletilmiş karşılığıyla çarpılır. `NoGravity=false` ise
  yalnız Y bileşeninden `0.05` çıkarılır.
- Block/entity collision, water inertia, in-ground state, projectile hit,
  removal, particle/effect ve serverbound hareket paketleri bu kaydın dışında
  kalır. Hava hücresi koşulu sağlanmıyorsa entity yalnız genel tick state'ini
  korur; bu durum collision sonucu olarak yorumlanmaz.

## Rust doğrulaması

`ClientWorld::tick`, Arrow için geçiş hücrelerini air state ile sınırlayan
free-flight yolunu seçer. `ClientEntity::tick_arrow`, Java işlem sırasını
position, rotation, inertia ve gravity olarak korur. Unit testi gravity ve
`NoGravity` dallarını, önceki position'ı, tick sayısını ve rotation yaklaşımını
kontrol eder.

Vanilla bağlantı probe'u, bağlantı açıkken sunucu konsolundan eklenen Arrow
AddEntity packet'ini world kopyasına uygular ve packet geldiği anda tek tick
çalıştırır. Example, AddEntity başlangıç state'ini, tick sonrası position,
motion ve rotation değerlerini kesin eşitlikle karşılaştırır.

## Vanilla server-conformance doğrulaması

**Tarih:** 2026-09-14

**Referans:** Java Edition 26.2 / protokol 776 / Java 25

**Sunucu:** Mojang vanilla 26.2 dedicated server, offline, seed `776`,
creative test modu, `view-distance=5`, `simulation-distance=5`,
`127.0.0.1:25568`

**Sunucu SHA-1:** `823e2250d24b3ddac457a60c92a6a941943fcd6a`

Probe bağlantısı açıkken sunucu konsolunda açık hava hücresine şu Arrow
eklendi:

```text
execute as P08ArrowTick104 at @s run summon minecraft:arrow ~ ~15 ~ {Motion:[0.25d,0.5d,0.25d]}
```

Çalıştırma:

```text
cargo run --locked --offline --example world_entity_arrow_tick_probe -- 127.0.0.1:25568 P08ArrowTick104
```

Canlı sonuç:

```text
connection=ok
play_login=true
player_loaded_sent=true
arrow_add_entity_id=25437
arrow_add_position=(16.5, 81.0, -27.5)
arrow_add_motion=(0.2500152597204419, 0.5000305194408838, 0.2500152597204419)
arrow_after_tick_position=(16.750015259720442, 81.50003051944088, -27.249984740279558)
arrow_after_tick_motion=(0.24751510950756883, 0.4450302190151377, 0.24751510950756883)
arrow_before_tick_rotation=(0.0, 0.0, 0.0)
arrow_after_tick_rotation=(8.999999, 10.947107, 0.0)
arrow_tick_count=1
world_entity_count=64
world_chunk_count=157
received_packet_count=11469
```

AddEntity movement'i protocol fixed-point çözümünden sonra `0.250015...`,
`0.500030...` ve `0.250015...` olarak world'e girdi. Son position bu hareket
vektörünün doğrudan eklenmesiyle; son motion ise önce `0.99F` inertia, sonra
`0.05` gravity ile elde edildi. Rotation sonucu da aynı başlangıç hareketi ve
`0.2F` lerp ile eşleşti. Entity ID, spawn konumu, dünya entity sayısı ve packet
sayısı canlı oturum state'ine bağlıdır.

## Açık kapsam

Arrow block clip ve entity hit test'i, in-ground metadata/flags, water inertia,
projectile impact/removal, particle/effect state'i, diğer arrow türleri,
genel motion integration, serverbound movement/paddle akışı, renderer ve
Java client parity'si açık kalır. P08'in `client-parity` kapanışı da henüz
yoktur.
