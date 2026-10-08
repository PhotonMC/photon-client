# Mth clampedMap(float) davranış sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, Java Edition 26.2 `net.minecraft.util.Mth` içindeki `clampedMap(float, float, float, float, float)` yordamını kapsar.

Yordam kaynakta `inverseLerp(value, fromMin, fromMax)` sonucunu `clampedLerp(..., toMin, toMax)` yordamına verir. F32 çıkarma, bölme, clamp dalları ve son lerp işlemleri Java kaynak sırasını korur; giriş aralığı ters çevrilmez ve sonuç özel değerlerle normalize edilmez.

Behavior runner `value`, `from_min`, `from_max`, `to_min` ve `to_max` alanlarını lowercase raw f32 IEEE-754 bit string'leri olarak taşır. Oracle, Java `Mth.clampedMap(float)` overload'unu doğrudan çağırır ve sonucu `result.bits` altında raw f32 bit string olarak döndürür; beklenen değerler Rust'tan türetilmez.

İncelenen kaynak konumu: `net/minecraft/util/Mth.java:658-660`.

Kapsam dışı kalanlar: `clampedMap(double)`, `map`, `clampedLerp`, `inverseLerp` ve diğer Mth yardımcıları.


## Client-parity doğrulaması

`crates/photon-core/src/math.rs`, Java `Mth.clampedMap(float)` yordamının `inverseLerp` ve `clampedLerp` bileşimini aynı f32 işlem sırasıyla taşır. `examples/behavior_adapter.rs` sonucu raw f32 bit dizgisi olarak dışarı verir.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Temel eşleme, kaynak ve hedef clamp sınırları, ters aralıklar, signed zero, subnormal değerler, f32 hassasiyet sırası, sonsuzluk, NaN ve sonlu taşma içeren **20 özgün senaryo** hazırlandı.
- Oracle referansı iki ayrı JVM çalıştırmasıyla yeniden doğrulandı.
- Rust debug ve release adaptörleri güncel **2299/2299** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `9421a309da79905eddc1ce85361a8baef1e5c3a592c7326ac123cb13c20a6508`. Birleşik referans kimliği: `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b`.

Adapter hashleri: debug `6e9d1efd81eb178582a5369aed3a170bdc729264a5335bdded96c50bfb50b3ad`, release `130c76c303ffd5d73c67bb1f8b81e3d59bfb0c7a484c6d747eff32d1dcdfcb828`.

Yerel raporlar: `compare-00tgiqc4` ve `compare-aylr4ozm` (math debug/release), `compare-5bjkz31s` ve `compare-zfkba6ts` (birleşik debug/release), `verify-t1_8mau7` (math) ve `verify-mgwg6r2l` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt yalnızca `Mth.clampedMap(float)` overload'unun sabit raw f32 girdilerdeki sonucunu kapsar. `clampedMap(double)`, `map`, `clampedLerp`, `inverseLerp`, diğer Mth yardımcıları ve bu yordamı kullanan istemci akışları ayrı dilimlerdir. `.data/` altındaki Java runtime, capture ve çıktılar yalnızca yerel test girdisi olarak kalır.
