# P09.n — ClientWorld block-state registry/context — [client-parity] birleşik kayıt

Bu kayıt, chunk içindeki global block-state ID'sinin `ClientWorld` içindeki
vanilla 26.2 collision shape çözümüne bağlanmasını taşır. Kanıt sınıfı
`client-parity`'dir: aynı state ID, bilinmeyen ID fallback'i ve konumlandırılmış
block collision sorgusu Java oracle'ında ve Rust adapter'ında karşılaştırılır.

## Java kaynak incelemesi

Java Edition 26.2 / protokol 776 / Java 25 kaynak incelemesinde şu yollar esas
alındı:

- `world/level/block/Block.java:82,127-138`, global `BlockState` ID mapper'ını,
  `getId` çözümünü ve bilinmeyen ID'yi air default state'e döndüren
  `stateById` fallback'ini tanımlar.
- `world/level/block/Blocks.java:5997-6003`, built-in block registry içindeki
  her block'un mümkün state'lerini global state mapper'ına ekler.
- `world/level/block/state/StateDefinition.java:26-121`, property
  kombinasyonlarından ordered possible state listesini kurar.
- `world/level/block/state/BlockBehaviour.java:323-340,686-692,922-954`,
  collision shape'in block davranışından ve boş collision context'ten
  çözülmesini, ayrıca state cache'ini yürütür.
- `world/level/block/SlabBlock.java:43-60`, slab TYPE state'ine göre bottom,
  top ve double shape dallarını seçer.
- `world/level/BlockCollisions.java:19-110` ve
  `world/level/CollisionGetter.java:39-80`, çözülen state shape'ini cursor
  sırasıyla world koordinatına taşıyıp `noBlockCollision` yoluna verir.

## Sözleşme

- `ClientWorld`, chunk palette/state ID'lerinin sorgulandığı ayrı bir
  `ClientBlockStateRegistry` taşır.
- Bu dilimin pinned 26.2 tablosu air `0`, stone `1`, oak slab top `13331`,
  bottom `13333` ve double `13335` state'lerini gerçek Java registry ID'leriyle
  çözer. Bilinmeyen ID'ler Java `Block.stateById` fallback'i gibi air `0` ve
  boş collision shape üretir.
- Registry üzerinden toplanan shape'ler mevcut `BlockCollisions` eşiklerini,
  cursor sırasını, tam blok hızlı yolunu, partial shape kesişimini ve world
  koordinatına taşımayı korur.
- State çözümleme ve collision toplama aynı `ClientWorld` state'inden yapılır;
  adapter'a bağımsız shape map'i geçirilmez.

## Test girdileri ve Java yakalama

`tools/behavior/cases.py` içindeki tek parametrik kayıt, beş yerleşim üzerinde
altı state ID sorgular: air, stone, üç slab state'i ve bilinmeyen ID. Java
oracle gerçek `Block.stateById`, `Block.getId`, `BlockState.getCollisionShape`
ve `CollisionGetter.getBlockCollisions` çağrılarını kullanır. Beklenen sonuç
Rust çıktısından türetilmez.

## Rust doğrulaması

`ClientWorld::new` pinned registry'yi oluşturur. `block_state_registry` state
çözümünü dışarı verir; `collect_registered_block_collision_shapes` chunk
state'lerini bu registry'den alınan shape'lerle mevcut block collision
traversal'ına bağlar. Adapter, bu iki production API'sini kullanır.

Java referans kimliği:
`4e71d0d387c0cab942b8c52c782a6d74f2ea519661df5b3ab52363cd2deb1bfc`.

Math corpus'undaki 2925 kayıt, Java'nın iki ardışık capture çıktısında kararlı
kaldı. Java `verify` başarılı oldu. Rust debug adapter SHA-256 değeri
`737f25df0ed4ad0c6803d5ee144a7252e8a18364a23976ac26392a4556e223a5`, release
adapter SHA-256 değeri
`f9ebaaad780eaebcfee655170bf918b59af6ce411a748ca6cafe4e5e3de3d902` ile iki
karşılaştırma da **2925/2925** eşleşti.

```text
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py verify --family math
cargo build --locked --offline --example behavior_adapter
python3 tools/behavior/run.py compare --family math --adapter target/debug/examples/behavior_adapter
cargo build --locked --offline --release --example behavior_adapter
python3 tools/behavior/run.py compare --family math --adapter target/release/examples/behavior_adapter
```

## Açık kapsam

Java built-in registry'sinin tamamı bu dilimde kopyalanmadı; pinned runtime
probe'u toplam 32366 possible block state bulunduğunu gösterdi. Tam state
tablosu, stairs ve komşu state'e bağlı shape'ler, fluids, collision context,
block entity/dynamic shape verisi ve registry güncelleme yolu açık kaldı.
Bu kayıt canlı sunucu kabulü değildir; vanilla Java istemcisiyle sentetik
`ClientWorld` state karşılaştırmasıdır.
