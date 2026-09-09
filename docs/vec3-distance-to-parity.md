# P01.b — Vec3 distanceTo doğrulaması

`src/math.rs`, Java `Vec3.distanceTo(Vec3)` metodunun hedef-minus-mevcut fark sırasını, x/y/z karelerinin kaynak sırasındaki toplamını ve son `sqrt` çağrısını karşılar. Java oracle doğrudan `distanceTo` çağırır; Rust adaptörü sonucu f64 bitleriyle JSONL'e taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 929 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **929/929** math senaryosunda eşleşti.
- Beş yeni senaryoda temel uzaklık, kaynak toplama sırası, signed zero, subnormal underflow ve overflow/özel değer sonuçları karşılaştırıldı.
- Birleşik altı aile corpus'u **1431/1431** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1431 senaryoluk Java referansını yeniden doğruladı.

Math referansı `8fdf229fdfa07bea578ab4c7a2dd8d338928e403b9c3475b341fcc20842f0def`, birleşik referans `e01fd95590d0e47ec886dfd327be291cda2db519ff726b58b32f5fa0a8e99c88` kimliğindedir.

Yerel raporlar: `compare-eiqs7rub` (math debug), `compare-t3kloypj` (math release), `compare-wfqzv7gl` (birleşik debug), `compare-_to6641m` (birleşik release) ve `verify-oni_ax6_` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt beş özgün sentetik Vec3 uzaklık sorgusuyla sınırlıdır. `distanceToSqr`, `closerThan` ve diğer Vec3 yardımcıları ayrı dilimlerdir.
