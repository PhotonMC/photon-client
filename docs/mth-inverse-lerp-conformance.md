# Mth inverseLerp davranış sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, Java Edition 26.2 `net.minecraft.util.Mth` içindeki `inverseLerp` f32 ve f64 overload'larını kapsar.

Her overload, `value - min` payını ve `max - min` paydasını kendi kayan nokta hassasiyetinde hesaplar ve payı paydaya böler. Sonuç aralığa sıkıştırılmaz; sınır dışı faktörler, ters sınırlar, sıfır aralık ve özel değer sonuçları doğrudan Java aritmetiğinden gelir.

Behavior runner `value`, `min` ve `max` alanlarını ilgili hassasiyetin sekiz veya on altı haneli lowercase raw IEEE-754 bit string'leri olarak taşır. Oracle, Java `Mth.inverseLerp` overload'unu doğrudan çağırır ve sonucu `result.bits` altında aynı hassasiyetin raw bit string'i olarak döndürür; beklenen değerler Rust'tan türetilmez.

İncelenen kaynak konumu: `net/minecraft/util/Mth.java:338-344`.

Kapsam dışı kalanlar: `lerp`, `clampedLerp`, `map`/`clampedMap`, vektör overload'ları ve bu helper'ı kullanan istemci akışları.


## Client-parity doğrulaması

`crates/voxel-core/src/math.rs`, Java `Mth.inverseLerp(float, float, float)` ve `Mth.inverseLerp(double, double, double)` overload'larını pay/payda çıkarma ve bölme sırasıyla taşır. `examples/behavior_adapter.rs` sonucu ilgili hassasiyetin raw IEEE-754 bitleri olarak dışarı verir.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- f32 ve f64 overload'ları için temel aralık, aralık dışı faktör, negatif/ters sınır, tam sınır, signed zero, sıfır aralık, subnormal, sonlu taşma, Infinity ve NaN içeren toplam **28 özgün senaryo** hazırlandı.
- Oracle referansı iki ayrı JVM çalıştırmasıyla yeniden doğrulandı.
- Rust debug ve release adaptörleri güncel **2299/2299** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `9421a309da79905eddc1ce85361a8baef1e5c3a592c7326ac123cb13c20a6508`. Birleşik referans kimliği: `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b`.

Adapter hashleri: debug `6e9d1efd81eb178582a5369aed3a170bdc729264a5335bdded96c50bfb50b3ad`, release `130c76c303ffd5d73c67bb1f8b81e3d59bfb0c7a484c6d747eff32d1dcdfcb828`.

Yerel raporlar: `compare-00tgiqc4` ve `compare-aylr4ozm` (math debug/release), `compare-5bjkz31s` ve `compare-zfkba6ts` (birleşik debug/release), `verify-t1_8mau7` (math) ve `verify-mgwg6r2l` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt iki `Mth.inverseLerp` overload'unun sabit f32/f64 girdilerde doğrudan aritmetik sonucunu kapsar. `lerp`, `clampedLerp`, `map`/`clampedMap`, vektör overload'ları ve bu helper'ı kullanan istemci akışları sonraki dilimlerdir. `.data/` altındaki Java runtime, capture ve çıktılar yalnızca yerel test girdisi olarak kalır.
