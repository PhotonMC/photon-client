# P01.c — SectionPos blok koordinatı yardımcıları doğrulaması

`src/binary.rs`, SectionPos x/y/z section koordinatlarını Java `minBlockX/Y/Z` ve `maxBlockX/Y/Z` sonuçlarıyla aynı blok sınırlarına çevirir. Shift ve maksimum uçtaki `+15` i32 sarma davranışı korunur.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- SectionPos blok koordinatı yardımcıları için 6 özgün senaryo hazırlandı.
- Köken, pozitif/negatif koordinatlar, shift sınırı ve i32 koordinat uçları doğrudan Java/Rust API'leriyle karşılaştırıldı.
- Rust debug ve release adaptörleri güncel **1843/1843** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2345/2345** senaryoda hem debug hem release adaptörüyle eşleşti.
- Math ve birleşik `verify` çalıştırmaları kayıtlı Java referanslarını yeniden doğruladı.

Math referans kimliği: `dfeb55b1b61cbc7649df7ae8af88469cbf7480a194892aacbcf8e5e5b93806bd`. Birleşik referans kimliği: `95b56d3ef2ba474460f7f24c5d4b696ec8041e8c1baffa192876f9bf0b0f3576`.

Adapter hashleri: debug `7ca32df88b2b9428c0f70c0e484bc8528524a081223d5abc6f31856bb87963f8`, release `0b8de97e52cf25fab7836202ea3b5a09d7db759ce6fab368fea489722e507f79`.

Yerel raporlar: `compare-5_avciij` ve `compare-e6e6agr4` (math debug/release), `compare-gvo3eind` ve `compare-wmabbl5t` (birleşik debug/release), `verify-zbtpetmh` (math) ve `verify-vzenwlxd` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt, SectionPos section koordinatlarının altı min/max blok sınırı helper'ına çevrilmesiyle sınırlıdır. SectionPos üreticileri, packed erişimcileri, relative konumlar, origin/center/chunk dönüşümleri ve stream codec tüketimi ayrı dilimlerdir. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
