# P01.d — Shapes.blockOccludes doğrulaması

`src/shape.rs`, Java `Shapes.blockOccludes` predicate'ini `block_occludes` ile uygular. Ortak `slice_at` yardımcı yöntemi yön sınırındaki voxel dilimlerini kurar; `min_coordinate`/`max_coordinate` ve mevcut BooleanOp join yolu Java sınır kontrollerini taşır. `examples/behavior_adapter.rs` ve `tests/behavior/java/MathOracle.java` aynı JSONL sorgularını Rust ve Java tarafına bağlar. Davranış sınırları [blockOccludes sözleşmesinde](shape-occlusion-contract.md) kayıtlıdır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- 528 senaryoluk math referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri math ailesinde **528/528** eşleşti.
- Sekiz yeni occlusion senaryosu tam blok, sınır koşulu, kısmi yüz kapsamı, yön operand sırası, empty occluder, dikey temas ve OR birleşimi sonuçlarını karşılaştırdı.
- Birleşik altı aile corpus'u **1031/1031** senaryoda hem debug hem release adaptörüyle eşleşti.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `cb2511839f7c5474e299aa625ddba51089f3c035453190ee147a26361e7a57ef`.

Birleşik referans kimliği: `cc0f402235532cc834549c53fd069f064a8e771ef6ba6e2943e472b1d9b6b06d`.

Yerel raporlar: `compare-_tou3p2b` (math debug), `compare-mkn5lc06` (math release), `compare-rpvlapca` (birleşik debug) ve `compare-dwzkalq0` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt sekiz sentetik `blockOccludes` sorgusuyla sınırlıdır. Rotation [ayrı kanıtta](shape-rotate-parity.md), diğer BooleanOp dalları ve world face/collider toplama sonraki dilimlerde doğrulanacaktır.
