# P01.b — Vec3 multiply overload doğrulaması

`src/math.rs`, Java `Vec3.multiply(Vec3)` ve `Vec3.multiply(double, double, double)` overload'larının bileşen yönlendirmesini ve x/y/z çarpımlarını karşılar. Java oracle overload'ları doğrudan çağırır; Rust adaptörü aynı girdileri üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1025 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1025/1025** math senaryosunda eşleşti.
- Beş yeni senaryoda Vec3 ve üç bileşenli overload'lar, overload zinciri, signed zero, taşma ve NaN/Infinity girdileri karşılaştırıldı.
- Birleşik altı aile corpus'u **1527/1527** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1527 senaryoluk Java referansını yeniden doğruladı.

Math referansı `2c79e2aaba1154782177df83cd39488c1c9a365c76e1971733e837c447fe9fa3`, birleşik referans `74f88dfa13fb755a92b838953181016d177ef3a05b113b5fa8ef1f3735031acc` kimliğindedir.

Yerel raporlar: `compare-0dsoh6yn` (math debug), `compare-wdh5yokb` (math release), `compare-7wace8y9` (birleşik debug), `compare-ohni39xi` (birleşik release) ve `verify-a7iqdyvs` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt beş özgün sentetik Vec3 step dizisi ve vektör snapshot'larıyla sınırlıdır. `scale`, `reverse`, `horizontal`, uzunluk yardımcıları ve diğer Vec3 overload'ları ayrı dilimlerdir.
