# P01.d — AABB collidedAlongVector doğrulaması

`src/aabb.rs`, Java `AABB.collidedAlongVector(Vec3, List<AABB>)` yolunu hareketli kutunun merkez segmenti, aday kutuların boyut tabanlı inflate edilmesi, contains kontrolleri ve `clip` sonucu üzerinden uygular. `examples/behavior_adapter.rs` çoklu AABB listesini JSONL math senaryolarına bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- 456 senaryoluk math referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **456/456** math senaryosunda eşleşti.
- Sekiz yeni senaryoda boş liste, başlangıç/bitiş contains, segment geçişi, kaçırma, çoklu aday, epsilon ve NaN davranışları ile negatif inflate miktarı karşılaştırıldı.
- Birleşik altı aile corpus'u **1144/1144** senaryoda hem debug hem release adaptörüyle eşleşti.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `e8e76ccc62aed58e30c8ea04f5862929a6644eaba7132217f369f558775216a5`.

Birleşik referans kimliği: `20a0069e9085f30715b87aa3faae8a6ddc18d0b8a9e38233defa7ee4c21de6a8`.

Yerel raporlar: `compare-doqiofma` (math debug), `compare-jpkfv0nx` (math release), `compare-wess7nxh` (birleşik debug) ve `compare-pqt_xf95` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt `AABB.collidedAlongVector`, mevcut `AABB.clip(Vec3, Vec3)` ve JSONL list adaptörüyle sınırlıdır. Çoklu AABB clip sonraki dilimdir.
