# P01.b — Vec3 yerel koordinat yardımcıları doğrulaması

`src/math.rs`, Java `Vec3.applyLocalCoordinatesToRotation` ve `addLocalCoordinates` yardımcılarını aynı f32 trigonometrik taban ve f64 bileşen toplama sırasıyla sunar. Java oracle doğrudan Java metodlarını çağırır; Rust adaptörü aynı sonuçları üretim API'sinden taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 2124 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **2124/2124** math senaryosunda eşleşti.
- Dört `applyLocalCoordinatesToRotation` ve iki `addLocalCoordinates` senaryosunda Vec3 snapshot bileşenleri, uzunluk ve uzunluk karesi f64 bitleri karşılaştırıldı.
- Birleşik altı aile corpus'u **2626/2626** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2626 senaryoluk Java referansını yeniden doğruladı.

Math referansı `9421a309da79905eddc1ce85361a8baef1e5c3a592c7326ac123cb13c20a6508`, birleşik referans `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b` kimliklerindedir.

Yerel raporlar: `compare-00tgiqc4` (math debug), `compare-aylr4ozm` (math release), `compare-5bjkz31s` (birleşik debug), `compare-zfkba6ts` (birleşik release) ve `verify-mgwg6r2l` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt dört statik dönüşüm, iki instance forwarding ve bu girdilerdeki f32/f64 özel bit sonuçlarıyla sınırlıdır. Kamera, oyuncu hareketi ve diğer yerel yön tüketicileri sonraki istemci dilimleridir.
