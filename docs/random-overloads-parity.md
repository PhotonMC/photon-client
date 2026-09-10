# P01.e — random yardımcı overload Rust davranış dilimi

`src/random.rs`, Java `RandomSource` varsayılan integer aralıklarını, float/double triangle yardımcılarını ve `consumeCount` tüketim döngüsünü uygular. `examples/behavior_adapter.rs` aynı üretim API'sini JSONL senaryolarına bağlar; beklenen değer üretmez.

## Referans ve davranış

Java `RandomSource` default metotları aralık bound'larını ve origin kaydırmasını i32 semantiğiyle hesaplar. Triangle yolları iki ardışık aynı tür random değerin farkını kullanır; `consumeCount` pozitif tur sayısı kadar `nextInt` tüketir. Rust karşılığı bu işlem sırasını ve float/double sonuç bitlerini korur.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- 340 senaryoluk math referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Debug ve release Rust adaptörleri **340/340** eşleşti: önceki 337 math senaryosu ve 3 yeni random overload senaryosu.
- Altı ailenin birleşik corpus'u **2501/2501** senaryoda hem debug hem release adaptörüyle eşleşti.
- Inclusive ve origin/bound integer sonuçları, float/double triangle bitleri ve `consumeCount` sonrası kaynak konumu karşılaştırıldı.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `fe90194dbcd2294c66724523fa9ff881dae0f6d36ea62df758b4e64626760bf1`.

Birleşik referans kimliği: `838ea33f41ccc8a9281f9614ce843c3bb820ec822d6601ba52ebcb496a9f6040`.

Yerel raporlar: `compare-57rndqkc` (math debug), `compare-uqisv1v5` (math release), `compare-h1kbcaxz` (birleşik debug) ve `compare-js7_mas4` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt legacy `RandomSource` üzerindeki seçili 3 default yardımcı senaryosuyla sınırlıdır. Overflow kaynaklı geçersiz bound hataları, diğer `RandomSource` implementasyonları ve istemcinin daha geniş random tüketim sıraları sonraki testlerdir.
