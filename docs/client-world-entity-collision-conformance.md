# P09.h — Entity collision shape collection — [client-parity] birleşik kayıt

Bu kayıt, Java'nın `EntityGetter.getEntityCollisions` sorgusunu, aday entity
kutularından collision shape listesine kadar taşır. Kanıt sınıfı
`client-parity`'dir: aynı sentetik entity ve AABB girdileri Java 26.2
oracle'ında ve Rust adapter'ında çalıştırılır.

## Java kaynak incelemesi

Java Edition 26.2 / protokol 776 / Java 25 kaynak incelemesinde şu yollar esas
alındı:

- `world/level/EntityGetter.java:56-75`, test alanı boyutu `1.0E-7` altında
  kaldığında boş liste döndürür; aksi hâlde sorgu alanını her yönde
  `1.0E-7` genişletir, seçilen entity'leri sırayla shape'e çevirir.
- `world/entity/EntitySelector.java:17-18`, source yokken `NO_SPECTATORS` ile
  `canBeCollidedWith(null)` birleşimini kullanır.
- `world/entity/Entity.java:2372-2378,3574-3576`, source varken adayın
  collide edilebilirliğini ve aynı araçta olma kapısını belirler.
- `world/phys/AABB.java:189-200,241-267`, inflate, strict intersects ve
  ortalama üç kenar uzunluğundan `getSize` hesabını tanımlar.

## Sözleşme

- `testArea.getSize() < 1.0E-7` ise entity sorgusu yapılmadan boş shape listesi
  döner.
- Diğer durumda broad-phase alanı tüm eksenlerde `1.0E-7` inflate edilir;
  AABB kesişmesi yüz teması kabul etmeyen strict karşılaştırmadır.
- Source yoksa spectator olmayan ve `canBeCollidedWith(null)` sonucu true olan
  adaylar seçilir.
- Source varsa `getEntities(source, ...)` source kimliğini dışarıda bırakır;
  spectator olmayan ve `source.canCollideWith(candidate)` sonucu true olan
  adaylar seçilir. Aynı root vehicle kapısı bu sonuçta elenir.
- Sonuç shape'leri aday sorgusunun giriş sırasını korur ve her kutu Java'nın
  `Shapes.create(entity.getBoundingBox())` yolundan gelir.

## Test girdileri ve Java yakalama

`tools/behavior/cases.py` içindeki üç sentetik kayıt şu dalları taşır:

- küçük test alanının erken dönüşü;
- source yokken filtre ve broad-phase sırası;
- source varken source identity, aynı vehicle, spectator ve collidable
  filtreleri.

Java oracle, gerçek `EntityGetter.getEntityCollisions` default metodunu çağırır.
Entity ve `getEntities` davranışı yalnızca kontrollü fixture olarak sağlanır;
beklenen shape listesi Rust çıktısından türetilmez.

## Rust doğrulaması

`src/world.rs` içindeki `collect_entity_collision_shapes`, aynı erken dönüş,
epsilon'lı sorgu alanı, strict AABB filtresi, source/vehicle/spectator
kapıları ve aday sırasını uygular. Adapter bu production helper'ı kullanır ve
her seçilen entity kutusunu `Shapes::create_from_aabb` ile shape'e dönüştürür.

Java referans kimliği:
`44a3f322a7ab5e88365035e1012c9829d32783a5561fbcde3a10a61576278882`.

Math corpus'undaki 2919 kayıt, Java'nın iki ardışık capture çıktısında kararlı
kaldı. Java `verify` başarılı oldu. Rust debug adapter SHA-256 değeri
`04636edcdfce50641ea23eb8c9466ddfe9b89d3b478ee3d71a4874027fe8f6bb`, release
adapter SHA-256 değeri
`e151dc10594f6ae7f74a179c7e4d5d0c3b6678069bc7c98e11bed0ea327ad464` ile iki
karşılaştırma da **2919/2919** eşleşti.

```text
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py verify --family math
cargo build --locked --offline --example behavior_adapter
python3 tools/behavior/run.py compare --family math --adapter target/debug/examples/behavior_adapter
cargo build --locked --offline --release --example behavior_adapter
python3 tools/behavior/run.py compare --family math --adapter target/release/examples/behavior_adapter
```

## Açık kapsam

Gerçek `ClientWorld` entity index'i ve canlı entity state'inden adayların
toplanması, block/entity/world-border shape'lerinin tek collision sorgusunda
birleştirilmesi ve bu listenin `Entity.collide` akışına bağlanması sonraki
P09 dilimleridir. Entity yaşam durumu, removal, gerçek `canBeCollidedWith`
varyantları ve world-border shape'i bu fixture helper'ının kapsamına girmez.

Bu kayıt canlı sunucu kabulü değildir; Java istemcisiyle sentetik entity
collision shape sonuçlarının karşılaştırılmasıdır.
