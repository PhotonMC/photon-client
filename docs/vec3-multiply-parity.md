# P01.b — Vec3 multiply overload doğrulaması

`src/math.rs`, Java `Vec3.multiply(Vec3)` ve `Vec3.multiply(double, double, double)` overload'larının bileşen yönlendirmesini ve x/y/z çarpımlarını karşılar. Java oracle overload'ları doğrudan çağırır; Rust adaptörü aynı girdileri üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1221 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1221/1221** math senaryosunda eşleşti.
- Beş yeni senaryoda Vec3 ve üç bileşenli overload'lar, overload zinciri, signed zero, taşma ve NaN/Infinity girdileri karşılaştırıldı.
- Birleşik altı aile corpus'u **1723/1723** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1723 senaryoluk Java referansını yeniden doğruladı.

Math referansı `df8b0fd452c7f12175fe9ced3c534c1f64bdb7df75cc28529becd0eb8ab97b3c`, birleşik referans `86380f621c6f188dddfa1e68a94057a8ada6d638b9f5c04d3a2c9682b1ca75cf` kimliğindedir.

Yerel raporlar: `compare-6dvmyv_h` (math debug), `compare-ak0n9_u3` (math release), `compare-_8esk0jy` (birleşik debug), `compare-g98k9sn0` (birleşik release) ve `verify-72fv37y0` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt beş özgün sentetik Vec3 step dizisi ve vektör snapshot'larıyla sınırlıdır. `scale`, `reverse`, `horizontal`, uzunluk yardımcıları ve diğer Vec3 overload'ları ayrı dilimlerdir.
