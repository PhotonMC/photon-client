# P01.d — VoxelShape collision doğrulaması

`src/shape.rs`, Java `VoxelShape.collide` taramasını shape koordinat düzlemleri ve dolu hücreler üzerinden uygular. `examples/behavior_adapter.rs` tek shape, moving AABB, eksen ve distance girdilerini JSONL'e bağlar. Çoklu shape döngüsü ayrı [kanıtta](shape-collision-many-parity.md) doğrulanır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- 361 senaryoluk math referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **361/361** math senaryosunda eşleşti.
- Sekiz yeni senaryoda pozitif/negatif x, eksen permütasyonu, ortogonal boşluk, sıfır/küçük eşik, empty shape ve sınırlı distance sonuçları karşılaştırıldı.
- Birleşik altı aile corpus'u **2265/2265** senaryoda hem debug hem release adaptörüyle eşleşti.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `b31620fa93e0533f6498b86deedf056a7e4dda293e4fa80ae354dd69541d1ebd`.

Birleşik referans kimliği: `0ad19435d5e4101420352fbb4920646f636d51c748f4cfe43f9a29d2da727cd5`.

Yerel raporlar: `compare-w81fhtwn` (math debug), `compare-o5vdd23o` (math release), `compare-efi7e5qh` (birleşik debug) ve `compare-qw_kmael` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt tek `Shapes.box` girdisinin `VoxelShape.collide` çağrısıyla sınırlıdır. `Shapes.collide` çoklu shape iterasyonu için ayrı [kanıt](shape-collision-many-parity.md) vardır; birleşik/multi-box shape collision, face/occlusion, entity collider toplama ve hareket çözüm sırası sonraki dilimlerdir.
