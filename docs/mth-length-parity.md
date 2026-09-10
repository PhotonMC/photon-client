# P01.a — Mth length doğrulaması

`src/math.rs`, Java `Mth.lengthSquared` ve `Mth.length` overload'larını; iki/üç bileşenli double yolları, `length(float, float)` double ara yolunu ve üçlü f32 kare toplamını uygular. `examples/behavior_adapter.rs` sonucu Java oracle ile aynı hassasiyetin raw bit biçiminde dışarı verir.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- İki/üç bileşenli f64 `lengthSquared` ve `length`, f32 iki bileşenli `length` ve f32 üç bileşenli `lengthSquared` yollarını kapsayan toplam **16 özgün senaryo** hazırlandı.
- 3-4 temel üçgeni, signed zero, Infinity, NaN, double overflow ve f32 overflow girdileriyle birlikte kaynak overload çözümlemesi ve sonuç hassasiyeti karşılaştırıldı.
- Rust debug ve release adaptörleri güncel **2076/2076** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2578/2578** senaryoda hem debug hem release adaptörüyle eşleşti.
- Math ve birleşik `verify` çalıştırmaları Java referansını iki JVM tekrarıyla yeniden doğruladı.

Math referans kimliği: `e84d285a8cd335bd1ba5cd88d076e7cb38315013b991057804c6f1b4598f1c80`. Birleşik referans kimliği: `9c94496c39504f2bd146d331630516f020e05259c67c0bca659d1f1ca92c74a8`.

Adapter hashleri: debug `6eb1e772a2e06c20b8d0c09ba0b2a11b5f1df7a671ee8d1513127bc318d143ef`, release `318c5ef22a8d56277aa19f9da9f5becca9f575904eba79756b959bdc8c993fe88`.

Yerel raporlar: `compare-k2v65l83` ve `compare-9uf1gox2` (math debug/release), `compare-ptlm9ptr` ve `compare-86z18x_k` (birleşik debug/release), `verify-9dtf6rea` (math) ve `verify-31k8bjcu` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt yalnızca altı `Mth.lengthSquared/length` overload'unun sabit girdili sonuç bitlerine ve mevcut corpus regresyonuna aittir. Vec2/Vec3 uzunluk metotları, `Mth.quantize`, `Mth.outFromOrigin` dışındaki sonraki `Mth` yardımcıları ve bu yardımcıları kullanan istemci çağrı zincirleri sonraki dilimlerdir. `.data/` altındaki Java runtime, capture ve çıktılar yalnızca yerel test girdisi olarak kalır.
