# P01.b — Vec2 distanceToSqr doğrulaması

`src/math.rs`, Java `Vec2.distanceToSqr(Vec2)` metodunu karşılayan `Vec2::distance_to_sqr` metodunu içerir. Java oracle doğrudan `distanceToSqr` çağrısını yapar; Rust adaptörü aynı f32 bit girdilerini üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 1653 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **1653/1653** math senaryosunda eşleşti.
- Beş yeni senaryoda f32 çıkarma, kare alma, kaynak sıralı toplama, signed zero, overflow ve özel değer sonuçları karşılaştırıldı.
- Birleşik altı aile corpus'u **2155/2155** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2155 senaryoluk Java referansını yeniden doğruladı.

Math referansı `e9b1c1e55b2a03528b712cd304101c7b36378556372ebad60bb330a4a60eb192`, birleşik referans `2a68356f797e134e8a268355635fffb2db6a83b94e11d3afbdf308978093028e` kimliklerindedir.

Yerel raporlar: `compare-ceusm4y4` (math debug), `compare-hconpy1a` (math release), `compare-rhjx0z7e` (birleşik debug), `compare-jtl411eu` (birleşik release) ve `verify-u5d5zd4c` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt beş özgün sentetik Vec2 çifti ve distanceToSqr sonuçlarının f32 bit değerleriyle sınırlıdır. Vec2 add, dot, rotate ve diğer yardımcıları ayrı davranış dilimleridir.
