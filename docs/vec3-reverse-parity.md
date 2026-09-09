# P01.b — Vec3 reverse doğrulaması

`src/math.rs`, Java `Vec3.reverse()` metodunun `scale(-1.0)` yönlendirmesini karşılar. Java oracle doğrudan çağrıyı yapar; Rust adaptörü aynı girdiyi `Vec3::reverse` API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 705 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **705/705** math senaryosunda eşleşti.
- Beş yeni senaryoda sonlu bileşenler, ardışık işlem, signed zero, subnormal, büyük sonlu ve Infinity girdileri karşılaştırıldı.
- Birleşik altı aile corpus'u **1159/1159** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1159 senaryoluk Java referansını yeniden doğruladı.

Math referansı `ee70cefd63c52a2329bae4fec46f29bfb59aefb64f7f48829c4a516dc7685fb9`, birleşik referans `eb9f0578dc53caaa0fab43aebfbe945ab957c5f68d0e3eb2d790a5f4fc1c92f2` kimliğindedir.

Yerel raporlar: `compare-g26jdt9a` (math debug), `compare-wlw80e9f` (math release), `compare-l4qzoucn` (birleşik debug), `compare-ixiuyjvh` (birleşik release) ve `verify-xucqlaod` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt beş özgün sentetik Vec3 step dizisi ve başlangıç/her adım snapshot'larıyla sınırlıdır. `scale`, `horizontal`, `offsetRandom`, `horizontalDistance`, uzunluk yardımcıları ve diğer Vec3 overload'ları ayrı dilimlerdir.
