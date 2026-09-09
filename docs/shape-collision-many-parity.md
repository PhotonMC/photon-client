# P01.d — Shapes.collide çoklu shape doğrulaması

`src/shape.rs`, Java `Shapes.collide` döngüsünü aynı moving AABB ve distance ile shape slice'ı üzerinde verilen sırada uygular. `examples/behavior_adapter.rs`, `shapes` dizisini, moving AABB'yi, ekseni ve başlangıç distance'ını JSONL'e bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- 366 senaryoluk math referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **366/366** math senaryosunda eşleşti.
- Beş yeni senaryoda pozitif/negatif yönde ardışık daraltma, empty shape, sıfır distance erken dönüşü ve boş iterable sonuçları karşılaştırıldı.
- Birleşik altı aile corpus'u **1028/1028** senaryoda hem debug hem release adaptörüyle eşleşti.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `14e57e20448a161506b7554345c087616634381f5b7865b345738ceea2dac811`.

Birleşik referans kimliği: `973124eeb6072de7c2a8f42e33b01304c2a5e79e2ef0a87968d085bb7084a0ee`.

Yerel raporlar: `compare-tdfdd7rh` (math debug), `compare-piprh3re` (math release), `compare-4pe6qeij` (birleşik debug) ve `compare-94buxmhn` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt, tek kutudan üretilen shape'lerin `Shapes.collide` iterable döngüsüyle sınırlanmıştır. Birleşik/multi-box shape, face/occlusion, entity collider toplama ve hareket çözüm sırası sonraki dilimlerdir.
