# P01.b — Vec3 distanceTo doğrulaması

`src/math.rs`, Java `Vec3.distanceTo(Vec3)` metodunun hedef-minus-mevcut fark sırasını, x/y/z karelerinin kaynak sırasındaki toplamını ve son `sqrt` çağrısını karşılar. Java oracle doğrudan `distanceTo` çağırır; Rust adaptörü sonucu f64 bitleriyle JSONL'e taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 905 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **905/905** math senaryosunda eşleşti.
- Beş yeni senaryoda temel uzaklık, kaynak toplama sırası, signed zero, subnormal underflow ve overflow/özel değer sonuçları karşılaştırıldı.
- Birleşik altı aile corpus'u **1387/1387** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1387 senaryoluk Java referansını yeniden doğruladı.

Math referansı `847816d7bf7d2e0e9cecc67b5d7010685842d00cd4860fb3cf9447d37a9fe8a9`, birleşik referans `a2de3a61075984570c78972cf64abf406bbb6ee6d60136396680e14b82219664` kimliğindedir.

Yerel raporlar: `compare-psz_s0y8` (math debug), `compare-xhpjkx1_` (math release), `compare-1i6_miir` (birleşik debug), `compare-zyhj5dym` (birleşik release) ve `verify-b04aqmvb` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt beş özgün sentetik Vec3 uzaklık sorgusuyla sınırlıdır. `distanceToSqr`, `closerThan` ve diğer Vec3 yardımcıları ayrı dilimlerdir.
