# P01.b — Vec3 closerThan doğrulaması

`src/math.rs`, Java `Vec3.closerThan(Position, double)` ve `Vec3.closerThan(Vec3, double, double)` overload'larının strict eşiklerini, x/z toplama sırasını ve y koşulunu karşılar. Java oracle iki overload'u doğrudan çağırır; Rust adaptörü aynı girdideki iki Rust API sonucunu taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 765 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **765/765** math senaryosunda eşleşti.
- Beş yeni senaryoda iç eşikler, strict sınır eşitliği, ayrı y kapısı, negatif sıfır eşikleri ve NaN/Infinity girdileri karşılaştırıldı.
- Birleşik altı aile corpus'u **1219/1219** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1219 senaryoluk Java referansını yeniden doğruladı.

Math referansı `6ac05a95b5fa504d351f637b459f1beea56623b8add26c2ecf63290fcff2a29e`, birleşik referans `b2152a4c774941fe4652233a0e5b73b96996f580d6423ec90d76cbe55ddeff4a` kimliğindedir.

Yerel raporlar: `compare-ya8jvd1o` (math debug), `compare-rh0c0wxr` (math release), `compare-u_e3yyct` (birleşik debug), `compare-326ehkgq` (birleşik release) ve `verify-_zpzaxza` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt beş özgün Vec3 girdisinde iki closerThan overload'unun boolean sonuçlarıyla sınırlıdır. `distanceToSqr` ve Mth yardımcıları ayrı dilimlerdir; Position arayüzünün Vec3 dışındaki uygulamaları bu adaptörde temsil edilmez.
