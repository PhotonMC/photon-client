# P01.d — Shapes.blockOccludes doğrulaması

`src/shape.rs`, Java `Shapes.blockOccludes` predicate'ini `block_occludes` ile uygular. Ortak `slice_at` yardımcı yöntemi yön sınırındaki voxel dilimlerini kurar; `min_coordinate`/`max_coordinate` ve mevcut BooleanOp join yolu Java sınır kontrollerini taşır. `examples/behavior_adapter.rs` ve `tests/behavior/java/MathOracle.java` aynı JSONL sorgularını Rust ve Java tarafına bağlar. Davranış sınırları [blockOccludes sözleşmesinde](shape-occlusion-contract.md) kayıtlıdır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- 511 senaryoluk math referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri math ailesinde **511/511** eşleşti.
- Sekiz yeni occlusion senaryosu tam blok, sınır koşulu, kısmi yüz kapsamı, yön operand sırası, empty occluder, dikey temas ve OR birleşimi sonuçlarını karşılaştırdı.
- Birleşik altı aile corpus'u **965/965** senaryoda hem debug hem release adaptörüyle eşleşti.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `cb2511839f7c5474e299aa625ddba51089f3c035453190ee147a26361e7a57ef`.

Birleşik referans kimliği: `2105d50b3ac69a505c2685a91ad23c0857f85a9601098d2f6ab2b966e92919ef`.

Yerel raporlar: `compare-_tou3p2b` (math debug), `compare-mkn5lc06` (math release), `compare-4xmx_z9_` (birleşik debug) ve `compare-y1pppv24` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt sekiz sentetik `blockOccludes` sorgusuyla sınırlıdır. `mergedFaceOccludes`, `faceShapeOccludes`, rotation, diğer BooleanOp dalları ve world face/collider toplama sonraki dilimlerde doğrulanacaktır.
