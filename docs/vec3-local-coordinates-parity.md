# P01.b — Vec3 yerel koordinat yardımcıları doğrulaması

`src/math.rs`, Java `Vec3.applyLocalCoordinatesToRotation` ve `addLocalCoordinates` yardımcılarını aynı f32 trigonometrik taban ve f64 bileşen toplama sırasıyla sunar. Java oracle doğrudan Java metodlarını çağırır; Rust adaptörü aynı sonuçları üretim API'sinden taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 989 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **989/989** math senaryosunda eşleşti.
- Dört `applyLocalCoordinatesToRotation` ve iki `addLocalCoordinates` senaryosunda Vec3 snapshot bileşenleri, uzunluk ve uzunluk karesi f64 bitleri karşılaştırıldı.
- Birleşik altı aile corpus'u **1491/1491** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1491 senaryoluk Java referansını yeniden doğruladı.

Math referansı `9df41e71098bfa6c6de40d3789b0153838da1ca1634d3f012c1af262d09ffd35`, birleşik referans `0f630454331682d5ce34027e78b67ab2555fdb80eaa4f4e4bb2485885abebab6` kimliklerindedir.

Yerel raporlar: `compare-jccgb79g` (math debug), `compare-r5e2j3zv` (math release), `compare-n67o4lpe` (birleşik debug), `compare-y4v3jm1g` (birleşik release) ve `verify-hq80hpap` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt dört statik dönüşüm, iki instance forwarding ve bu girdilerdeki f32/f64 özel bit sonuçlarıyla sınırlıdır. Kamera, oyuncu hareketi ve diğer yerel yön tüketicileri sonraki istemci dilimleridir.
