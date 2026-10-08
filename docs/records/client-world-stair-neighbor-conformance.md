# P09.aa — ClientWorld StairBlock komşu shape güncellemesi — [client-parity] birleşik kayıt

Bu kayıt, yüklü `ClientWorld` içindeki bir oak stairs state'inin yatay komşu
güncellemesi sonrası Java `StairBlock.getStairsShape` dallarına göre yeni state
ID'sine geçirilmesini ve güncel collision shape'inin sorgulanmasını taşır.
Kanıt sınıfı `client-parity`'dir. Canlı vanilla bağlantısı aşağıda yalnızca
world teslimi için ayrı adjunct olarak tutulur.

## Java kaynak incelemesi

Java Edition 26.2 / protokol 776 / Java 25 kaynak incelemesinde şu yollar esas
alındı:

- `world/level/block/StairBlock.java:100-108`, placement state'inin facing,
  half ve waterlogged alanlarını kurduktan sonra shape'i komşulardan çözer.
- `world/level/block/StairBlock.java:111-129`, yatay komşu güncellemesinde
  `SHAPE` değerini yeniler; waterlogged state için su tick'i planlar.
- `world/level/block/StairBlock.java:131-163`, arka komşuda outer, ön
  komşuda inner shape dallarını; facing axis, half ve `canTakeShape` kapılarıyla
  seçer. Uygun komşu yoksa straight state'e döner.
- `world/level/block/Block.java:127-139`, state ID çözümünü ve bilinmeyen ID'nin
  air fallback'ini belirler.
- `world/level/BlockCollisions.java:55-103`, seçilen collision shape'ini
  world koordinatına taşıyıp `noBlockCollision` sorgusuna verir.

## Davranış sözleşmesi

- Pinned oak stairs state'leri target için north/bottom/straight dry `3918` ve
  waterlogged `3917`; komşu gözlemleri west/bottom/straight `3958`,
  east/bottom/straight `3978` ve north/top/straight `3908`'dir.
- Arka komşunun counter-clockwise facing'i `OUTER_LEFT` (`3924`), clockwise
  facing'i `OUTER_RIGHT` (`3926`) üretir. Ön komşular aynı ayrımı
  `INNER_LEFT` (`3920`) ve `INNER_RIGHT` (`3922`) olarak üretir.
- Half eşleşmezse arka/ön branch çalışmaz. `canTakeShape` yönündeki aynı
  facing ve aynı half stairs state'i de outer/inner seçimini engeller.
- Waterlogged target, shape seçimini değiştirmez; sonuç state ID'si waterlogged
  ailede korunur. Bu kayıt oak, north target ve pinned komşu subset'iyle
  sınırlıdır.

## Test girdileri ve Java yakalama

`tools/behavior/cases.py` içindeki
`world/client-collision/stair-neighbor-shape-updates` kaydı yedi gözlem taşır:

| Gözlem | Başlangıç | Komşu düzeni | Java güncel state/shape |
| --- | ---: | --- | --- |
| `behind-outer-left` | `3918` | behind west | `3924` / `outer_left` |
| `behind-outer-right` | `3918` | behind east | `3926` / `outer_right` |
| `front-inner-left` | `3918` | front west | `3920` / `inner_left` |
| `front-inner-right` | `3918` | front east | `3922` / `inner_right` |
| `half-mismatch-keeps-straight` | `3918` | behind north top | `3918` / `straight` |
| `can-take-shape-blocked` | `3918` | behind west + east same-facing stairs | `3918` / `straight` |
| `waterlogged-outer-left` | `3917` | behind west | `3923` / `outer_left` |

Java oracle gerçek `Block.stateById`, `StairBlock` private
`getStairsShape` branch'i, `BlockState.setValue(SHAPE, ...)`, collision shape
ve `CollisionGetter.getBlockCollisions` yollarını kullanır. Her input'ta
komşular yüklendikten sonra hedefe yatay update uygulanır; beklenen çıktı Rust
sonucundan türetilmez.

Java referans kimliği:
`5df0643685ad729e91a6f4ae7aac91198b781acf63f4f3f06a89ea92972f216d`.

Math corpus'u iki Java capture çalışmasında kararlı kaldı; toplam 2938 kayıt
karşılaştırıldı.

## Rust doğrulaması

`ClientBlockStateRegistry::vanilla_26_2` bu dilimde kullanılan pinned stairs
state ID'lerini shape descriptor ve facing/half/waterlogged metadata'sıyla
tutar. `stair_shape_update_state_id`, Java'nın arka/ön ve `canTakeShape`
önceliğini altı yüklü komşu üzerinden uygular. `ClientWorld::apply_stair_neighbor_shape_update`
sonucu yüklü section hücresine yazar; ardından normal registered block
collision traversal güncel state ID'sinin kutularını world koordinatına taşır.

Debug adapter SHA-256 değeri
`3029552dfc46858fed28ef85a9c19c879af84c24148bab07208c237cdddc1e63` ile
**2938/2938** eşleşti; fark `null` kaldı.
Release adapter SHA-256 değeri
`afb0ea0b3e630bc9663dcdc8dee508b0764a91e34388cb1032ce233071321fd8` ile
aynı capture üzerinde **2938/2938** eşleşti; fark `null` kaldı.

Çalıştırılan kontroller:

```text
python3 tools/behavior/run.py capture --family math --jdk-home /tmp/photon-jdk-wrapper
python3 tools/behavior/run.py verify --family math --jdk-home /tmp/photon-jdk-wrapper
cargo test --locked --offline client_world_updates_stair_shape_from_loaded_neighbors
cargo check --locked --offline --all-targets
cargo clippy --locked --offline --all-targets -- -D warnings
cargo build --locked --offline --example behavior_adapter
python3 tools/behavior/run.py compare --family math --jdk-home /tmp/photon-jdk-wrapper --adapter target/debug/examples/behavior_adapter
cargo build --locked --offline --release --example behavior_adapter
python3 tools/behavior/run.py compare --family math --jdk-home /tmp/photon-jdk-wrapper --adapter target/release/examples/behavior_adapter
```

## Vanilla server adjunct smoke

Mojang vanilla 26.2 server jar SHA-1'i
`823e2250d24b3ddac457a60c92a6a941943fcd6a` olan yerel server
`127.0.0.1:25568` üzerinde `LiveStairs` adıyla çalıştırıldı. Rust client
bağlantı, play login, world construction, chunk/section teslimi ve tick
akışını gözledi. Bu koşu synthetic stairs state update'ini server üzerinden
yeniden üretmez; bu nedenle `server-conformance` adjunct'ıdır ve Java
istemcisiyle stairs parity kanıtı olarak kullanılmaz.

## Açık kapsam

Tam 32366 block-state registry/context, target facing ve half kombinasyonları,
diğer stairs türleri, placement click/fluid context'i, water tick scheduling,
gerçek clientbound block update dispatch'i, dynamic collision context'i ve
stair dışı block state aileleri bu kaydın dışındadır.
