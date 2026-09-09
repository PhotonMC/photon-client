# P01.d — VoxelShape min/max sütun sorguları doğrulaması

`src/shape.rs`, Java `VoxelShape.min(axis, b, c)` ve `max(axis, b, c)` overload'larını `VoxelShape::min_at` ve `VoxelShape::max_at` ile uygular. `column_axes`, `find_index` ve `is_full_wide` yardımcıları Java'nın `AxisCycle` sütun seçimini, hücre aralığını ve dolu hücre taramasını korur. `examples/behavior_adapter.rs` `axis_limits_at` JSONL yolunu bu API'ye bağlar. Davranış sınırları [referans sözleşmesinde](shape-axis-limits-at-contract.md) kayıtlıdır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1113 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1113/1113** math senaryosunda eşleşti.
- Üç yeni senaryoda empty shape, ayrık iki box'ın X/Y/Z sütunları, negatif koordinatlar ve dış sütun için `+∞/−∞` sonuçları karşılaştırıldı.
- Birleşik altı aile corpus'u **1615/1615** senaryoda hem debug hem release adaptörüyle eşleşti.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `747c3645458af2ae3b9de41407461996409d550b8fca05e990e18b3b1983f461`.

Birleşik referans kimliği: `747c3645458af2ae3b9de41407461996409d550b8fca05e990e18b3b1983f461`.

Yerel raporlar: `compare-y28xdodh` (math debug), `compare-_7cfq3in` (math release), `compare-cxv1an3r` (birleşik debug) ve `compare-1jjs0aun` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt üç sentetik iki-box shape listesi ve seçilmiş sütun sorgularıyla sınırlıdır. Koordinat listelerinin doğrudan dışa açılması, world collider toplama ve bu sorguların istemci hareketindeki çağrı zinciri sonraki kapsamdır. Global `min(axis)` ve `max(axis)` sorguları [ayrı kanıtta](shape-axis-limits-parity.md) doğrulanır.
