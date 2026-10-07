# P09.q — ClientWorld source vehicle pushable collision — [client-parity] birleşik kayıt

Bu kayıt, `ClientWorld` entity collision sorgusunda source entity'nin runtime
collision dispatch'ini taşır. Vanilla boat ve minecart source'ları,
`canBeCollidedWith` sonucu false olan fakat `isPushable` sonucu true olan
adayları collision shape'i olarak kabul eder; base Entity source aynı adayları
kabul etmez. Kanıt sınıfı `client-parity`'dir.

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
- `world/entity/vehicle/minecart/AbstractMinecart.java:162-169`, minecart
  collision dispatch'ini aynı `AbstractBoat.canVehicleCollide` statik yardımcısına
  devreder ve minecart'ı pushable olarak bildirir.
- `world/entity/EntityTypes.java` içindeki oak boat ve player kayıtları ile
  minecart kayıtları ve `world/entity/EntityType.java:340-342`, type registry
  dimensions'ını feet-origin AABB'e dönüştürür. Pinned type ID'leri oak boat
  için `0`, minecart ailesi için `25, 29, 56, 65, 85, 122, 134`, Player için
  `156`'dır; minecart ailesinin ölçüsü `0.98 × 0.7`'dir.
- `world/level/entity/TransientEntitySectionManager.java` ve
  `LevelEntityGetter`, P09.m'de kurulan section üyeliği ve AABB lookup yolunu
  sağlar.

## Sözleşme

- Non-null source bir vanilla boat ise, query alanına giren adayın
  `canBeCollidedWith` sonucu false olsa bile `isPushable` true olduğunda adayın
  registry dimensions kutusu collision'a katılır.
- Non-null source bir vanilla minecart ise aynı `canVehicleCollide` dalı
  çalışır. Minecart ailesinin yedi pinned EntityType ID'si aynı
  `0.98 × 0.7` registry dimensions kutusunu ve pushable aday kapısını kullanır.
- Base Entity source için aynı pushable aday, `canBeCollidedWith` false olduğu
  için collision'a katılmaz.
- Player adayının pinned standing dimensions'ı `0.6 × 1.8` ve feet-origin
  konumu kullanılır. Source kendi ID'siyle section'da bulunsa da sonuçtan
  çıkarılır.
- Testteki `can_be_collided_with=false` alanı Java fixture'ında açıkça
  korunur; Boat ve Minecart sonuçlarının `isPushable` OR dalından geldiğini
  gösterir.

## Test girdileri ve Java yakalama

`tools/behavior/cases.py` içindeki `source-vehicle-pushable` kaydı dört gözlem
taşır: oak boat source'unun pushable Player'ı kabul etmesi, base Player
source'unun aynı adayı reddetmesi, minecart source'unun pushable Player'ı
kabul etmesi ve boat source'unun minecart adayını kabul etmesi. Java oracle,
source için gerçek Boat/Minecart runtime class'larını, adaylar için
`isPushable()` override'ı olan kontrollü fixture'ları ve gerçek transient
section lookup'ını kullanır. Beklenen çıktılar Java istemcisinden alınmıştır.

Java referans kimliği:
`03a25797cb760e61a3eb8faaf6a84c66946f9017684652e877ef4dfc1361985f`.

Math corpus'undaki 2930 kayıt iki ardışık Java capture çıktısında kararlı kaldı.
Rust release adapter SHA-256 değeri
`8238a32152641f6ffe2ec6f2a3df3c6af5942d206b7f301f96dc80afaf3e5508` ile
**2930/2930** eşleşti.

```text
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py verify --family math
cargo build --locked --offline --release --example behavior_adapter
python3 tools/behavior/run.py compare --family math --adapter target/release/examples/behavior_adapter
```

Rust tarafında source type-aware collector, boat/minecart pushable candidate
state'i ve pinned Player/minecart dimensions'ı
`ClientWorld::no_entity_collision_for_entity` yoluna bağlandı. Eski
source'suz/sentetik collector API'si aynı filtre sonucunu korur.

## Açık kapsam

Tam EntityType registry/context, yaşayan entity'nin gerçek
`isPushable` koşulları (spectator, climbable ve game-mode state'i), diğer source
override'ları, kalıcı spatial section güncellemeleri ve çözülen movement AABB'i
açık kaldı. Bu kayıt canlı sunucu kabulü değildir; vanilla Java istemcisiyle
sentetik `ClientWorld` entity state karşılaştırmasıdır.
