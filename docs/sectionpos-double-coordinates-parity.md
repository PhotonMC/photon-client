# P01.c — SectionPos double koordinat doğrulaması

`src/binary.rs`, SectionPos `posToSectionCoord(double)` ve `blockToSectionCoord(double)` overload'larını Java'nın floor, narrowing cast ve arithmetic shift sonuçlarıyla eşleştirir.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- SectionPos double koordinatları için 6 özgün senaryo hazırlandı.
- İki overload; pozitif/negatif sınırlar, subnormal, en büyük sonlu değer ve NaN girdileriyle doğrudan Java/Rust API'leriyle karşılaştırıldı.
- Rust debug ve release adaptörleri güncel **1398/1398** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1900/1900** senaryoda hem debug hem release adaptörüyle eşleşti.
- Math ve birleşik `verify` çalıştırmaları kayıtlı Java referanslarını iki JVM tekrarıyla yeniden doğruladı.

Math referans kimliği: `4cc46837e07b2a435f9a79ffddebd09d7fed0ad74e1310e64453de08cb4bf4bd`. Birleşik referans kimliği: `dd8cd9014d22178671ef078ce966d61e97e820f7256f6c8a55c5e30c2f1ec4ea`.

Adapter hashleri: debug `7fcaaf04b4fa67068cb95022cc7e832f9013a10846c57ad8dee850949ac64f59`, release `b98e59cbf911f59510b79a9d182e31d455c93c6084ae18d3ce74b07bab212466`.

Yerel raporlar: `compare-xljwmsb7` (math debug), `compare-s_oqvkq3` (math release), `compare-uf5y7y3a` (birleşik debug), `compare-wrj9a6ex` (birleşik release), `verify-_l31ex7g` (math) ve `verify-5ju3ok97` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt, iki SectionPos double overload'unun altı son int çıktısıyla sınırlıdır. `Mth.floor(float)`, `Mth.lfloor(double)`, diğer SectionPos overload'ları ve gerçek istemci kullanım zincirleri ayrı kapsamdır. `.data/` altındaki Java runtime, capture ve türetilmiş çıktılar test girdisi olarak yerel kalır.
