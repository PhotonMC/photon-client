# P01.b — Vec2 dot doğrulaması

`src/math.rs`, Java `Vec2.dot(Vec2)` metodunu karşılayan `Vec2::dot` metodunu içerir. Java oracle doğrudan `dot` çağrısını yapar; Rust adaptörü aynı f32 bit girdilerini üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 854 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **854/854** math senaryosunda eşleşti.
- Beş yeni senaryoda temel çarpım, f32 yuvarlama, signed zero, overflow ve özel değer sonuçları karşılaştırıldı.
- Birleşik altı aile corpus'u **1308/1308** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1308 senaryoluk Java referansını yeniden doğruladı.

Math referansı `0ba4ea9be106c1ef86d8a7180620724b6127bb00eb89447c2677101d0067f400`, birleşik referans `94ba7a39728d11e2636807a8347713b80c9849396db8a78a6093edab3bc18c80` kimliklerindedir.

Yerel raporlar: `compare-o4ywsb7q` (math debug), `compare-7enf_yu2` (math release), `compare-dheuiwbd` (birleşik debug), `compare-vkmyk8cn` (birleşik release) ve `verify-32kafz9f` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt beş özgün sentetik Vec2 çifti ve dot sonuçlarının f32 bit snapshot'larıyla sınırlıdır. Vec2 add, scale, normalize, distance ve diğer yardımcıları ayrı davranış dilimleridir.
