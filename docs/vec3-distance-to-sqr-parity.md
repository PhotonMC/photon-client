# P01.b — Vec3 distanceToSqr doğrulaması

`src/math.rs`, Java `Vec3.distanceToSqr(Vec3)` ve `Vec3.distanceToSqr(double, double, double)` overload'larını karşılayan iki Rust metodunu içerir. Java oracle doğrudan iki overload'u çağırır; Rust adaptörü aynı f64 bit girdilerini üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 2076 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **2076/2076** math senaryosunda eşleşti.
- Beş yeni senaryoda Vec3 ve üç bileşenli overload'lar, kaynak toplama sırası, signed zero ve overflow/NaN sonuçları karşılaştırıldı.
- Birleşik altı aile corpus'u **2578/2578** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2578 senaryoluk Java referansını yeniden doğruladı.

Math referansı `e84d285a8cd335bd1ba5cd88d076e7cb38315013b991057804c6f1b4598f1c80`, birleşik referans `9c94496c39504f2bd146d331630516f020e05259c67c0bca659d1f1ca92c74a8` kimliğindedir.

Yerel raporlar: `compare-k2v65l83` (math debug), `compare-9uf1gox2` (math release), `compare-ptlm9ptr` (birleşik debug), `compare-86z18x_k` (birleşik release) ve `verify-31k8bjcu` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt beş özgün sentetik Vec3 mesafe karesi sorgusuyla sınırlıdır. `distanceTo`, `closerThan` ve diğer Vec3 yardımcıları ayrı dilimlerdir.
