# P01.d — Shapes.blockOccludes sözleşmesi

Bu dilim Java 26.2 `Shapes.blockOccludes(VoxelShape, VoxelShape, Direction)` davranışını kapsar. Tam blok teması, sınır aralığı, kısmi yüz boşluğu, örtülen kısmi yüz, ters yön, boş occluder, dikey temas ve OR birleşimi için sekiz özgün senaryo hazırlanmıştır. Rust sonucu [parite kanıtında](shape-occlusion-parity.md) Java çıktısıyla karşılaştırılmıştır.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/shapes/Shapes.java:246-263`, `VoxelShape.java:29-121,202-235`, `SliceShape.java:6-33` ve `SubShape.java:6-47` incelendi. `MathOracle` doğrudan `blockOccludes` çağrısını gözler; beklenen boolean sonucu Rust sonucundan üretmez.

Bu dilim block-local shape'ler arasındaki tek yönlü block occlusion predicate'ini kapsar. Rotation [ayrı sözleşmede](shape-rotate-contract.md), dünyadaki yüz/collider toplama sonraki dilimlerde ele alınır.

## Davranış sözleşmesi

- Occluder empty ise sonuç false olur. Her iki shape de tam blok olduğunda sonuç true olur.
- Direction'ın ekseni seçilir. Pozitif yönde ilk shape, negatif yönde occluder ilk shape olarak değerlendirilir; BooleanOp buna göre `ONLY_FIRST` veya `ONLY_SECOND` olur.
- İlk shape'in seçilen eksendeki max değeri 1'e, ikinci shape'in min değeri 0'a `1e-7` fuzzy eşitlik ile ulaşmıyorsa sonuç false olur.
- Sınır voxel'lerinin seçilen eksendeki son/ilk hücreleri ayrı slice olarak karşılaştırılır. İlk slice'ın ikinci tarafından kapatılmayan dolu hücresi varsa sonuç false, yoksa true olur.

## Senaryolar ve JSONL

| Grup | Sayı | Girdiler |
| --- | ---: | --- |
| tam blok | 1 | Aynı tam shape ve doğu yönü |
| eksen boşluğu | 1 | İlk shape max sınırına ulaşmaz |
| kısmi yüz boşluğu | 1 | İki yüz farklı yarıları kaplar |
| kısmi yüz örtüsü | 1 | Occluder ilk yüzü tamamen kapsar |
| ters yön | 1 | Batı yönünde operand sırası |
| empty occluder | 1 | Eşik altı empty occluder |
| dikey temas | 1 | Güney yönünde Z yüzleri |
| OR birleşimi | 1 | İki yarım shape ile birleşik tam yüz |

Girdiler `family: "math"`, `operation: "shape"` ve `kind: "block_occludes"` veya `kind: "block_occludes_join"` taşır. `direction` altı Java yön adından biridir. Sonuç yalnızca `occludes` boolean alanıdır.

## Java yakalama

Depo kökünden:

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py verify --family math
cargo build --locked --offline --example behavior_adapter
python3 tools/behavior/run.py compare --family math --adapter target/debug/examples/behavior_adapter
```

Bu dilimin sekiz senaryosunu içeren math referans kimliği `cb2511839f7c5474e299aa625ddba51129f3c035453190ee147a26361e7a57ef`'dir. Birleşik altı aile referansı `a346583dc8bd7a5d672fe737c86d9febc8a2861cc99d0c486416f41813870b7c` ile doğrulanmıştır. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.
