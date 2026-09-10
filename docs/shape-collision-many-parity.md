# P01.d — Shapes.collide çoklu shape doğrulaması

`src/shape.rs`, Java `Shapes.collide` döngüsünü aynı moving AABB ve distance ile shape slice'ı üzerinde verilen sırada uygular. `examples/behavior_adapter.rs`, `shapes` dizisini, moving AABB'yi, ekseni ve başlangıç distance'ını JSONL'e bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- 366 senaryoluk math referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **366/366** math senaryosunda eşleşti.
- Beş yeni senaryoda pozitif/negatif yönde ardışık daraltma, empty shape, sıfır distance erken dönüşü ve boş iterable sonuçları karşılaştırıldı.
- Birleşik altı aile corpus'u **2549/2549** senaryoda hem debug hem release adaptörüyle eşleşti.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `14e57e20448a185606b7554345c087616634381f5b7865b345738ceea2dac811`.

Birleşik referans kimliği: `510e1d4575f7dd92a09a6d3f896c2b8077a8fb2c01dd57ea32b9b48ce94a7583`.

Yerel raporlar: `compare-tdfdd7rh` (math debug), `compare-piprh3re` (math release), `compare-qz7665of` (birleşik debug) ve `compare-2e2muf1s` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt, tek kutudan üretilen shape'lerin `Shapes.collide` iterable döngüsüyle sınırlanmıştır. Birleşik/multi-box shape, face/occlusion, entity collider toplama ve hareket çözüm sırası sonraki dilimlerdir.
