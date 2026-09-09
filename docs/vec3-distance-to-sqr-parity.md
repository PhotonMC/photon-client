# P01.b — Vec3 distanceToSqr doğrulaması

`src/math.rs`, Java `Vec3.distanceToSqr(Vec3)` ve `Vec3.distanceToSqr(double, double, double)` overload'larını karşılayan iki Rust metodunu içerir. Java oracle doğrudan iki overload'u çağırır; Rust adaptörü aynı f64 bit girdilerini üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 795 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **795/795** math senaryosunda eşleşti.
- Beş yeni senaryoda Vec3 ve üç bileşenli overload'lar, kaynak toplama sırası, signed zero ve overflow/NaN sonuçları karşılaştırıldı.
- Birleşik altı aile corpus'u **1249/1249** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1249 senaryoluk Java referansını yeniden doğruladı.

Math referansı `34a733639464dd64f3f5dc27cf1a37ef7f28848dab5c72a251bcb219841995ea`, birleşik referans `fc482de613c60d17cab8a4d2e03e66942832cc35cad40eb9f95939d0037ac82f` kimliğindedir.

Yerel raporlar: `compare-1xrk6zz1` (math debug), `compare-qzmn2buo` (math release), `compare-h3m21imi` (birleşik debug), `compare-xnouubh6` (birleşik release) ve `verify-5mg03fti` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt beş özgün sentetik Vec3 mesafe karesi sorgusuyla sınırlıdır. `distanceTo`, `closerThan` ve diğer Vec3 yardımcıları ayrı dilimlerdir.
