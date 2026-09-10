# P01.e — `fork` ve positional random Rust davranış dilimi

`src/random.rs`, Java legacy random kaynağının `fork` ve `forkPositional` yollarını uygular. Positional factory koordinat, UTF-16 string hash'i ve doğrudan seed türevleri üretir. `examples/behavior_adapter.rs` aynı üretim API'sini JSONL senaryolarına bağlar; beklenen değer üretmez.

## Referans ve davranış

Java `LegacyRandomSource.fork` parent kaynaktan bir `nextLong` çekip yeni kaynağı bu signed seed ile kurar. `forkPositional` aynı çekimi factory seed'i olarak saklar. `at` Java `Mth.getSeed` aritmetik taşma sırasını, `fromHashOf` Java UTF-16 `String.hashCode()` değerini, `fromSeed` ise factory seed'inden bağımsız doğrudan seed'i kullanır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- 334 senaryoluk math referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Debug ve release Rust adaptörleri **334/334** eşleşti: önceki 329 math senaryosu ve 5 yeni fork/positional senaryosu.
- Altı ailenin birleşik corpus'u **2036/2036** senaryoda hem debug hem release adaptörüyle eşleşti.
- Parent `nextLong` tüketimi, child kaynak dizileri, i32 koordinat sınırları, Java UTF-16 hash'i ve Gaussian child sonuçları karşılaştırıldı.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `aed4472ff0ba0e85f3a9ae2fe61132e8abf7af4145d0d28fcac8a64bb06fedb4`.

Birleşik referans kimliği: `5607afc8c59b5ee3393ed20e95d9fbca8ce40846df423d5e52589b902d90b90f`.

Yerel raporlar: `compare-vzri6usr` (math debug), `compare-03cd0hof` (math release), `compare-tgx0qoj4` (birleşik debug) ve `compare-t1bsi_04` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt legacy `fork` ve positional factory'nin seçili 5 yeni senaryosuyla sınırlıdır. Xoroshiro/ThreadSafe kaynakları, `Identifier` overload'ları ve istemcinin daha geniş random tüketim sıraları sonraki dilimlerdir; `setSeed` sonrası Gaussian cache temizleme için ayrı [kanıt](random-set-seed-parity.md) vardır.
