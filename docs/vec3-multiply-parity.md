# P01.b — Vec3 multiply overload doğrulaması

`src/math.rs`, Java `Vec3.multiply(Vec3)` ve `Vec3.multiply(double, double, double)` overload'larının bileşen yönlendirmesini ve x/y/z çarpımlarını karşılar. Java oracle overload'ları doğrudan çağırır; Rust adaptörü aynı girdileri üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 741 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **741/741** math senaryosunda eşleşti.
- Beş yeni senaryoda Vec3 ve üç bileşenli overload'lar, overload zinciri, signed zero, taşma ve NaN/Infinity girdileri karşılaştırıldı.
- Birleşik altı aile corpus'u **1195/1195** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1195 senaryoluk Java referansını yeniden doğruladı.

Math referansı `76ec7e6f558a6c9ddbeb7cfe4d528ae99ab6fa83980eeb2c8bda1376494a158f`, birleşik referans `7eb966aeeec47bb875a064f29ee8e61b18ab59b1678b26c179ead90046713129` kimliğindedir.

Yerel raporlar: `compare-bgegqdoc` (math debug), `compare-lkhuntdt` (math release), `compare-crkt4gor` (birleşik debug), `compare-7us5i_vu` (birleşik release) ve `verify-j0qpg9v6` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt beş özgün sentetik Vec3 step dizisi ve vektör snapshot'larıyla sınırlıdır. `scale`, `reverse`, `horizontal`, uzunluk yardımcıları ve diğer Vec3 overload'ları ayrı dilimlerdir.
