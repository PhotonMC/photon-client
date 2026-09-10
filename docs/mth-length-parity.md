# P01.a — Mth length doğrulaması

`src/math.rs`, Java `Mth.lengthSquared` ve `Mth.length` overload'larını; iki/üç bileşenli double yolları, `length(float, float)` double ara yolunu ve üçlü f32 kare toplamını uygular. `examples/behavior_adapter.rs` sonucu Java oracle ile aynı hassasiyetin raw bit biçiminde dışarı verir.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- İki/üç bileşenli f64 `lengthSquared` ve `length`, f32 iki bileşenli `length` ve f32 üç bileşenli `lengthSquared` yollarını kapsayan toplam **16 özgün senaryo** hazırlandı.
- 3-4 temel üçgeni, signed zero, Infinity, NaN, double overflow ve f32 overflow girdileriyle birlikte kaynak overload çözümlemesi ve sonuç hassasiyeti karşılaştırıldı.
- Rust debug ve release adaptörleri güncel **1987/1987** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2489/2489** senaryoda hem debug hem release adaptörüyle eşleşti.
- Math ve birleşik `verify` çalıştırmaları Java referansını iki JVM tekrarıyla yeniden doğruladı.

Math referans kimliği: `665bdda6827e71f559f4e9a959ed126f3d3e055594f3b677359c51d0d3a74e16`. Birleşik referans kimliği: `5b47ccafa315b01ab570af21d177805383c93acb6457c83b1c5c54b1c8cd6ccc`.

Adapter hashleri: debug `bf472fbe984c9033c542831c832c85f3ea36283e022040542677049db4615a06`, release `04de447e6d530b9e66134316577c0e326f349761b64ee41c18f4e76fd6d3b226`.

Yerel raporlar: `compare-zt_h_egg` ve `compare-7afcvx21` (math debug/release), `compare-251aed0_` ve `compare-jqj2kta_` (birleşik debug/release), `verify-5v_ilol4` (math) ve `verify-csdh4b83` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt yalnızca altı `Mth.lengthSquared/length` overload'unun sabit girdili sonuç bitlerine ve mevcut corpus regresyonuna aittir. Vec2/Vec3 uzunluk metotları, `Mth.quantize`, `Mth.outFromOrigin` dışındaki sonraki `Mth` yardımcıları ve bu yardımcıları kullanan istemci çağrı zincirleri sonraki dilimlerdir. `.data/` altındaki Java runtime, capture ve çıktılar yalnızca yerel test girdisi olarak kalır.
