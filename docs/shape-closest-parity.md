# P01.d — VoxelShape closestPointTo doğrulaması

`src/shape.rs`, Java `VoxelShape.closestPointTo` sorgusunu `closest_point_to` ile uygular. `clamp_f64` Java double clamp sınırlarını, `to_aabbs` ise Java kutu tarama sırasını taşır. `examples/behavior_adapter.rs` ve `tests/behavior/java/MathOracle.java` aynı JSONL sorgularını Rust ve Java tarafına bağlar. Davranış sınırları [closestPointTo sözleşmesinde](shape-closest-contract.md) kayıtlıdır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- 528 senaryoluk math referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri math ailesinde **528/528** eşleşti.
- Altı yeni closest point senaryosu empty sonucu, iç nokta, eksen/diagonal clamp, OR eşit uzaklıkta ilk adayın korunması ve taşınmış kutu sonucunu karşılaştırdı.
- Birleşik altı aile corpus'u **1010/1010** senaryoda hem debug hem release adaptörüyle eşleşti.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `1209dbe51a271188f3d745fe39b8795d0bed606525a2c1744f5d46c696907e5b`.

Birleşik referans kimliği: `8da9917470c5c593079aa05c0442ec62f5465feedc365b1e8fdaafdd8715563c`.

Yerel raporlar: `compare-jrgkywp4` (math debug), `compare-ocswdo24` (math release), `compare-sq3wafy1` (birleşik debug) ve `compare-1i1dj3ib` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt altı sentetik `closestPointTo` sorgusuyla sınırlıdır. Face/occlusion sorguları, diğer shape merger dalları, world collider toplama ve entity hedefleme sonraki dilimlerde doğrulanacaktır.
