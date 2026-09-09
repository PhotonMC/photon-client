# P01.b — Vec3 reverse doğrulaması

`src/math.rs`, Java `Vec3.reverse()` metodunun `scale(-1.0)` yönlendirmesini karşılar. Java oracle doğrudan çağrıyı yapar; Rust adaptörü aynı girdiyi `Vec3::reverse` API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 741 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **741/741** math senaryosunda eşleşti.
- Beş yeni senaryoda sonlu bileşenler, ardışık işlem, signed zero, subnormal, büyük sonlu ve Infinity girdileri karşılaştırıldı.
- Birleşik altı aile corpus'u **1195/1195** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1195 senaryoluk Java referansını yeniden doğruladı.

Math referansı `76ec7e6f558a6c9ddbeb7cfe4d528ae99ab6fa83980eeb2c8bda1376494a158f`, birleşik referans `7eb966aeeec47bb875a064f29ee8e61b18ab59b1678b26c179ead90046713129` kimliğindedir.

Yerel raporlar: `compare-bgegqdoc` (math debug), `compare-lkhuntdt` (math release), `compare-crkt4gor` (birleşik debug), `compare-7us5i_vu` (birleşik release) ve `verify-j0qpg9v6` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt beş özgün sentetik Vec3 step dizisi ve başlangıç/her adım snapshot'larıyla sınırlıdır. `scale`, `horizontal`, `offsetRandom`, `horizontalDistance`, uzunluk yardımcıları ve diğer Vec3 overload'ları ayrı dilimlerdir.
