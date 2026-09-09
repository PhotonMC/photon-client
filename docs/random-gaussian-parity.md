# P01.e — `nextGaussian` Rust davranış dilimi

`src/random.rs`, Java `LegacyRandomSource.nextGaussian` yolunun polar örnekleme ve eş sonuç önbelleğini uygular. `examples/behavior_adapter.rs` aynı üretim API'sini JSONL senaryolarına bağlar; beklenen değer üretmez.

## Referans ve davranış

Java `RandomSource.create(seed)` legacy kaynağı seçer. `MarsagliaPolarGaussian` iki `nextDouble` ile birim disk içinde örnek arar, kabul edilen çiftten iki Gaussian sonucu hesaplar ve ikinci sonucu sonraki çağrıya saklar. Rust aynı çağrı sırasını ve sonuçları ham f64 bitleriyle taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- 329 senaryoluk math referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Debug ve release Rust adaptörleri **329/329** eşleşti: 323 önceki math senaryosu ve 6 yeni Gaussian senaryosu.
- Altı ailenin birleşik corpus'u **1497/1497** senaryoda hem debug hem release adaptörüyle eşleşti.
- Sonuç bitleri, önbellekten dönen ikinci değer ve Gaussian çağrıları arasındaki `nextInt`/`nextDouble` tüketim sırası karşılaştırıldı.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `5c0a43bc10b588fbff0d141e430711057cf9e06b60cbc2bd9d6fe4fde4286159`.

Birleşik referans kimliği: `f43f8cbd1b5228b300d98e13e54dbf07d01ae1691a4a16ba16b2e05745e9b4e9`.

Yerel raporlar: `compare-c3rqn2f5` (math debug), `compare-vdvn52ql` (math release), `compare-sdhltqtk` (birleşik debug) ve `compare-af9v4abw` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt legacy `nextGaussian` için seçili 6 yeni senaryoyla sınırlıdır. Diğer random overload'ları ve Gaussian kullanan istemci yolları sonraki dilimlerdir; `setSeed` için ayrı [kanıt](random-set-seed-parity.md), `fork`/`forkPositional` için ayrı [kanıt](random-fork-parity.md) vardır.
