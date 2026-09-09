# P01.d — mergedFaceOccludes ve faceShapeOccludes doğrulaması

`src/shape.rs`, Java `Shapes.mergedFaceOccludes` ve `Shapes.faceShapeOccludes` sorgularını `merged_face_occludes` ve `face_shape_occludes` ile uygular. Canonical block kısa yolu, direction'a göre boundary slice seçimi, fuzzy sınır kontrolü ve birim blokta OR coverage sorgusu aynı BooleanOp yoluna bağlanır. `examples/behavior_adapter.rs` ve `tests/behavior/java/MathOracle.java` aynı JSONL sorgularını Rust ve Java tarafına bağlar. Davranış sınırları [sözleşmede](shape-face-occlusion-contract.md) kayıtlıdır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- 528 senaryoluk math referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri math ailesinde **528/528** eşleşti.
- On bir yeni face occlusion senaryosu empty/block kısa yollarını, tamamlayıcı yüzleri, açık aralığı, boundary sınır boşluklarını ve ters yönü karşılaştırdı.
- Birleşik altı aile corpus'u **1303/1303** senaryoda hem debug hem release adaptörüyle eşleşti.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `29b52a3495f7ab0bf3cecf9bba27102aadc1918ed81f000a8f346929ae1807b2`.

Birleşik referans kimliği: `2934949cd24cd9684ec3034e431974ec99773d6473c977dd6771ec28e892c647`.

Yerel raporlar: `compare-aeud0vo1` (math debug), `compare-v545girn` (math release), `compare-fw1d9dde` (birleşik debug) ve `compare-lx5vlkq_` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt on sentetik `mergedFaceOccludes`/`faceShapeOccludes` sorgusuyla sınırlıdır. Rotation [ayrı kanıtta](shape-rotate-parity.md), diğer merger dalları ve world face/collider toplama sonraki dilimlerde doğrulanacaktır.
