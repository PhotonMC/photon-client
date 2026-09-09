# P01.b — Vec3 yerel koordinat yardımcıları doğrulaması

`src/math.rs`, Java `Vec3.applyLocalCoordinatesToRotation` ve `addLocalCoordinates` yardımcılarını aynı f32 trigonometrik taban ve f64 bileşen toplama sırasıyla sunar. Java oracle doğrudan Java metodlarını çağırır; Rust adaptörü aynı sonuçları üretim API'sinden taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 917 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **917/917** math senaryosunda eşleşti.
- Dört `applyLocalCoordinatesToRotation` ve iki `addLocalCoordinates` senaryosunda Vec3 snapshot bileşenleri, uzunluk ve uzunluk karesi f64 bitleri karşılaştırıldı.
- Birleşik altı aile corpus'u **1419/1419** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1419 senaryoluk Java referansını yeniden doğruladı.

Math referansı `7c835b893c95d986797a072a7b69eba5f1f31dad85d36d5bd51ba6394a221696`, birleşik referans `cfb0f4a3444412c75186c0d8b209a59632b9d62debcc435aa548bf4c364d8656` kimliklerindedir.

Yerel raporlar: `compare-dvzelshx` (math debug), `compare-sy72qqem` (math release), `compare-ng1mczig` (birleşik debug), `compare-7brjjbpz` (birleşik release) ve `verify-3oltsdwp` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt dört statik dönüşüm, iki instance forwarding ve bu girdilerdeki f32/f64 özel bit sonuçlarıyla sınırlıdır. Kamera, oyuncu hareketi ve diğer yerel yön tüketicileri sonraki istemci dilimleridir.
