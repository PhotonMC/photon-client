# P01.b — Vec3 projectedOn doğrulaması

`src/math.rs`, Java `Vec3.projectedOn(Vec3)` metodunu karşılar. Rust adaptörü sıfır uzunluk karesi kısa yolunu ve Java `dot → scale → scale` sırasını uygular; Java oracle doğrudan `Vec3.projectedOn` çağrısını kullanır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 765 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **765/765** math senaryosunda eşleşti.
- Altı yeni senaryoda normal projection, orthogonal zero, signed-zero-preserving zero onto, negatif/fractional operands, Infinity/NaN propagation ve subnormal length-squared underflow karşılaştırıldı.
- Birleşik altı aile corpus'u **1219/1219** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1219 senaryoluk Java referansını yeniden doğruladı.

Math referansı `6ac05a95b5fa504d351f637b459f1beea56623b8add26c2ecf63290fcff2a29e`, birleşik referans `b2152a4c774941fe4652233a0e5b73b96996f580d6423ec90d76cbe55ddeff4a` kimliğindedir. Math karşılaştırma raporları `compare-ya8jvd1o` (debug) ve `compare-rh0c0wxr` (release); birleşik raporlar `compare-u_e3yyct` (debug) ve `compare-326ehkgq` (release), Java doğrulaması `verify-_zpzaxza` kimliklerindedir.

## Sınırlar

Kanıt altı özgün sentetik Vec3 girdisinde `Vec3.projectedOn(Vec3)` sonucunun bileşen, uzunluk ve uzunluk karesi snapshot'larıyla sınırlıdır. `toVector3f`, local coordinate helpers ve Vec3'ün kalan yardımcıları ayrı dilimlerdir.
