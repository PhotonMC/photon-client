# P01.b — Vec3 add/subtract overload doğrulaması

`src/math.rs`, Java `Vec3.add` ve `Vec3.subtract` overload'larını karşılayan altı Rust metodu içerir. Java oracle overload'ları doğrudan çağırır; Rust adaptörü aynı girdileri üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 655 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **655/655** math senaryosunda eşleşti.
- Beş yeni senaryoda Vec3, scalar ve üç bileşenli overload zincirleri; signed zero, Infinity ve NaN girdileri karşılaştırıldı.
- Birleşik altı aile corpus'u **1109/1109** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1109 senaryoluk Java referansını yeniden doğruladı.

Math referansı `d6cf00dcef96ef95a3851f155f114c4a188711f01d73e154880b74a70e7fc845`, birleşik referans `69fde0fc4aa758f89939bd5a995df20db6ad31f7cfbc226ea4cd13ecaa84013d` kimliğindedir.

Yerel raporlar: `compare-s2mg_zzb` (math debug), `compare-vw7pifam` (math release), `compare-8ia7c6j_` (birleşik debug), `compare-fa02lzlz` (birleşik release) ve `verify-vt3ayq2q` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt beş özgün sentetik Vec3 step dizisi ve vektör snapshot'larıyla sınırlıdır. `vectorTo`, dot/cross, distance ve diğer Vec3 yardımcıları ayrı dilimlerdir.
