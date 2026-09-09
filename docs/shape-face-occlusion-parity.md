# P01.d — mergedFaceOccludes ve faceShapeOccludes doğrulaması

`src/shape.rs`, Java `Shapes.mergedFaceOccludes` ve `Shapes.faceShapeOccludes` sorgularını `merged_face_occludes` ve `face_shape_occludes` ile uygular. Canonical block kısa yolu, direction'a göre boundary slice seçimi, fuzzy sınır kontrolü ve birim blokta OR coverage sorgusu aynı BooleanOp yoluna bağlanır. `examples/behavior_adapter.rs` ve `tests/behavior/java/MathOracle.java` aynı JSONL sorgularını Rust ve Java tarafına bağlar. Davranış sınırları [sözleşmede](shape-face-occlusion-contract.md) kayıtlıdır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- 528 senaryoluk math referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri math ailesinde **528/528** eşleşti.
- On bir yeni face occlusion senaryosu empty/block kısa yollarını, tamamlayıcı yüzleri, açık aralığı, boundary sınır boşluklarını ve ters yönü karşılaştırdı.
- Birleşik altı aile corpus'u **1070/1070** senaryoda hem debug hem release adaptörüyle eşleşti.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `29b52a3495f7ab0bf3cecf9bba27102aadc1918ed81f000a8f346929ae1807b2`.

Birleşik referans kimliği: `e5d1fae4e50021d9e6e70de3182785f1dc68c1643d56f5a99d7e5aae91807e14`.

Yerel raporlar: `compare-aeud0vo1` (math debug), `compare-v545girn` (math release), `compare-s3ki0jz5` (birleşik debug) ve `compare-fng7itjt` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt on sentetik `mergedFaceOccludes`/`faceShapeOccludes` sorgusuyla sınırlıdır. Rotation [ayrı kanıtta](shape-rotate-parity.md), diğer merger dalları ve world face/collider toplama sonraki dilimlerde doğrulanacaktır.
