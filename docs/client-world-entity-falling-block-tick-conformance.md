# P08.bo-entity-falling-block-tick — [server-conformance] FallingBlock free-flight tick

Bu kayıt, vanilla sunucusunun oluşturduğu FallingBlock entity'sinin yüklü,
çarpışmasız ve fluidsiz hava hücresindeki tek `ClientWorld` tick'ini kapsar.
Kanıt sınıfı `server-conformance`'dır: Rust probe Mojang vanilla 26.2
dedicated server'dan gelen AddEntity state'ini alır ve aynı state üzerinde
Rust world tick'ini yürütür. Java client ile aynı girdide durum
karşılaştırması yapılmadığı için bu kayıt `client-parity` değildir.

## Java kaynak incelemesi

`EntityTypes.java:448-455` FallingBlock entity'sini registry sırasındaki
protocol type ID `51` ile tanımlar. `ClientLevel.java:467-472`, uygun root
entity için eski transform'un saklanmasını, tick sayacının artırılmasını ve
entity `tick()` gövdesinin çağrılmasını belirler.

`FallingBlockEntity.java:142-155`, varsayılan gravity'nin `0.04` olduğunu ve
tick'te önce gravity uygulanıp sonra delta movement ile hareket edildiğini
belirler. Bu client dalında hareketten sonra air drag uygulanmaz.
`Entity.java:1510-1529`, `NoGravity` açıkken gravity değerinin sıfırlanmasını
tanımlar. `FallingBlockEntity.java:158-245`, yere ulaşma, block-state yerleştirme,
drop ve server-side lifecycle davranışlarını ayrı bir server dalında yürütür.

## Sözleşme

- Yalnız FallingBlock protocol type ID `51` için bu dar free-flight tick yolu
  uygulanır.
- Başlangıçta entity'nin mevcut konumu ve gravity sonrası hedef konumu yüklü,
  boş vanilla air state ID `0` hücrelerinde olmalıdır. Bu koşul, hareket öncesi
  block collision bulunmayan fixture'ı temsil eder.
- Tick başında eski position/rotation saklanır ve FallingBlock tick sayacı bir
  artırılır.
- `NoGravity=false` ise önce hareketin Y bileşeninden `0.04` çıkarılır; entity
  bu güncellenmiş hareketle konum alır.
- Konum adımından sonra movement bileşenleri değişmeden kalır; bu client
  free-flight dalında FallingBlock için air drag uygulanmaz ve rotasyon
  değişmez.
- Block-state çözümü ve air-state discard, block/entity collision, fluid/block
  effects, portal, yere oturma, bounce, block placement, drop/removal,
  renderer state'i ve Java client parity'si bu kaydın dışındadır.

## Rust doğrulaması

`ClientWorld::tick`, FallingBlock için mevcut ve gravity sonrası geçiş
hücrelerini vanilla air state ile sınırlar. `ClientEntity::tick_falling_block`,
Java'nın gravity → position sırasını uygular ve gravity sonrası hareketi
korur. Generic `world_entity_tick_observations` kaydı AddEntity başlangıç
state'ini ve tek tick sonucunu taşır.

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

Probe bağlantısı açıkken oyuncunun 15 blok üstüne, yüklü hava hücresine şu
kum FallingBlock entity'si eklendi:

```text
execute as P08Fall105 at @s run summon minecraft:falling_block ~ ~15 ~ {BlockState:{Name:"minecraft:sand"},Time:0}
```

Çalıştırma:

```text
cargo run --locked --offline --example world_entity_falling_block_tick_probe -- 127.0.0.1:25568 P08Fall105
```

Canlı sonuç:

```text
connection=ok
play_login=true
player_loaded_sent=true
falling_block_add_entity_id=26434
falling_block_add_position=(27.5, 87.0, -35.5)
falling_block_add_motion=(0.0, 0.0, 0.0)
falling_block_after_tick_position=(27.5, 86.96, -35.5)
falling_block_after_tick_motion=(0.0, -0.04, 0.0)
falling_block_before_tick_rotation=(0.0, 0.0, 0.0)
falling_block_after_tick_rotation=(0.0, 0.0, 0.0)
falling_block_tick_count=1
world_entity_count=98
world_chunk_count=157
received_packet_count=13172
```

Canlı AddEntity movement'i sıfırdı. Tek tick'te position Y değeri `0.04`
azaldı; hareket Y değeri `-0.04` olarak kaldı. Bu, FallingBlock client
free-flight dalındaki gravity → move sırasını ve air drag uygulanmadığını
gösterir. Entity ID, dünya entity sayısı ve packet sayısı canlı oturum
state'ine bağlıdır.

## Açık kapsam

FallingBlock block-state/metadata çözümü, collision ve fluid etkileri,
server-side yere oturma ve block placement, bounce/drop/removal lifecycle'ı,
block entity verisi, renderer ve Java client parity'si açık kalır. P08'in
`client-parity` kapanışı da henüz yoktur.
