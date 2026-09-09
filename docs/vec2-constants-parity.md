# P01.b — Vec2 sabitleri doğrulaması

`src/math.rs`, Java `Vec2.ZERO`, `ONE`, `UNIT_X`, `NEG_UNIT_X`, `UNIT_Y`, `NEG_UNIT_Y`, `MAX` ve `MIN` sabitlerini associated constant olarak sunar. Java oracle sabitleri doğrudan okur; Rust adaptörü aynı değerleri üretim API’sinden taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 929 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **929/929** math senaryosunda eşleşti.
- Sekiz yeni senaryoda bütün public Vec2 sabitlerinin f32 bileşenleri, `MAX` sınırı ve `MIN` pozitif subnormal değeri karşılaştırıldı.
- Birleşik altı aile corpus’u **1431/1431** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1431 senaryoluk Java referansını yeniden doğruladı.

Math referansı `8fdf229fdfa07bea578ab4c7a2dd8d338928e403b9c3475b341fcc20842f0def`, birleşik referans `e01fd95590d0e47ec886dfd327be291cda2db519ff726b58b32f5fa0a8e99c88` kimliklerindedir.

Yerel raporlar: `compare-eiqs7rub` (math debug), `compare-t3kloypj` (math release), `compare-wfqzv7gl` (birleşik debug), `compare-_to6641m` (birleşik release) ve `verify-oni_ax6_` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt sekiz özgün sabit sorgusu ve Vec2 snapshot sonuçlarıyla sınırlıdır. Sabitlerin oyun davranışındaki tüm tüketicileri sonraki istemci dilimleridir.
