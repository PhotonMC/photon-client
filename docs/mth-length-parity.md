# P01.a — Mth length doğrulaması

`src/math.rs`, Java `Mth.lengthSquared` ve `Mth.length` overload'larını; iki/üç bileşenli double yolları, `length(float, float)` double ara yolunu ve üçlü f32 kare toplamını uygular. `examples/behavior_adapter.rs` sonucu Java oracle ile aynı hassasiyetin raw bit biçiminde dışarı verir.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- İki/üç bileşenli f64 `lengthSquared` ve `length`, f32 iki bileşenli `length` ve f32 üç bileşenli `lengthSquared` yollarını kapsayan toplam **16 özgün senaryo** hazırlandı.
- 3-4 temel üçgeni, signed zero, Infinity, NaN, double overflow ve f32 overflow girdileriyle birlikte kaynak overload çözümlemesi ve sonuç hassasiyeti karşılaştırıldı.
- Rust debug ve release adaptörleri güncel **1450/1450** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **1952/1952** senaryoda hem debug hem release adaptörüyle eşleşti.
- Math ve birleşik `verify` çalıştırmaları Java referansını iki JVM tekrarıyla yeniden doğruladı.

Math referans kimliği: `fb79a016fcf5041a8631f665991eaad422d1346ef0c1c0c12f2166cdb0057846`. Birleşik referans kimliği: `4a0878eb618b5394066f49b60b766126e243dc668d2c5a98b675f2d4a0cf4f11`.

Adapter hashleri: debug `1a871a434c144a587a2723327bd46aaf9a6d1b4612396a66fffe175369c31d12`, release `8c7871587c66a581ea4b33d57a1ef23281843488a7661885a1a704dee0ad3e74`.

Yerel raporlar: `compare-qtqxwt9b` ve `compare-ebava42g` (math debug/release), `compare-x0hkkvkv` ve `compare-x5wgwswa` (birleşik debug/release), `verify-m2eek8to` (math) ve `verify-rc21vyy9` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt yalnızca altı `Mth.lengthSquared/length` overload'unun sabit girdili sonuç bitlerine ve mevcut corpus regresyonuna aittir. Vec2/Vec3 uzunluk metotları, `Mth.quantize`, `Mth.outFromOrigin` dışındaki sonraki `Mth` yardımcıları ve bu yardımcıları kullanan istemci çağrı zincirleri sonraki dilimlerdir. `.data/` altındaki Java runtime, capture ve çıktılar yalnızca yerel test girdisi olarak kalır.
