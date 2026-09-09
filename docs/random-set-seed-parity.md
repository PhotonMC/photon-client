# P01.e — `setSeed` Rust davranış dilimi

`src/random.rs`, Java legacy random kaynağının `setSeed` ile LCG durumunu yeniden kurmasını ve Gaussian önbelleğini temizlemesini uygular. `examples/behavior_adapter.rs` aynı üretim API'sini JSONL senaryolarına bağlar; beklenen değer üretmez.

## Referans ve davranış

Java `LegacyRandomSource.setSeed` signed seed'i 48 bitlik başlangıç durumuna dönüştürür ve `MarsagliaPolarGaussian` önbelleğini sıfırlar. Rust constructor'ı da aynı reset yolunu kullanır; sonradan çağrılan `set_seed` eski eş Gaussian'ı korumaz.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- 337 senaryoluk math referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Debug ve release Rust adaptörleri **337/337** eşleşti: önceki 334 math senaryosu ve 3 yeni setSeed senaryosu.
- Altı ailenin birleşik corpus'u **1104/1104** senaryoda hem debug hem release adaptörüyle eşleşti.
- Gaussian önbelleği doluyken aynı seed'e dönme, yeniden seed sonrası yeni LCG başlangıcı ve i64 sınır seed'leri karşılaştırıldı.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `8dc5cad800d19ec1507e2454e97aeb79e53b77267dddada36f77b45771b44f99`.

Birleşik referans kimliği: `9102b9506f63d756b598cc51931df0c5c678c82edfc98fd8f2cf6d70583ae5d8`.

Yerel raporlar: `compare-yxttfhcx` (math debug), `compare-h6jl6e0l` (math release), `compare-o61mnrf6` (birleşik debug) ve `compare-6fi73fsy` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt legacy `setSeed` ve Gaussian cache reset yolunun seçili 3 yeni senaryosuyla sınırlıdır. `nextIntBetweenInclusive`, origin/bound overload'ları, triangle/consumeCount, Xoroshiro/ThreadSafe kaynakları ve istemcinin daha geniş random tüketim sıraları sonraki dilimlerdir.
