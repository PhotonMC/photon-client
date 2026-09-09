# P01.c — SectionPos blok koordinatı yardımcıları doğrulaması

`src/binary.rs`, SectionPos x/y/z section koordinatlarını Java `minBlockX/Y/Z` ve `maxBlockX/Y/Z` sonuçlarıyla aynı blok sınırlarına çevirir. Shift ve maksimum uçtaki `+15` i32 sarma davranışı korunur.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- SectionPos blok koordinatı yardımcıları için 6 özgün senaryo hazırlandı.
- Köken, pozitif/negatif koordinatlar, shift sınırı ve i32 koordinat uçları doğrudan Java/Rust API'leriyle karşılaştırıldı.
- Rust debug ve release adaptörleri güncel **1013/1013** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1515/1515** senaryoda hem debug hem release adaptörüyle eşleşti.
- Math ve birleşik `verify` çalıştırmaları kayıtlı Java referanslarını yeniden doğruladı.

Math referans kimliği: `5ce242e52e41e1869ca609121bb59c3acf18c2e75ba193e14bf67ead2c12ed7b`. Birleşik referans kimliği: `b6d98de9faec99fa12618268a20692921e10931fc251b87964810964c0de2933`.

Adapter hashleri: debug `7ca32df88b2b9428c0f70c0e484bc8528524a081223d5abc6f31723bb87963f8`, release `0b8de97e52cf25fab7836202ea3b5a09d7db759ce6fab368fea489722e507f79`.

Yerel raporlar: `compare-dsrceuzh` ve `compare-2cvy6vt1` (math debug/release), `compare-5omvt90q` ve `compare-orlrlnqo` (birleşik debug/release), `verify-4bc3l5fr` (math) ve `verify-wa8bpydo` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt, SectionPos section koordinatlarının altı min/max blok sınırı helper'ına çevrilmesiyle sınırlıdır. SectionPos üreticileri, packed erişimcileri, relative konumlar, origin/center/chunk dönüşümleri ve stream codec tüketimi ayrı dilimlerdir. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
