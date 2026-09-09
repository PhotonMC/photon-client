# P01.d — VoxelShape min/max sütun sorguları doğrulaması

`src/shape.rs`, Java `VoxelShape.min(axis, b, c)` ve `max(axis, b, c)` overload'larını `VoxelShape::min_at` ve `VoxelShape::max_at` ile uygular. `column_axes`, `find_index` ve `is_full_wide` yardımcıları Java'nın `AxisCycle` sütun seçimini, hücre aralığını ve dolu hücre taramasını korur. `examples/behavior_adapter.rs` `axis_limits_at` JSONL yolunu bu API'ye bağlar. Davranış sınırları [referans sözleşmesinde](shape-axis-limits-at-contract.md) kayıtlıdır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 953 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **953/953** math senaryosunda eşleşti.
- Üç yeni senaryoda empty shape, ayrık iki box'ın X/Y/Z sütunları, negatif koordinatlar ve dış sütun için `+∞/−∞` sonuçları karşılaştırıldı.
- Birleşik altı aile corpus'u **1455/1455** senaryoda hem debug hem release adaptörüyle eşleşti.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `e267ec135794f878c9e54d22faa3451ffe125c25ad17b7ad67e223c1d1fb3b55`.

Birleşik referans kimliği: `e267ec135794f878c9e54d22faa3451ffe125c25ad17b7ad67e223c1d1fb3b55`.

Yerel raporlar: `compare-j1r42nvx` (math debug), `compare-yls3sbu1` (math release), `compare-ttkrbj5a` (birleşik debug) ve `compare-dz8xzceq` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt üç sentetik iki-box shape listesi ve seçilmiş sütun sorgularıyla sınırlıdır. Koordinat listelerinin doğrudan dışa açılması, world collider toplama ve bu sorguların istemci hareketindeki çağrı zinciri sonraki kapsamdır. Global `min(axis)` ve `max(axis)` sorguları [ayrı kanıtta](shape-axis-limits-parity.md) doğrulanır.
