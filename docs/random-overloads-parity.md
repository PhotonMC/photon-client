# P01.e — random yardımcı overload Rust davranış dilimi

`src/random.rs`, Java `RandomSource` varsayılan integer aralıklarını, float/double triangle yardımcılarını ve `consumeCount` tüketim döngüsünü uygular. `examples/behavior_adapter.rs` aynı üretim API'sini JSONL senaryolarına bağlar; beklenen değer üretmez.

## Referans ve davranış

Java `RandomSource` default metotları aralık bound'larını ve origin kaydırmasını i32 semantiğiyle hesaplar. Triangle yolları iki ardışık aynı tür random değerin farkını kullanır; `consumeCount` pozitif tur sayısı kadar `nextInt` tüketir. Rust karşılığı bu işlem sırasını ve float/double sonuç bitlerini korur.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- 340 senaryoluk math referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Debug ve release Rust adaptörleri **340/340** eşleşti: önceki 337 math senaryosu ve 3 yeni random overload senaryosu.
- Altı ailenin birleşik corpus'u **1347/1347** senaryoda hem debug hem release adaptörüyle eşleşti.
- Inclusive ve origin/bound integer sonuçları, float/double triangle bitleri ve `consumeCount` sonrası kaynak konumu karşılaştırıldı.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `fe90194dbcd2294c66724523fa9ff881dae0f6d36ea62df758b4e64626760bf1`.

Birleşik referans kimliği: `2ab36c787d438222d85bab3a2d9318b031512b21c923892daf5b2c0ad183f52c`.

Yerel raporlar: `compare-57rndqkc` (math debug), `compare-uqisv1v5` (math release), `compare-xswxfvez` (birleşik debug) ve `compare-n8easogi` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt legacy `RandomSource` üzerindeki seçili 3 default yardımcı senaryosuyla sınırlıdır. Overflow kaynaklı geçersiz bound hataları, diğer `RandomSource` implementasyonları ve istemcinin daha geniş random tüketim sıraları sonraki testlerdir.
