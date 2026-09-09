# P01.b — Vec3 projectedOn doğrulaması

`src/math.rs`, Java `Vec3.projectedOn(Vec3)` metodunu karşılar. Rust adaptörü sıfır uzunluk karesi kısa yolunu ve Java `dot → scale → scale` sırasını uygular; Java oracle doğrudan `Vec3.projectedOn` çağrısını kullanır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 795 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **795/795** math senaryosunda eşleşti.
- Altı yeni senaryoda normal projection, orthogonal zero, signed-zero-preserving zero onto, negatif/fractional operands, Infinity/NaN propagation ve subnormal length-squared underflow karşılaştırıldı.
- Birleşik altı aile corpus'u **1249/1249** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1249 senaryoluk Java referansını yeniden doğruladı.

Math referansı `34a733639464dd64f3f5dc27cf1a37ef7f28848dab5c72a251bcb219841995ea`, birleşik referans `fc482de613c60d17cab8a4d2e03e66942832cc35cad40eb9f95939d0037ac82f` kimliğindedir. Math karşılaştırma raporları `compare-1xrk6zz1` (debug) ve `compare-qzmn2buo` (release); birleşik raporlar `compare-h3m21imi` (debug) ve `compare-xnouubh6` (release), Java doğrulaması `verify-5mg03fti` kimliklerindedir.

## Sınırlar

Kanıt altı özgün sentetik Vec3 girdisinde `Vec3.projectedOn(Vec3)` sonucunun bileşen, uzunluk ve uzunluk karesi snapshot'larıyla sınırlıdır. `toVector3f`, local coordinate helpers ve Vec3'ün kalan yardımcıları ayrı dilimlerdir.
