# P01.e — `nextGaussian` Rust davranış dilimi

`src/random.rs`, Java `LegacyRandomSource.nextGaussian` yolunun polar örnekleme ve eş sonuç önbelleğini uygular. `examples/behavior_adapter.rs` aynı üretim API'sini JSONL senaryolarına bağlar; beklenen değer üretmez.

## Referans ve davranış

Java `RandomSource.create(seed)` legacy kaynağı seçer. `MarsagliaPolarGaussian` iki `nextDouble` ile birim disk içinde örnek arar, kabul edilen çiftten iki Gaussian sonucu hesaplar ve ikinci sonucu sonraki çağrıya saklar. Rust aynı çağrı sırasını ve sonuçları ham f64 bitleriyle taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- 329 senaryoluk math referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Debug ve release Rust adaptörleri **329/329** eşleşti: 323 önceki math senaryosu ve 6 yeni Gaussian senaryosu.
- Altı ailenin birleşik corpus'u **1479/1479** senaryoda hem debug hem release adaptörüyle eşleşti.
- Sonuç bitleri, önbellekten dönen ikinci değer ve Gaussian çağrıları arasındaki `nextInt`/`nextDouble` tüketim sırası karşılaştırıldı.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `5c0a43bc10b588fbff0d141e430711057cf9e06b60cbc2bd9d6fe4fde4286159`.

Birleşik referans kimliği: `65cbf0b19b2c96be41cf3bbad7266d1e118047e47e641f45d1473f7a741de148`.

Yerel raporlar: `compare-c3rqn2f5` (math debug), `compare-vdvn52ql` (math release), `compare-88a_ixm3` (birleşik debug) ve `compare-sk_17ilr` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt legacy `nextGaussian` için seçili 6 yeni senaryoyla sınırlıdır. Diğer random overload'ları ve Gaussian kullanan istemci yolları sonraki dilimlerdir; `setSeed` için ayrı [kanıt](random-set-seed-parity.md), `fork`/`forkPositional` için ayrı [kanıt](random-fork-parity.md) vardır.
