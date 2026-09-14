# P09.p — ClientWorld shulker collision — [client-parity] birleşik kayıt

Bu kayıt, `ClientWorld` entity index'inde shulker type'ının taban `Entity`
davranışından ayrılan collider kapısını taşır. Kanıt sınıfı `client-parity`'dir:
aynı type ID, konum ve query AABB'si Java'nın transient section lookup yolunda ve
Rust `ClientWorld` adapter'ında karşılaştırılır.

## Java kaynak incelemesi

Java Edition 26.2 / protokol 776 / Java 25 kaynak incelemesinde şu yollar esas
alındı:

- `world/level/EntityGetter.java:54-75`, null source collision sorgusunda
  `EntitySelector.CAN_BE_COLLIDED_WITH` filtresini seçer ve geçen entity'nin
  bounding box'ını shape'e çevirir.
- `world/entity/EntitySelector.java:18`, spectator filtresine ek olarak
  `canBeCollidedWith(null)` sonucunu ister.
- `world/entity/Entity.java:2376-2378`, taban entity'yi collider olmaktan
  çıkarır.
- `world/entity/monster/Shulker.java:472-476`, shulker'ın canlı olduğu sürece
  `canBeCollidedWith` override'ı ile bu kapıyı açar.
- `world/entity/EntityTypes.java:844-847` ve
  `world/entity/EntityType.java:340-342`, shulker type ID `112` için
  `1.0F × 1.0F` dimensions değerini ve feet-origin bounding box üretimini
  sağlar.
- `world/level/entity/TransientEntitySectionManager.java` ile
  `LevelEntityGetter`, P09.m'deki section yerleşimi ve AABB sorgusu yolunu
  korur.

## Sözleşme

- Type ID `112`, registry dimensions'ından `1.0 × 1.0` entity collider kutusu
  üretir; kutunun merkezi entity konumundadır ve Y başlangıcı feet konumudur.
- Null source sorgusunda shulker, base `Entity` gibi sessizce elenmez;
  `Shulker.canBeCollidedWith` override'ı nedeniyle kesişen query'de collision
  sonucu üretir.
- Query dışındaki shulker section index'ten döndürülmez ve collision sonucu
  üretmez. P09.m'de doğrulanan source exclusion ve entity count kuralları
  aynen geçerlidir.
- Senaryodaki `can_be_collided_with=false` alanı, Java fixture'ının genel
  aday kapısını değil gerçek shulker override'ını kullandığını görünür kılar;
  shulker dalında bu alan beklenen sonucu değiştirmez.

## Test girdileri ve Java yakalama

`tools/behavior/cases.py` içindeki `indexed-shulker-state` kaydı iki gözlem
taşır: query ile kesişen canlı shulker override'ı ve query dışında kalan
shulker. Java oracle, dimensions kutusunu gerçek registry'den alır ve gerçek
`Shulker.canBeCollidedWith` metodunu çalıştıran kontrollü bir shulker fixture'ı
transient entity section manager'a ekler. Fixture yalnız canlılık durumunu
sabitler; collision kararı shulker sınıfının override'ından gelir.

Java referans kimliği:
`c88d3618aa8bbef9483c4e2a86aae1fed67359e6508cd458be8ed0a6e7d11115`.

Math corpus'undaki 2927 kayıt, Java'nın iki ardışık capture çıktısında kararlı
kaldı. Java `verify` başarılı oldu. Rust debug adapter SHA-256 değeri
`a906918e907b097d2aabdffab084fb11eea215793d4323f151b6ae258e800d66`, release
adapter SHA-256 değeri
`74653e9656f400784fedb7ece32ffb0fbe11f9b3120452f006ee283e15817375` ile iki
karşılaştırma da **2927/2927** eşleşti.

```text
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py verify --family math
cargo build --locked --offline --example behavior_adapter
python3 tools/behavior/run.py compare --family math --adapter target/debug/examples/behavior_adapter
cargo build --locked --offline --release --example behavior_adapter
python3 tools/behavior/run.py compare --family math --adapter target/release/examples/behavior_adapter
```

## Açık kapsam

Tam EntityType registry/context, diğer özel `canBeCollidedWith` dalları
(`HappyGhast`), source type'ına bağlı `isPushable` davranışı, spectator ve
game-mode state'i, dimensions refresh/scale ve kalıcı spatial section index
güncellemeleri açık kaldı. Bu kayıt canlı sunucu kabulü değildir; vanilla Java
istemcisiyle sentetik `ClientWorld` entity state karşılaştırmasıdır.
