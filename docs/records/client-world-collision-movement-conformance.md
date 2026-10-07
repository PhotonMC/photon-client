# P09.b — Entity collision movement resolution — [client-parity] birleşik kayıt

Bu kayıt, Java istemcisinin `Entity.collideWithShapes` hareket çözüm primitive'ini
Rust'a taşır. Kanıt sınıfı `client-parity`'dir: aynı sentetik hareketli AABB ve
dünya koordinatlarındaki collider shape listesi, Java 26.2 istemci JAR'ında ve
Rust adapter'ında çalıştırılıp karşılaştırılır.

## Java kaynak incelemesi

Java Edition 26.2 / protokol 776 / Java 25 kaynak incelemesinde şu yollar esas
alındı:

- `world/entity/Entity.java:1196-1207`, `collideBoundingBox` hareketli AABB'yi
  genişletilmiş arama alanıyla collider toplama yoluna verir ve ardından
  `collideWithShapes` çağrısını yapar.
- `world/entity/Entity.java:1245-1260`, boş shape listesinde hareketi olduğu
  gibi döndürür; aksi halde sıfır çözülmüş hareketten başlayıp her eksende
  mevcut çözülmüş AABB'yi kullanarak `Shapes.collide` çağrısı yapar.
- `core/Direction.java:379-381`, hareketin Y eksenini önce, X/Z eksenlerini
  `abs(x) < abs(z)` strict karşılaştırmasıyla `YZX` veya `YXZ` sırasına koyar.
- `world/phys/shapes/Shapes.java:234-244` ve `VoxelShape.java:252-307`, shape
  listesini sırayla dolaşır ve hareketli AABB'nin eksen mesafesini sınırlar.

## Sözleşme

- Collider listesi boşsa giriş hareketi aynen döner.
- Dolu listede çözülmüş hareket başlangıçta `(0, 0, 0)`'dır.
- Her eksen için giriş hareketindeki o eksen sıfır değilse, başlangıç AABB'si
  önceki çözülmüş hareketle taşınır ve `Shapes.collide` tüm shape listesine
  giriş eksen mesafesiyle uygulanır.
- Eksen sırası her zaman Y'den başlar; `abs(x) < abs(z)` ise kalan sıra Z, X;
  aksi halde X, Z'dir. Eşitlikte X önce gelir.
- Shape'ler Java `Shapes.collide` sözleşmesindeki iterable sıralarını korur.
- Bu dilim block-state → collision-shape registry çözümünü, block/entity/world
  border collider toplamasını, entity boyut/pose üretimini, step-up adaylarını,
  on-ground ve collision flag'lerini kapsamaz. Bunlar P09'un sonraki dallarıdır.

## Test girdileri ve Java yakalama

Dört özgün sentetik hareket girdisi `tools/behavior/cases.py` içindeki
`shape/collide-movement/*` satırlarında tutulur. Java `MathOracle`, hedef
algoritmayı yeniden yazmadan özgün JAR'daki private `Entity.collideWithShapes`
metodunu reflection ile çağırır. AABB ve shape uçları f64 IEEE-754 bitleriyle
taşınır.

| Senaryo | Amaç | Java çözülmüş hareket |
| --- | --- | --- |
| `empty` | Boş collider listesi | `(1.25, -2.5, 3.75)` |
| `x-then-z` | `abs(x) > abs(z)` ve ikinci eksende daralma | `(2.0, 0.0, 0.5)` |
| `z-then-x` | `abs(z) > abs(x)` ve ters eksen sırası | `(0.5, 0.0, 2.0)` |
| `y-first` | Dikey çarpışmadan sonra yatay çözüm | `(0.25, -1.0, 0.0)` |

Java referans kimliği:
`76cade4f27d4e6b2967592d73c44df8c3f403a332a07f91f0b01c3dcfc4c6ade`.
Bu capture, Java'nın aynı girdilerle iki ardışık çalıştırmasının çıktısını
kararlı verdi.

## Rust doğrulaması

`src/world.rs` içindeki `collide_entity_movement`, Java'nın çözüm başlangıç
değerini, `Direction::axis_step_order` sırasını, taşınmış AABB'yi ve
`Shapes::collide` çağrısını aynı işlem sırasıyla uygular. `shape_result`, aynı
girdiyi production Rust fonksiyonuna taşır. World modülündeki
`collide_entity_movement_follows_java_axis_order` testi dört yolu doğrudan
kontrol eder.

```text
python3 tools/behavior/run.py verify --family math
cargo build --locked --offline --example behavior_adapter
python3 tools/behavior/run.py compare --family math --adapter target/debug/examples/behavior_adapter
```

Math corpus'u 2898 kayıtla doğrulandı; bunların dördü bu dilimin yeni
`collide_movement` girdileridir. Java ve Rust sonuçları **2898/2898** eşleşti.

## Açık kapsam

ClientWorld içindeki block-state registry ve gerçek block collision shape
toplama, entity collider'ları ve world border, entity dimensions/pose, step-up,
on-ground/collision flag state'i, hareket tick entegrasyonu ve vanilla Java
istemcisiyle canlı dünya state karşılaştırması açık kalır.
