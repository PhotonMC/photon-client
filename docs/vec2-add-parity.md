# P01.b — Vec2 add overload'ları doğrulaması

`src/math.rs`, Java `Vec2.add(Vec2)` ve `Vec2.add(float)` metodlarını karşılayan `Vec2::add_vec2` ve `Vec2::add_scalar` metodlarını içerir. Java oracle doğrudan `add` çağrılarını yapar; Rust adaptörü aynı f32 bit girdilerini üretim API'sine taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 875 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **875/875** math senaryosunda eşleşti.
- Beş yeni senaryoda Vec2 ve scalar overload'ları, signed zero, overflow ve özel değer sonuçları karşılaştırıldı.
- Birleşik altı aile corpus'u **1329/1329** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 1329 senaryoluk Java referansını yeniden doğruladı.

Math referansı `5613b7aa336a4fb4cf729c33b1e4d9566e5c0795c663ab83879f3787157f0831`, birleşik referans `22a5e87b3b46a0afade4d43a8a9e0dfaf96d8d892ea18ddc0ba5ebf77750772c` kimliklerindedir.

Yerel raporlar: `compare-p4m_t_r8` (math debug), `compare-eskrxypz` (math release), `compare-skwbedqw` (birleşik debug), `compare-9bwuf8cs` (birleşik release) ve `verify-5pjzzzfd` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt beş özgün sentetik Vec2 girdisi ve add sonuçlarının f32 bit snapshot'larıyla sınırlıdır. Vec2 dot, rotate, distance ve diğer yardımcıları ayrı davranış dilimleridir.
