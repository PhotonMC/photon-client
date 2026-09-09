# P01.c — SectionPos blok koordinatı yardımcıları doğrulaması

`src/binary.rs`, SectionPos x/y/z section koordinatlarını Java `minBlockX/Y/Z` ve `maxBlockX/Y/Z` sonuçlarıyla aynı blok sınırlarına çevirir. Shift ve maksimum uçtaki `+15` i32 sarma davranışı korunur.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- SectionPos blok koordinatı yardımcıları için 6 özgün senaryo hazırlandı.
- Köken, pozitif/negatif koordinatlar, shift sınırı ve i32 koordinat uçları doğrudan Java/Rust API'leriyle karşılaştırıldı.
- Rust debug ve release adaptörleri güncel **1221/1221** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1723/1723** senaryoda hem debug hem release adaptörüyle eşleşti.
- Math ve birleşik `verify` çalıştırmaları kayıtlı Java referanslarını yeniden doğruladı.

Math referans kimliği: `df8b0fd452c7f12175fe9ced3c534c1f64bdb7df75cc28529becd0eb8ab97b3c`. Birleşik referans kimliği: `86380f621c6f188dddfa1e68a94057a8ada6d638b9f5c04d3a2c9682b1ca75cf`.

Adapter hashleri: debug `7ca32df88b2b9428c0f70c0e484bc8528524a081223d5abc6f31723bb87963f8`, release `0b8de97e52cf25fab7836202ea3b5a09d7db759ce6fab368fea489722e507f79`.

Yerel raporlar: `compare-6dvmyv_h` ve `compare-ak0n9_u3` (math debug/release), `compare-_8esk0jy` ve `compare-g98k9sn0` (birleşik debug/release), `verify-6_r80rf4` (math) ve `verify-72fv37y0` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt, SectionPos section koordinatlarının altı min/max blok sınırı helper'ına çevrilmesiyle sınırlıdır. SectionPos üreticileri, packed erişimcileri, relative konumlar, origin/center/chunk dönüşümleri ve stream codec tüketimi ayrı dilimlerdir. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
