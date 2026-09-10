# P01.b — Vec3 reverse doğrulaması

`src/math.rs`, Java `Vec3.reverse()` metodunun `scale(-1.0)` yönlendirmesini karşılar. Java oracle doğrudan çağrıyı yapar; Rust adaptörü aynı girdiyi `Vec3::reverse` API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1703 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1703/1703** math senaryosunda eşleşti.
- Beş yeni senaryoda sonlu bileşenler, ardışık işlem, signed zero, subnormal, büyük sonlu ve Infinity girdileri karşılaştırıldı.
- Birleşik altı aile corpus'u **2205/2205** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2205 senaryoluk Java referansını yeniden doğruladı.

Math referansı `b93cda4c5257c03011378a24fe916e8b8cd7e11d2fe75e1a1de108e4b4ebd630`, birleşik referans `413b981c2a619373545c6c4c74d3e7fec6b1a2e1dc34d5a49ee2f4eed5231162` kimliğindedir.

Yerel raporlar: `compare-a8qhmnrl` (math debug), `compare-usm27y8z` (math release), `compare-8vwtvwwl` (birleşik debug), `compare-uey51w68` (birleşik release) ve `verify-l3irw60r` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt beş özgün sentetik Vec3 step dizisi ve başlangıç/her adım snapshot'larıyla sınırlıdır. `scale`, `horizontal`, `offsetRandom`, `horizontalDistance`, uzunluk yardımcıları ve diğer Vec3 overload'ları ayrı dilimlerdir.
