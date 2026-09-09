# P01.c — SectionPos blok koordinatı yardımcıları doğrulaması

`src/binary.rs`, SectionPos x/y/z section koordinatlarını Java `minBlockX/Y/Z` ve `maxBlockX/Y/Z` sonuçlarıyla aynı blok sınırlarına çevirir. Shift ve maksimum uçtaki `+15` i32 sarma davranışı korunur.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- SectionPos blok koordinatı yardımcıları için 6 özgün senaryo hazırlandı.
- Köken, pozitif/negatif koordinatlar, shift sınırı ve i32 koordinat uçları doğrudan Java/Rust API'leriyle karşılaştırıldı.
- Rust debug ve release adaptörleri güncel **1001/1001** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1503/1503** senaryoda hem debug hem release adaptörüyle eşleşti.
- Math ve birleşik `verify` çalıştırmaları kayıtlı Java referanslarını yeniden doğruladı.

Math referans kimliği: `7d65abdf4967bf0f7585e849fc4e6783027eaef1ef313c8b7d22e85922955d4a`. Birleşik referans kimliği: `2411fed740f5d2548545a86cee882fb42daaf0d9fbededf1b224dce54a261a5d`.

Adapter hashleri: debug `7ca32df88b2b9428c0f70c0e484bc8528524a081223d5abc6f31723bb87963f8`, release `0b8de97e52cf25fab7836202ea3b5a09d7db759ce6fab368fea489722e507f79`.

Yerel raporlar: `compare-12czaqmk` ve `compare-fm0qnyjh` (math debug/release), `compare-sakmrbet` ve `compare-pd6md7hs` (birleşik debug/release), `verify-25ncc2dd` (math) ve `verify-5t2xrfy6` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt, SectionPos section koordinatlarının altı min/max blok sınırı helper'ına çevrilmesiyle sınırlıdır. SectionPos üreticileri, packed erişimcileri, relative konumlar, origin/center/chunk dönüşümleri ve stream codec tüketimi ayrı dilimlerdir. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
