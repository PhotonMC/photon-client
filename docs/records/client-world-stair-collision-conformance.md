# P09.o — ClientWorld stair collision shapes — [client-parity] birleşik kayıt

Bu kayıt, oak stairs state'lerinin `ClientWorld` block collision sorgusunda
tek kutu yerine Java'nın birleşik iki veya üç kutulu shape'leri üretmesini
taşır. Kanıt sınıfı `client-parity`'dir: aynı pinned state ID'leri, shape box
sırası ve world koordinatındaki collision sonuçları Java oracle'ında ve Rust
adapter'ında karşılaştırılır.

## Java kaynak incelemesi

Java Edition 26.2 / protokol 776 / Java 25 kaynak incelemesinde şu yollar esas
alındı:

- `world/level/block/StairBlock.java:38-45`, straight, outer ve inner temel
  shape'lerini kurar; bottom/top tabloları yatay dönüş ve üst yarı dönüşüyle
  üretir.
- `world/level/block/StairBlock.java:54-88`, default state'in facing, half,
  shape ve waterlogged alanlarını kurar; collision shape'i `SHAPE` ve `HALF`
  dallarından seçip facing'e göre döndürür.
- `world/level/block/StairBlock.java:124-163`, komşu stairs state'lerinden
  outer/inner shape kararını üretir. Bu dilim state ID'siyle zaten kodlanmış
  shape'i çözer; komşu güncelleme yolu sonraki kapsamdır.
- `world/level/block/Block.java:127-138` ve
  `world/level/BlockCollisions.java:55-103`, state ID fallback'ini ve çözülen
  shape'in block position'a taşınmasını sağlar.

## Sözleşme

- `ClientBlockStateRegistry`, stairs için Java registry'de gözlenen
  `3908` (top straight), `3918` (bottom straight), `3920` (bottom inner-left)
  ve `3926` (bottom outer-right) state ID'lerini tutar.
- Her stairs state'i, Java `Shapes.or` ile birleşen kutu listesi olarak
  saklanır. Liste sırası `VoxelShape.toAabbs` snapshot'ında korunur; shape
  block konumuna taşındığında aynı kutu geometrisi world koordinatına eklenir.
- P09.n'deki bilinmeyen ID fallback'i ve mevcut cursor, epsilon, strict
  kesişim ve `noBlockCollision` kuralları bu state'lerde de geçerlidir.

## Test girdileri ve Java yakalama

`tools/behavior/cases.py` içindeki `stair-state-shapes` satırı dört stairs
state'ini ve bir bilinmeyen ID'yi aynı query alanında yerleştirir. Java oracle
gerçek `Block.stateById`, `BlockState.getCollisionShape` ve
`CollisionGetter.getBlockCollisions` yollarını kullanır. Beklenen sonuç Rust
çıktısından türetilmez.

## Rust doğrulaması

Registry shape descriptor'ı birden fazla ordered box taşıyacak şekilde
genişletildi. `ClientWorld::collect_registered_block_collision_shapes` bu
descriptor'ı Java shape join sırasıyla VoxelShape'e çevirip mevcut block
traversal'ına verir. Adapter production registry ve world APIsini kullanır.

Java referans kimliği:
`0e2dff4874235ce2e05db88be1278df68f7e794082f97e3dfa0823de022a7ef2`.

Math corpus'undaki 2926 kayıt, Java'nın iki ardışık capture çıktısında kararlı
kaldı. Java `verify` başarılı oldu. Rust debug adapter SHA-256 değeri
`cc7c63b3e5aec38b5155431751153bb723a6da9481d08bef19bd3e37ea2abc24`, release
adapter SHA-256 değeri
`f910ca010d26be6e01a3bf5bac2ef0bb4b492cb6a7c362b1ca6c8f2836da3288` ile iki
karşılaştırma da **2926/2926** eşleşti.

```text
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py verify --family math
cargo build --locked --offline --example behavior_adapter
python3 tools/behavior/run.py compare --family math --adapter target/debug/examples/behavior_adapter
cargo build --locked --offline --release --example behavior_adapter
python3 tools/behavior/run.py compare --family math --adapter target/release/examples/behavior_adapter
```

## Açık kapsam

Komşu block state'lerinden `StairBlock.getStairsShape` ile shape güncellemesi,
facing/half/waterlogged kombinasyonlarının tamamı, diğer stairs türleri,
tam 32366 state registry, collision context ve dynamic shape verisi açık
kaldı. Bu kayıt canlı sunucu kabulü değildir; vanilla Java istemcisiyle
sentetik `ClientWorld` state karşılaştırmasıdır.
