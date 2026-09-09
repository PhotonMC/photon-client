# P01.b — Vec3 horizontal doğrulaması

`src/math.rs`, Java `Vec3.horizontal()` metodunun x/z bileşenlerini koruyup y bileşenini pozitif sıfıra çeviren davranışını karşılar. Java oracle doğrudan çağrıyı yapar; Rust adaptörü aynı girdiyi `Vec3::horizontal` API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 989 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **989/989** math senaryosunda eşleşti.
- Beş yeni senaryoda sonlu bileşenler, ardışık işlem, signed zero, subnormal, büyük sonlu ve Infinity girdileri karşılaştırıldı.
- Birleşik altı aile corpus'u **1491/1491** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1491 senaryoluk Java referansını yeniden doğruladı.

Math referansı `9df41e71098bfa6c6de40d3789b0153838da1ca1634d3f012c1af262d09ffd35`, birleşik referans `0f630454331682d5ce34027e78b67ab2555fdb80eaa4f4e4bb2485885abebab6` kimliğindedir.

Yerel raporlar: `compare-jccgb79g` (math debug), `compare-r5e2j3zv` (math release), `compare-n67o4lpe` (birleşik debug), `compare-y4v3jm1g` (birleşik release) ve `verify-hq80hpap` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt beş özgün sentetik Vec3 step dizisi ve başlangıç/her adım snapshot'larıyla sınırlıdır. `reverse`, `scale`, `horizontalDistance`, uzunluk yardımcıları ve diğer Vec3 overload'ları ayrı dilimlerdir.
