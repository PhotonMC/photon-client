# P01.d — VoxelShape min/max sütun sorguları doğrulaması

`src/shape.rs`, Java `VoxelShape.min(axis, b, c)` ve `max(axis, b, c)` overload'larını `VoxelShape::min_at` ve `VoxelShape::max_at` ile uygular. `column_axes`, `find_index` ve `is_full_wide` yardımcıları Java'nın `AxisCycle` sütun seçimini, hücre aralığını ve dolu hücre taramasını korur. `examples/behavior_adapter.rs` `axis_limits_at` JSONL yolunu bu API'ye bağlar. Davranış sınırları [referans sözleşmesinde](shape-axis-limits-at-contract.md) kayıtlıdır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 917 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **917/917** math senaryosunda eşleşti.
- Üç yeni senaryoda empty shape, ayrık iki box'ın X/Y/Z sütunları, negatif koordinatlar ve dış sütun için `+∞/−∞` sonuçları karşılaştırıldı.
- Birleşik altı aile corpus'u **1419/1419** senaryoda hem debug hem release adaptörüyle eşleşti.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `cfb0f4a3444412c75186c0d8b209a59632b9d62debcc435aa548bf4c364d8656`.

Birleşik referans kimliği: `cfb0f4a3444412c75186c0d8b209a59632b9d62debcc435aa548bf4c364d8656`.

Yerel raporlar: `compare-dvzelshx` (math debug), `compare-sy72qqem` (math release), `compare-ng1mczig` (birleşik debug) ve `compare-7brjjbpz` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt üç sentetik iki-box shape listesi ve seçilmiş sütun sorgularıyla sınırlıdır. Koordinat listelerinin doğrudan dışa açılması, world collider toplama ve bu sorguların istemci hareketindeki çağrı zinciri sonraki kapsamdır. Global `min(axis)` ve `max(axis)` sorguları [ayrı kanıtta](shape-axis-limits-parity.md) doğrulanır.
