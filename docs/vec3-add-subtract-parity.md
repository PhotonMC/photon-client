# P01.b — Vec3 add/subtract overload doğrulaması

`src/math.rs`, Java `Vec3.add` ve `Vec3.subtract` overload'larını karşılayan altı Rust metodu içerir. Java oracle overload'ları doğrudan çağırır; Rust adaptörü aynı girdileri üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1534 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1534/1534** math senaryosunda eşleşti.
- Beş yeni senaryoda Vec3, scalar ve üç bileşenli overload zincirleri; signed zero, Infinity ve NaN girdileri karşılaştırıldı.
- Birleşik altı aile corpus'u **2036/2036** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2036 senaryoluk Java referansını yeniden doğruladı.

Math referansı `99f42388f3562f04669e01818a39202ee8f698f0151f153acf5352e96f0fcc58`, birleşik referans `5607afc8c59b5ee3393ed20e95d9fbca8ce40846df423d5e52589b902d90b90f` kimliğindedir.

Yerel raporlar: `compare-t6yvipac` (math debug), `compare-6_snmnk3` (math release), `compare-tgx0qoj4` (birleşik debug), `compare-t1bsi_04` (birleşik release) ve `verify-zwp9iban` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt beş özgün sentetik Vec3 step dizisi ve vektör snapshot'larıyla sınırlıdır. `dot`, `cross`, `distanceToSqr`, `distanceTo` ve diğer Vec3 yardımcıları ayrı dilimlerdir.
