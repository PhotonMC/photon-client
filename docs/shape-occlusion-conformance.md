# P01.d — Shapes.blockOccludes sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim Java 26.2 `Shapes.blockOccludes(VoxelShape, VoxelShape, Direction)` davranışını kapsar. Tam blok teması, sınır aralığı, kısmi yüz boşluğu, örtülen kısmi yüz, ters yön, boş occluder, dikey temas ve OR birleşimi için sekiz özgün senaryo hazırlanmıştır. Rust sonucu [parite kanıtında](#client-parity-doğrulaması) Java çıktısıyla karşılaştırılmıştır.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/shapes/Shapes.java:246-263`, `VoxelShape.java:29-121,202-235`, `SliceShape.java:6-33` ve `SubShape.java:6-47` incelendi. `MathOracle` doğrudan `blockOccludes` çağrısını gözler; beklenen boolean sonucu Rust sonucundan üretmez.

Bu dilim block-local shape'ler arasındaki tek yönlü block occlusion predicate'ini kapsar. Rotation [ayrı sözleşmede](shape-rotate-conformance.md), dünyadaki yüz/collider toplama sonraki dilimlerde ele alınır.

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

Bu dilimin sekiz senaryosunu içeren math referans kimliği `cb2511839f7c5474e299aa625ddba51129f3c035453190ee147a26361e7a57ef`'dir. Birleşik altı aile referansı `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b` ile doğrulanmıştır. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.


## Client-parity doğrulaması

`src/shape.rs`, Java `Shapes.blockOccludes` predicate'ini `block_occludes` ile uygular. Ortak `slice_at` yardımcı yöntemi yön sınırındaki voxel dilimlerini kurar; `min_coordinate`/`max_coordinate` ve mevcut BooleanOp join yolu Java sınır kontrollerini taşır. `examples/behavior_adapter.rs` ve `tests/behavior/java/MathOracle.java` aynı JSONL sorgularını Rust ve Java tarafına bağlar. Davranış sınırları [blockOccludes sözleşmesinde](#sözleşme) kayıtlıdır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- 528 senaryoluk math referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri math ailesinde **528/528** eşleşti.
- Sekiz yeni occlusion senaryosu tam blok, sınır koşulu, kısmi yüz kapsamı, yön operand sırası, empty occluder, dikey temas ve OR birleşimi sonuçlarını karşılaştırdı.
- Birleşik altı aile corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `cb2511839f7c5474e299aa625ddba51129f3c035453190ee147a26361e7a57ef`.

Birleşik referans kimliği: `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b`.

Yerel raporlar: `compare-_tou3p2b` (math debug), `compare-mkn5lc06` (math release), `compare-5bjkz31s` (birleşik debug) ve `compare-zfkba6ts` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt sekiz sentetik `blockOccludes` sorgusuyla sınırlıdır. Rotation [ayrı kanıtta](shape-rotate-conformance.md), diğer BooleanOp dalları ve world face/collider toplama sonraki dilimlerde doğrulanacaktır.
