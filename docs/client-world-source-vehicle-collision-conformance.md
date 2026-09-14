# P09.q — ClientWorld source vehicle pushable collision — [client-parity] birleşik kayıt

Bu kayıt, `ClientWorld` entity collision sorgusunda source entity'nin runtime
collision dispatch'ini taşır. Vanilla boat source'u, `canBeCollidedWith` sonucu
false olan fakat `isPushable` sonucu true olan Player adayını collision shape'i
olarak kabul eder; base Entity source aynı adayı kabul etmez. Kanıt sınıfı
`client-parity`'dir.

## Java kaynak incelemesi

Java Edition 26.2 / protokol 776 / Java 25 kaynak incelemesinde şu yollar esas
alındı:

- `world/level/EntityGetter.java:56-75`, non-null source için entity selector'ı
  source'un `canCollideWith` metoduna bağlar ve seçilen bounding box'ları shape'e
  çevirir.
- `world/entity/Entity.java:2372-2378`, base `canCollideWith` yalnız adayın
  `canBeCollidedWith(source)` sonucunu ve aynı araç filtresini kullanır; base
  entity `canBeCollidedWith` ve `isPushable` kapıları kapalıdır.
- `world/entity/vehicle/boat/AbstractBoat.java:109-125`, boat collision
  dispatch'i adayın `canBeCollidedWith(source) || isPushable()` sonucunu
  değerlendirir ve boat'ın kendi collider/pushable kapılarını açar.
- `world/entity/EntityTypes.java` içindeki oak boat ve player kayıtları ile
  `world/entity/EntityType.java:340-342`, type registry dimensions'ını
  feet-origin AABB'e dönüştürür. Pinned type ID'leri oak boat için `0`, Player
  için `156`'dır.
- `world/level/entity/TransientEntitySectionManager.java` ve
  `LevelEntityGetter`, P09.m'de kurulan section üyeliği ve AABB lookup yolunu
  sağlar.

## Sözleşme

- Non-null source bir vanilla boat ise, query alanına giren adayın
  `canBeCollidedWith` sonucu false olsa bile `isPushable` true olduğunda adayın
  registry dimensions kutusu collision'a katılır.
- Base Entity source için aynı pushable aday, `canBeCollidedWith` false olduğu
  için collision'a katılmaz.
- Player adayının pinned standing dimensions'ı `0.6 × 1.8` ve feet-origin
  konumu kullanılır. Source kendi ID'siyle section'da bulunsa da sonuçtan
  çıkarılır.
- Testteki `can_be_collided_with=false` alanı Java fixture'ında açıkça
  korunur; Boat sonucunun `isPushable` OR dalından geldiğini gösterir.

## Test girdileri ve Java yakalama

`tools/behavior/cases.py` içindeki `source-vehicle-pushable` kaydı iki gözlem
taşır: oak boat source'unun pushable Player'ı kabul etmesi ve base Player
source'unun aynı adayı reddetmesi. Java oracle, source için gerçek Boat runtime
class'ını, aday için `isPushable()` override'ı olan kontrollü fixture'ı ve
gerçek transient section lookup'ını kullanır. Beklenen çıktılar Java
istemcisinden alınmıştır.

Java referans kimliği:
`ef20910e857b49c9abd27457459b5494808c28d6051f157f326f88e5f565ba1f`.

Math corpus'undaki 2928 kayıt iki ardışık Java capture çıktısında kararlı kaldı.
Rust release adapter SHA-256 değeri
`4b07ed3d387ac4318fe11837bae29a6952f7e5ebd14a0265c1272ab22b618940` ile
**2928/2928** eşleşti.

```text
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py verify --family math
cargo build --locked --offline --release --example behavior_adapter
python3 tools/behavior/run.py compare --family math --adapter target/release/examples/behavior_adapter
```

Rust tarafında source type-aware collector, pushable candidate state'i ve
pinned Player dimensions'ı `ClientWorld::no_entity_collision_for_entity`
yoluna bağlandı. Eski source'suz/sentetik collector API'si aynı filtre
sonucunu korur.

## Açık kapsam

Minecart'in aynı `AbstractBoat.canVehicleCollide` statik dalını kullanan source
dispatch'i, tam EntityType registry/context, yaşayan entity'nin gerçek
`isPushable` koşulları (spectator, climbable ve game-mode state'i), diğer source
override'ları, kalıcı spatial section güncellemeleri ve çözülen movement AABB'i
açık kaldı. Bu kayıt canlı sunucu kabulü değildir; vanilla Java istemcisiyle
sentetik `ClientWorld` entity state karşılaştırmasıdır.
