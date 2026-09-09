# P01.b — Vec3 horizontal doğrulaması

`src/math.rs`, Java `Vec3.horizontal()` metodunun x/z bileşenlerini koruyup y bileşenini pozitif sıfıra çeviren davranışını karşılar. Java oracle doğrudan çağrıyı yapar; Rust adaptörü aynı girdiyi `Vec3::horizontal` API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1031 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1031/1031** math senaryosunda eşleşti.
- Beş yeni senaryoda sonlu bileşenler, ardışık işlem, signed zero, subnormal, büyük sonlu ve Infinity girdileri karşılaştırıldı.
- Birleşik altı aile corpus'u **1533/1533** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1533 senaryoluk Java referansını yeniden doğruladı.

Math referansı `cbae17cd88ce5b1617d01034d27ad104db9b471eefc8bebf83ed4e8d1dc0aa70`, birleşik referans `338731e44c45c8acc6fbd70e7fe1439cdaa28ec5996f6a4d9517412cc50106a0` kimliğindedir.

Yerel raporlar: `compare-2ftkouxd` (math debug), `compare-g204xxpd` (math release), `compare-ag9x0lbh` (birleşik debug), `compare-sl2ed6ln` (birleşik release) ve `verify-5bpx38hs` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt beş özgün sentetik Vec3 step dizisi ve başlangıç/her adım snapshot'larıyla sınırlıdır. `reverse`, `scale`, `horizontalDistance`, uzunluk yardımcıları ve diğer Vec3 overload'ları ayrı dilimlerdir.
