# Mth rotLerpRad davranış sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, Java Edition 26.2 `net.minecraft.util.Mth` içindeki `rotLerpRad(float, float, float)` yordamını kapsar.

Yordam önce `to - from` farkını f32 olarak hesaplar. Fark `-π` değerinden küçük kaldığı sürece f32 `2π` ekler; fark `π` değerine eşit veya büyük olduğu sürece aynı f32 `2π` değerini çıkarır. Sonuç, clamp edilmemiş `alpha` ile `from + alpha * fark` kaynak sırasıyla üretilir. `+π` sınırı `-π` yönüne taşınır; `-π` sınırı olduğu gibi kalır. Signed zero, taşma ve NaN ara sonuçları bu işlemlerin Java davranışıyla birlikte korunur.

Behavior runner `alpha`, `from` ve `to` alanlarını lowercase raw f32 IEEE-754 bit dizgeleri olarak taşır. Oracle, Java `Mth.rotLerpRad` yordamını doğrudan çağırır ve sonucu `result.bits` altında raw f32 bit dizgesi olarak döndürür; beklenen değerler Rust'tan türetilmez. Özgün senaryolar sonlu farkları ve NaN yollarını kapsar. Farkın sonsuz olması Java'nın sonlanmayan `while` döngülerine girdiğinden, Infinity farkı üreten girdiler oracle senaryolarına dahil edilmemiştir.

İncelenen kaynak konumu: `net/minecraft/util/Mth.java:616-630`.

Kapsam dışı kalanlar: `rotLerp`, `triangleWave`, `square`, diğer açı yardımcıları ve `rotLerpRad` kullanan istemci akışları.


## Client-parity doğrulaması

`src/math.rs`, Java `Mth.rotLerpRad(float, float, float)` yordamının f32 fark çıkarma, π sınırlarında tekrarlı açı sarmalama ve `from + alpha * fark` işlem sırasını taşır. `examples/behavior_adapter.rs` sonucu raw f32 bit dizgesi olarak dışarı verir.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Temel interpolation, extrapolation, ±π sınırı, açı geçişi, tam tur, signed zero, subnormal, geniş sonlu fark ve NaN içeren **15 özgün senaryo** hazırlandı.
- Oracle referansı iki ayrı JVM çalıştırmasıyla yeniden doğrulandı.
- Rust debug ve release adaptörleri güncel **2299/2299** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `9421a309da79905eddc1ce85361a8baef1e5c3a592c7326ac123cb13c20a6508`. Birleşik referans kimliği: `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b`.

Adapter hashleri: debug `6e9d1efd81eb178582a5369aed3a170bdc729264a5335bdded96c50bfb50b3ad`, release `130c76c303ffd5d73c67bb1f8b81e3d59bfb0c7a484c6d747eff32d1dcdfcb828`.

Yerel raporlar: `compare-00tgiqc4` ve `compare-aylr4ozm` (math debug/release), `compare-5bjkz31s` ve `compare-zfkba6ts` (birleşik debug/release), `verify-t1_8mau7` (math) ve `verify-mgwg6r2l` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt `Mth.rotLerpRad` yordamının sabit raw f32 girdilerdeki sonucunu kapsar. `rotLerp`, `triangleWave`, `square`, diğer açı yardımcıları ve `rotLerpRad` kullanan istemci akışları ayrı dilimlerdir. Java döngülerinin sonlanmadığı Infinity farkları senaryo kapsamı dışındadır. `.data/` altındaki Java runtime, capture ve çıktılar yalnızca yerel test girdisi olarak kalır.
