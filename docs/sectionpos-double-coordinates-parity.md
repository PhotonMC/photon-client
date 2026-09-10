# P01.c — SectionPos double koordinat doğrulaması

`src/binary.rs`, SectionPos `posToSectionCoord(double)` ve `blockToSectionCoord(double)` overload'larını Java'nın floor, narrowing cast ve arithmetic shift sonuçlarıyla eşleştirir.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- SectionPos double koordinatları için 6 özgün senaryo hazırlandı.
- İki overload; pozitif/negatif sınırlar, subnormal, en büyük sonlu değer ve NaN girdileriyle doğrudan Java/Rust API'leriyle karşılaştırıldı.
- Rust debug ve release adaptörleri güncel **1843/1843** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2345/2345** senaryoda hem debug hem release adaptörüyle eşleşti.
- Math ve birleşik `verify` çalıştırmaları kayıtlı Java referanslarını iki JVM tekrarıyla yeniden doğruladı.

Math referans kimliği: `dfeb55b1b61cbc7649df7ae8af88469cbf7480a194892aacbcf8e5e5b93806bd`. Birleşik referans kimliği: `95b56d3ef2ba474460f7f24c5d4b696ec8041e8c1baffa192876f9bf0b0f3576`.

Adapter hashleri: debug `c3f51b7830e3c0554138ff428a979a8f54ffaf85f7e2082bc6962bdcf01ff1d9`, release `fa350ee97911ed79b9b6db86a8af51e8b7dc87401e8312957b94741266b4b51b`.

Yerel raporlar: `compare-5_avciij` (math debug), `compare-e6e6agr4` (math release), `compare-gvo3eind` (birleşik debug), `compare-wmabbl5t` (birleşik release), `verify-zbtpetmh` (math) ve `verify-vzenwlxd` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt, iki SectionPos double overload'unun altı son int çıktısıyla sınırlıdır. `Mth.floor(float)`, `Mth.lfloor(double)`, diğer SectionPos overload'ları ve gerçek istemci kullanım zincirleri ayrı kapsamdır. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
