# P01.b — Vec3 projectedOn doğrulaması

`src/math.rs`, Java `Vec3.projectedOn(Vec3)` metodunu karşılar. Rust adaptörü sıfır uzunluk karesi kısa yolunu ve Java `dot → scale → scale` sırasını uygular; Java oracle doğrudan `Vec3.projectedOn` çağrısını kullanır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 2076 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **2076/2076** math senaryosunda eşleşti.
- Altı yeni senaryoda normal projection, orthogonal zero, signed-zero-preserving zero onto, negatif/fractional operands, Infinity/NaN propagation ve subnormal length-squared underflow karşılaştırıldı.
- Birleşik altı aile corpus'u **2578/2578** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2578 senaryoluk Java referansını yeniden doğruladı.

Math referansı `e84d285a8cd335bd1ba5cd88d076e7cb38315013b991057804c6f1b4598f1c80`, birleşik referans `9c94496c39504f2bd146d331630516f020e05259c67c0bca659d1f1ca92c74a8` kimliğindedir. Math karşılaştırma raporları `compare-k2v65l83` (debug) ve `compare-9uf1gox2` (release); birleşik raporlar `compare-ptlm9ptr` (debug) ve `compare-86z18x_k` (release), Java doğrulaması `verify-31k8bjcu` kimliklerindedir.

## Sınırlar

Kanıt altı özgün sentetik Vec3 girdisinde `Vec3.projectedOn(Vec3)` sonucunun bileşen, uzunluk ve uzunluk karesi snapshot'larıyla sınırlıdır. `toVector3f`, local coordinate helpers ve Vec3'ün kalan yardımcıları ayrı dilimlerdir.
