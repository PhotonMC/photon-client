# P09.s — Happy Ghast collider override — [client-parity] birleşik kayıt

Bu kayıt, Java Edition 26.2'de Happy Ghast'ın kaynak entity türüne ve kendi
durumuna göre değişen `canBeCollidedWith` sonucunun `ClientWorld` entity
collision sorgusuna bağlandığını kanıtlar. Kanıt sınıfı `client-parity`'dir;
canlı sunucu kabulü değildir.

## Java kaynak incelemesi

İncelenen temel yollar:

- `world/entity/animal/happyghast/HappyGhast.java:79-81,576-585`, yetişkin
  ölçüsünü `4.0F × 4.0F` kurar; baby ölçüsü temel ölçünün `0.2375F` ile
  ölçeklenmiş halidir. Collider override'ı baby veya ölü entity'yi reddeder.
- Aynı override, yaşayan yetişkin için client tarafında kaynak bir `Player` ve
  oyuncu konumu entity box'ının üst sınırında veya üzerindeyse true döndürür;
  diğer durumda yalnızca kaynak da Happy Ghast ise adayın vehicle olmasını,
  aksi halde still-timeout durumunu kullanır.
- `world/level/EntityGetter.java:54-75` sorgu alanını `1.0E-7` genişletir,
  null source için adayın kendi collider kararını, non-null source için
  source'un `canCollideWith` dispatch'ini kullanır ve seçilen kutuları shape'e
  dönüştürür.
- `world/entity/Entity.java:2372-2378`, source dispatch'inin adayın
  `canBeCollidedWith` sonucu ve aynı araç filtresiyle kurulduğunu gösterir;
  `world/entity/vehicle/boat/AbstractBoat.java:109-125` source boat yolunda
  adayın `isPushable` istisnasını ekler. `LivingEntity.java:3371-3375`
  yaşayan, spectator olmayan ve tırmanmayan entity'lerin pushable olabildiği
  tabanı sağlar.
- `world/entity/EntityTypes.java:496-500`, Happy Ghast registry ölçüsünü ve
  `world/entity/player/Player.java` registry'siyle birlikte source türü
  ayrımını sağlar.

## Davranış sözleşmesi

- Happy Ghast adayı baby ise veya canlı değilse, still-timeout ve source türü
  ne olursa olsun collision adayı sayılmaz.
- Yaşayan yetişkin aday, client-side Player source'un Y konumu kendi bounding
  box'ının `maxY` değerine eşit veya büyükse collision adayı sayılır. Eşitlik
  sınırı strict değildir.
- Bu üstten oyuncu dalı çalışmıyorsa aday yalnızca source Happy Ghast ve aday
  vehicle ise veya aday still-timeout durumundaysa collision adayı sayılır.
- Null source sorgusu still-timeout gibi adayın kendi override sonucunu kullanır;
  source boat sorgusu, override false olsa bile yaşayan Happy Ghast'ın
  `isPushable` sonucunu kullanır.
- Yetişkin box feet-origin `4.0 × 4.0`, baby box `0.2375F` ölçekli aynı
  ölçüdür. Entity section index, source exclusion, epsilon broad-phase ve
  block/entity/world-border birleşimi mevcut `ClientWorld` yolunda korunur.

## Test girdileri ve Java yakalama

`world/client-collision/happy-ghast-collider-override` kaydı tek parametrik
oracle girdisinde şu gözlemleri taşır: yetişkin still-timeout, serbest yetişkin,
baby, ölü yetişkin, bounding box üst sınırında ve altında Player, vehicle olan
ve olmayan Happy Ghast source'ları ve yaşayan Happy Ghast'ı sorgulayan source
boat. Her gözlem hem `noEntityCollision` hem birleşik `noCollision` çıktısını
üretir.

Java oracle gerçek `TransientEntitySectionManager<Entity>` ve gerçek section
getter'ını kullanır. Happy Ghast, Player ve Boat fixture'ları yalnız gerekli
durumları kontrollü biçimde değiştirir; entity türü registry ölçüleri ve
collision dispatch'i Java'dan gelir. Beklenen çıktı Rust adapter'ından
türetilmemiştir.

Java referans kimliği:
`ee3cd492f7223707bc44b0279259861ef36709ac7cd6cd33bfb797d818c2fbdb`.

Math corpus'u iki ardışık Java capture çıktısında kararlı kaldı; toplam 2930
kayıt karşılaştırıldı. Rust debug ve release adapter'ları her ikisinde de
**2930/2930** eşleşti, fark `null` kaldı.

```text
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py verify --family math
cargo build --locked --offline --example behavior_adapter
python3 tools/behavior/run.py compare --family math --adapter target/debug/examples/behavior_adapter
cargo build --locked --offline --release --example behavior_adapter
python3 tools/behavior/run.py compare --family math --adapter target/release/examples/behavior_adapter
```

Rust tarafında `ClientWorld` Happy Ghast türünü section-indexed collision
adayına ekler; adult/baby ölçüsünü, alive ve still-timeout state'ini, source
türünü ve source Y konumunu Java override sırasıyla değerlendirir. Yaşayan
Happy Ghast'ın source boat sorgusundaki pushable yolu da aynı aday modeline
bağlanır.

## Açık kapsam

Bu dilim Happy Ghast collider override'ının section-indexed sorgudaki
collision kararını kapatır. Gerçek paket metadata'sından Happy Ghast baby,
still-timeout, vehicle ve lifecycle state'lerinin tüm güncelleme yolları,
tam `LivingEntity.isPushable` koşulları, Minecart source dispatch'i ve diğer
özel collidable entity türleri sonraki P08/P09 dilimleridir.
