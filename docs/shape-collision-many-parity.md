# P01.d — Shapes.collide çoklu shape doğrulaması

`src/shape.rs`, Java `Shapes.collide` döngüsünü aynı moving AABB ve distance ile shape slice'ı üzerinde verilen sırada uygular. `examples/behavior_adapter.rs`, `shapes` dizisini, moving AABB'yi, ekseni ve başlangıç distance'ını JSONL'e bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- 366 senaryoluk math referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **366/366** math senaryosunda eşleşti.
- Beş yeni senaryoda pozitif/negatif yönde ardışık daraltma, empty shape, sıfır distance erken dönüşü ve boş iterable sonuçları karşılaştırıldı.
- Birleşik altı aile corpus'u **1249/1249** senaryoda hem debug hem release adaptörüyle eşleşti.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `14e57e20448a161506b7554345c087616634381f5b7865b345738ceea2dac811`.

Birleşik referans kimliği: `fc482de613c60d17cab8a4d2e03e66942832cc35cad40eb9f95939d0037ac82f`.

Yerel raporlar: `compare-tdfdd7rh` (math debug), `compare-piprh3re` (math release), `compare-h3m21imi` (birleşik debug) ve `compare-xnouubh6` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt, tek kutudan üretilen shape'lerin `Shapes.collide` iterable döngüsüyle sınırlanmıştır. Birleşik/multi-box shape, face/occlusion, entity collider toplama ve hareket çözüm sırası sonraki dilimlerdir.
