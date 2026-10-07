# P01.d — mergedFaceOccludes ve faceShapeOccludes sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim Java 26.2 `Shapes.mergedFaceOccludes(VoxelShape, VoxelShape, Direction)` ve `Shapes.faceShapeOccludes(VoxelShape, VoxelShape)` davranışını kapsar. Boş yüzler, tam blok kısa yolu, tamamlayıcı yüzler, yüz aralığı ve birleşik yüz sınırları için on bir özgün senaryo hazırlanmıştır. Rust sonucu [parite kanıtında](#client-parity-doğrulaması) Java çıktısıyla karşılaştırılmıştır.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/shapes/Shapes.java:265-301`, `VoxelShape.java:29-121,202-235`, `SliceShape.java:6-33` ve `SubShape.java:6-47` incelendi. `MathOracle` iki Java predicate'ini doğrudan çağırır; beklenen boolean sonucu Rust sonucundan üretmez.

Bu dilim iki block-local face shape'in birim bloğu birlikte kapatıp kapatmadığını kapsar. Rotation [ayrı sözleşmede](shape-rotate-conformance.md), world face/collider toplama ve diğer merger dalları sonraki dilimlerde ele alınır.

## Davranış sözleşmesi

- `mergedFaceOccludes` için iki operand da canonical block değilse direction ekseni ve işareti operand sırasını belirler. İlk operandın max sınırı 1'e fuzzy eşit değilse, ikinci operandın min sınırı 0'a fuzzy eşit değilse ilgili operand empty sayılır.
- Sınır operandlarının son/ilk slice'ları OR ile birleştirilir. Bu birleşim birim blokta açık hücre bırakmıyorsa sonuç true olur; yalnızca bir operandın sınırdan uzak olması, diğer operandın tam blok olması halinde sonucu false yapmaz.
- `faceShapeOccludes` iki operanddan biri canonical block ise true, ikisi de empty ise false döner. Diğer durumda operandların OR birleşimi birim bloğu tamamen kapatıyorsa true döner.
- Her iki predicate de BooleanOp `ONLY_FIRST` ile birim blokta union dışında kalan hücreyi arar; açık hücre bulunması sonucu false yapar.

## Senaryolar ve JSONL

| Grup | Sayı | Girdiler |
| --- | ---: | --- |
| face empty | 1 | Eşik altı iki sliver |
| face block kısa yolu | 2 | Block ilk veya ikinci operand |
| face tamamlayıcı | 1 | Y eksenindeki yüz yarılarının OR birleşimi |
| face aralık | 1 | Arada boşluk bırakan iki yüz |
| face derinlik | 1 | Z ekseninde tamamlayıcı iki yarı |
| merged tamamlayıcı | 1 | Doğu sınırında iki tamamlayıcı slice |
| merged block kısa yolu | 1 | İlk operand canonical block |
| merged ilk sınır boşluğu | 1 | İlk operand sınırdan uzak, ikinci tam blok |
| merged ikinci sınır boşluğu | 1 | İkinci operand sınırdan uzak |
| merged ters yön | 1 | Batı yönünde operand sırası |

Girdiler `family: "math"`, `operation: "shape"` ve `kind: "face_occludes"` veya `kind: "merged_occludes"` taşır. `merged_occludes` için `direction` altı Java yön adından biridir. Sonuç yalnızca `occludes` boolean alanıdır.

## Java yakalama

Depo kökünden:

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py verify --family math
cargo build --locked --offline --example behavior_adapter
python3 tools/behavior/run.py compare --family math --adapter target/debug/examples/behavior_adapter
```

Bu dilimin on bir senaryosunu içeren math referans kimliği `29b52a3495f7ab0bf3cecf9bba27102aadc1918ed81f000a8f346929ae1807b2`'dir. Birleşik altı aile referansı `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b` ile doğrulanmıştır. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.


## Client-parity doğrulaması

`crates/voxel-core/src/shape.rs`, Java `Shapes.mergedFaceOccludes` ve `Shapes.faceShapeOccludes` sorgularını `merged_face_occludes` ve `face_shape_occludes` ile uygular. Canonical block kısa yolu, direction'a göre boundary slice seçimi, fuzzy sınır kontrolü ve birim blokta OR coverage sorgusu aynı BooleanOp yoluna bağlanır. `examples/behavior_adapter.rs` ve `tests/behavior/java/MathOracle.java` aynı JSONL sorgularını Rust ve Java tarafına bağlar. Davranış sınırları [sözleşmede](#sözleşme) kayıtlıdır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- 528 senaryoluk math referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri math ailesinde **528/528** eşleşti.
- On bir yeni face occlusion senaryosu empty/block kısa yollarını, tamamlayıcı yüzleri, açık aralığı, boundary sınır boşluklarını ve ters yönü karşılaştırdı.
- Birleşik altı aile corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `29b52a3495f7ab0bf3cecf9bba27102aadc1918ed81f000a8f346929ae1807b2`.

Birleşik referans kimliği: `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b`.

Yerel raporlar: `compare-aeud0vo1` (math debug), `compare-v545girn` (math release), `compare-5bjkz31s` (birleşik debug) ve `compare-zfkba6ts` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt on sentetik `mergedFaceOccludes`/`faceShapeOccludes` sorgusuyla sınırlıdır. Rotation [ayrı kanıtta](shape-rotate-conformance.md), diğer merger dalları ve world face/collider toplama sonraki dilimlerde doğrulanacaktır.
