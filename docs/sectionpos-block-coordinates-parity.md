# P01.c — SectionPos blok koordinatı yardımcıları doğrulaması

`src/binary.rs`, SectionPos x/y/z section koordinatlarını Java `minBlockX/Y/Z` ve `maxBlockX/Y/Z` sonuçlarıyla aynı blok sınırlarına çevirir. Shift ve maksimum uçtaki `+15` i32 sarma davranışı korunur.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- SectionPos blok koordinatı yardımcıları için 6 özgün senaryo hazırlandı.
- Köken, pozitif/negatif koordinatlar, shift sınırı ve i32 koordinat uçları doğrudan Java/Rust API'leriyle karşılaştırıldı.
- Rust debug ve release adaptörleri güncel **1638/1638** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2140/2140** senaryoda hem debug hem release adaptörüyle eşleşti.
- Math ve birleşik `verify` çalıştırmaları kayıtlı Java referanslarını yeniden doğruladı.

Math referans kimliği: `18248ddafaffd46fa827971d82704febaf2aa40a94cc0b653078329db1f4cacf`. Birleşik referans kimliği: `1d27ad7d5cb42689c896be0cd7749d1c016b384dacc64f804fb9b17c36a7f38f`.

Adapter hashleri: debug `7ca32df88b2b9428c0f70c0e484bc8528524a081223d5abc6f31856bb87963f8`, release `0b8de97e52cf25fab7836202ea3b5a09d7db759ce6fab368fea489722e507f79`.

Yerel raporlar: `compare-ep8fiu6` ve `compare-z9iwcisf` (math debug/release), `compare-a21c2t4m` ve `compare-uwg2_ewv` (birleşik debug/release), `verify-8mq2_8pm` (math) ve `verify-yscj0_fq` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt, SectionPos section koordinatlarının altı min/max blok sınırı helper'ına çevrilmesiyle sınırlıdır. SectionPos üreticileri, packed erişimcileri, relative konumlar, origin/center/chunk dönüşümleri ve stream codec tüketimi ayrı dilimlerdir. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
