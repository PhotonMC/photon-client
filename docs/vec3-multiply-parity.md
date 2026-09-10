# P01.b — Vec3 multiply overload doğrulaması

`src/math.rs`, Java `Vec3.multiply(Vec3)` ve `Vec3.multiply(double, double, double)` overload'larının bileşen yönlendirmesini ve x/y/z çarpımlarını karşılar. Java oracle overload'ları doğrudan çağırır; Rust adaptörü aynı girdileri üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1506 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1506/1506** math senaryosunda eşleşti.
- Beş yeni senaryoda Vec3 ve üç bileşenli overload'lar, overload zinciri, signed zero, taşma ve NaN/Infinity girdileri karşılaştırıldı.
- Birleşik altı aile corpus'u **2008/2008** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2008 senaryoluk Java referansını yeniden doğruladı.

Math referansı `521c517003287d78e7dee051e04062b28c1fdcf7965213cb691cb7e1e0a3f9df`, birleşik referans `8a534e59c23e5da8152cd10fc0f559f6e05cac44a30f5600b40c9a6b5a45ce57` kimliğindedir.

Yerel raporlar: `compare-wn40y3iw` (math debug), `compare-9phkvs2k` (math release), `compare-a415_00_` (birleşik debug), `compare-djb85w4w` (birleşik release) ve `verify-nqxvpkjm` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt beş özgün sentetik Vec3 step dizisi ve vektör snapshot'larıyla sınırlıdır. `scale`, `reverse`, `horizontal`, uzunluk yardımcıları ve diğer Vec3 overload'ları ayrı dilimlerdir.
