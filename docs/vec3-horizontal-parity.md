# P01.b — Vec3 horizontal doğrulaması

`src/math.rs`, Java `Vec3.horizontal()` metodunun x/z bileşenlerini koruyup y bileşenini pozitif sıfıra çeviren davranışını karşılar. Java oracle doğrudan çağrıyı yapar; Rust adaptörü aynı girdiyi `Vec3::horizontal` API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 771 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **771/771** math senaryosunda eşleşti.
- Beş yeni senaryoda sonlu bileşenler, ardışık işlem, signed zero, subnormal, büyük sonlu ve Infinity girdileri karşılaştırıldı.
- Birleşik altı aile corpus'u **1225/1225** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1225 senaryoluk Java referansını yeniden doğruladı.

Math referansı `ffefa3418c35a5dbf05a16b8c6bb664f38665423ada7b039029bea24f0e3f8ac`, birleşik referans `c8d3f97682c7983d7226696163a3943fe4ef6723170037ad5205d10ed0494654` kimliğindedir.

Yerel raporlar: `compare-nyuxtnwe` (math debug), `compare-c83i9uhh` (math release), `compare-2v85255z` (birleşik debug), `compare-4_se5ekn` (birleşik release) ve `verify-q8mzlbkq` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt beş özgün sentetik Vec3 step dizisi ve başlangıç/her adım snapshot'larıyla sınırlıdır. `reverse`, `scale`, `horizontalDistance`, uzunluk yardımcıları ve diğer Vec3 overload'ları ayrı dilimlerdir.
