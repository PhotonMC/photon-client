# P01.b — Vec2 distanceToSqr doğrulaması

`src/math.rs`, Java `Vec2.distanceToSqr(Vec2)` metodunu karşılayan `Vec2::distance_to_sqr` metodunu içerir. Java oracle doğrudan `distanceToSqr` çağrısını yapar; Rust adaptörü aynı f32 bit girdilerini üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1743 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1743/1743** math senaryosunda eşleşti.
- Beş yeni senaryoda f32 çıkarma, kare alma, kaynak sıralı toplama, signed zero, overflow ve özel değer sonuçları karşılaştırıldı.
- Birleşik altı aile corpus'u **2245/2245** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2245 senaryoluk Java referansını yeniden doğruladı.

Math referansı `6b06e5ddd2219dc868e358d02f9afcfbda6469cddd7391f150f9280440124f6a`, birleşik referans `eb48e23a1245d82fe150fe609e00283d8fa5a86dbd72ebeb887719a74df99968` kimliklerindedir.

Yerel raporlar: `compare-q3ryrs_k` (math debug), `compare-xtfhknri` (math release), `compare-ag5oxl30` (birleşik debug), `compare-fkrk2srf` (birleşik release) ve `verify-lstfjdye` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt beş özgün sentetik Vec2 çifti ve distanceToSqr sonuçlarının f32 bit değerleriyle sınırlıdır. Vec2 add, dot, rotate ve diğer yardımcıları ayrı davranış dilimleridir.
