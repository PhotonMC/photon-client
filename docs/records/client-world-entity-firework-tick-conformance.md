# P08.bp-entity-firework-tick — [server-conformance] Firework Rocket free-flight tick

Bu kayıt, vanilla sunucusunun oluşturduğu Firework Rocket entity'sinin yüklü,
çarpışmasız ve fluidsiz hava hücresindeki tek `ClientWorld` tick'ini kapsar.
Kanıt sınıfı `server-conformance`'dır: Rust probe Mojang vanilla 26.2
dedicated server'dan gelen AddEntity state'ini alır ve aynı state üzerinde
Rust world tick'ini yürütür. Java client ile aynı girdide durum
karşılaştırması yapılmadığı için bu kayıt `client-parity` değildir.

Bu dilim Java'daki normal, bağlı olmayan ve `shotAtAngle=false` uçuş dalını
kapsar. Bu dal, önceki serbest uçuş entity'lerinden farklı olarak yatay
hızlanma, dikey hız ekleme ve hareketten rotasyon üretme sırasını birlikte
uygular.

## Java kaynak incelemesi

`EntityTypes.java:460-467` Firework Rocket entity'sini registry sırasındaki
protocol type ID `53` ile tanımlar. `ClientLevel.java:467-472`, uygun root
entity için eski transform'un saklanmasını, tick sayacının artırılmasını ve
entity `tick()` gövdesinin çağrılmasını belirler.

`FireworkRocketEntity.java:113-169`, bağlı olmayan ve `shotAtAngle=false`
dalında yatay hareket bileşenlerinin `1.15` ile çarpıldığını, Y hareketine
`0.04` eklendiğini, bu hareketle önce hit test ve sonra konum adımı yapıldığını,
hareketin korunup `updateRotation()` çağrıldığını belirler.
`Projectile.java:329-346`, rotasyon hedefinin hareketten üretildiğini ve eski
rotasyona `0.2F` ile yaklaştırıldığını tanımlar. Bu dalda FallingBlock ve
Primed TNT'deki gibi air drag veya gravity uygulanmaz.

## Sözleşme

- Yalnız Firework Rocket protocol type ID `53` için normal, bağlı olmayan ve
  `shotAtAngle=false` free-flight tick yolu uygulanır.
- Başlangıçta entity'nin mevcut konumu ve hızlanmış hareket sonrası hedef
  konumu yüklü, boş vanilla air state ID `0` hücrelerinde olmalıdır. Bu koşul,
  hareket öncesi block collision bulunmayan fixture'ı temsil eder.
- Tick başında eski position/rotation saklanır ve Firework Rocket tick sayacı
  bir artırılır.
- Mevcut hareketin X/Z bileşenleri `1.15` ile çarpılır, Y bileşenine `0.04`
  eklenir; entity bu hareketle konum alır ve hareket state'i aynı değerle
  korunur.
- Rotasyon, hızlandırılmış hareketten Java `atan2` ve `0.2F` wrapped lerp
  sırasıyla üretilir; Z rotasyonu değişmez.
- Bağlı entity/fall-flying, `shotAtAngle=true`, hit/deflection, block/entity
  collision, fluid/block effects, life/lifetime, particle, launch sound,
  explosion/removal, firework item/component metadata, renderer state'i ve
  Java client parity'si bu kaydın dışındadır.

## Rust doğrulaması

`ClientWorld::tick`, Firework Rocket için mevcut ve hızlanmış geçiş hücrelerini
vanilla air state ile sınırlar. `ClientEntity::tick_firework`, normal uçuşta
Java'nın yatay hızlanma → dikey hız ekleme → position → rotation sırasını
uygular ve hareketi drag olmadan korur. Generic
`world_entity_tick_observations` kaydı AddEntity başlangıç state'ini ve tek
tick sonucunu taşır.

Unit testi önce implementasyon olmadan beklenen hareket farkını kırmızıya
çevirdi; implementasyon sonrası position, motion, previous position, rotation
ve tick count state'lerini kontrol eder. Kalıcı example, canlı vanilla
AddEntity değerlerini aynı formülle kesin eşitlikle karşılaştırır.

## Vanilla server-conformance doğrulaması

**Tarih:** 2026-09-14

**Referans:** Java Edition 26.2 / protokol 776 / Java 25

**Sunucu:** Mojang vanilla 26.2 dedicated server, offline, seed `776`,
creative test modu, `view-distance=5`, `simulation-distance=5`,
`127.0.0.1:25568`

**Sunucu SHA-1:** `823e2250d24b3ddac457a60c92a6a941943fcd6a`

Probe bağlantısı açıkken oyuncunun 15 blok üstüne, yüklü hava hücresine yaşam
süresi uzatılmış varsayılan Firework Rocket entity'si eklendi:

```text
execute as P08Firework at @s run summon minecraft:firework_rocket ~ ~15 ~ {LifeTime:100,Life:0}
```

Çalıştırma:

```text
cargo run --locked --offline --example world_entity_firework_tick_probe -- 127.0.0.1:25568 P08Firework
```

Canlı sonuç:

```text
connection=ok
play_login=true
player_loaded_sent=true
firework_add_entity_id=26765
firework_add_position=(27.5, 77.0, -37.5)
firework_add_motion=(0.0, 0.0, 0.0)
firework_after_tick_position=(27.5, 77.04, -37.5)
firework_after_tick_motion=(0.0, 0.04, 0.0)
firework_before_tick_rotation=(0.0, 0.0, 0.0)
firework_after_tick_rotation=(0.0, 18.0, 0.0)
firework_tick_count=1
world_entity_count=88
world_chunk_count=157
received_packet_count=14178
```

Canlı AddEntity movement'i sıfırdı. Tek tick'te position Y değeri `0.04`
arttı; hareket Y değeri `0.04` olarak kaldı ve sıfır yatay hareketten oluşan
pitch hedefi `90` dereceye `0.2F` ile yaklaştırılarak X rotasyonu `18.0` oldu.
Entity ID, dünya entity sayısı ve packet sayısı canlı oturum state'ine
bağlıdır.

## Açık kapsam

Firework Rocket item/component ve `shotAtAngle` metadata'sı, attached target ve
elytra boost dalı, collision/hit/deflection, lifetime/particle/sound/explosion
lifecycle'ı, renderer ve Java client parity'si açık kalır. P08'in
`client-parity` kapanışı da henüz yoktur.
