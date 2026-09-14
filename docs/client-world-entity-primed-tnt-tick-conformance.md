# P08.bn-entity-primed-tnt-tick — [server-conformance] Primed TNT free-flight tick

Bu kayıt, vanilla sunucusunun oluşturduğu Primed TNT entity'sinin yüklü,
çarpışmasız ve fluidsiz hava hücresindeki tek `ClientWorld` tick'ini kapsar.
Kanıt sınıfı `server-conformance`'dır: Rust probe Mojang vanilla 26.2
dedicated server'dan gelen AddEntity state'ini alır ve aynı state üzerinde
Rust world tick'ini yürütür. Java client ile aynı girdide durum
karşılaştırması yapılmadığı için bu kayıt `client-parity` değildir.

## Java kaynak incelemesi

`EntityTypes.java:981-990` Primed TNT entity'sini registry sırasındaki
protocol type ID `133` ile tanımlar. `ClientLevel.java:467-472`, uygun root
entity için eski transform'un saklanmasını, tick sayacının artırılmasını ve
entity `tick()` gövdesinin çağrılmasını belirler.

`PrimedTnt.java:93-122`, entity tick'inde önce gravity uygulandığını, sonra
delta movement ile hareket edildiğini ve hareketin `getAirDrag()` ile
çarpıldığını tanımlar. `PrimedTnt.java:94-96` varsayılan gravity değerini
`0.04`, `Entity.java:1533-1535` varsayılan air drag değerini `0.98F` olarak
belirler. `Entity.java:1510-1529`, `NoGravity` açıkken gravity değerinin
sıfırlanmasını tanımlar.

## Sözleşme

- Yalnız Primed TNT protocol type ID `133` için bu dar free-flight tick yolu
  uygulanır.
- Başlangıçta entity'nin mevcut konumu ve gravity sonrası hedef konumu yüklü
  vanilla air state ID `0` hücrelerinde olmalıdır. Bu koşul, hareket öncesi
  block collision bulunmayan fixture'ı temsil eder.
- Tick başında eski position/rotation saklanır ve Primed TNT tick sayacı bir
  artırılır.
- `NoGravity=false` ise önce hareketin Y bileşeninden `0.04` çıkarılır; entity
  bu güncellenmiş hareketle konum alır.
- Konum adımından sonra üç movement bileşeni Java f32 `0.98F` değerinin f64
  işlemine genişletilmiş karşılığıyla çarpılır. Bu hava dalında rotasyon
  değişmez ve zemin bounce yolu çalışmaz.
- Portal, block/entity collision, water, ground friction/bounce, fuse
  decrement, smoke particle, explosion, removal, block state ve renderer
  state'i bu kaydın dışındadır.

## Rust doğrulaması

`ClientWorld::tick`, Primed TNT için mevcut ve gravity sonrası geçiş
hücrelerini air state ile sınırlar. `ClientEntity::tick_primed_tnt`, Java'nın
gravity → position → air drag sırasını uygular. Generic
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

Probe bağlantısı açıkken oyuncunun 8 blok dışına, yüklü hava hücresine şu TNT
eklendi:

```text
execute as P08Tnt105 at @s run summon minecraft:tnt ~ ~15 ~ {fuse:80}
```

Çalıştırma:

```text
cargo run --locked --offline --example world_entity_primed_tnt_tick_probe -- 127.0.0.1:25568 P08Tnt105
```

Canlı sonuç:

```text
connection=ok
play_login=true
player_loaded_sent=true
primed_tnt_add_entity_id=26081
primed_tnt_add_position=(26.5, 77.0, -38.5)
primed_tnt_add_motion=(0.0, 0.0, 0.0)
primed_tnt_after_tick_position=(26.5, 76.96, -38.5)
primed_tnt_after_tick_motion=(0.0, -0.03920000076293945, 0.0)
primed_tnt_tick_count=1
world_entity_count=85
world_chunk_count=157
received_packet_count=17937
```

Canlı AddEntity movement'i sıfırdı. Tek tick'te position Y değeri `0.04`
azaldı; hareket Y değeri önce gravity uygulanıp sonra `0.98F` ile çarpılarak
`-0.03920000076293945` oldu. Entity ID, dünya entity sayısı ve packet sayısı
canlı oturum state'ine bağlıdır.

## Açık kapsam

Primed TNT fuse state'i ve fuse decrement, smoke particle, explosion/removal,
block/entity collision, water/ground friction ve bounce, portal, block state,
genel entity lifecycle, renderer ve Java client parity'si açık kalır. P08'in
`client-parity` kapanışı da henüz yoktur.
