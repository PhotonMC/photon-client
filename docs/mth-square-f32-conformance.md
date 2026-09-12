# Mth square(float) davranış sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, Java Edition 26.2 `net.minecraft.util.Mth` içindeki `square(float)` yordamını kapsar.

Yordam tek bir f32 çarpımıyla `x * x` sonucunu döndürür. Çarpım kaynakta görünen sırayla yapılır; Rust tarafında ortak bir `powi`, f64 genişletmesi veya özel değer normalizasyonu kullanılmaz. Signed zero karesi pozitif sıfırdır; sonlu taşma Infinity, alt taşma ve subnormal yuvarlama Java f32 çarpma semantiğine bırakılır.

Behavior runner `x` alanını lowercase raw f32 IEEE-754 bit dizgisi olarak taşır. Oracle, Java `Mth.square(float)` overload'unu doğrudan çağırır ve sonucu `result.bits` altında raw f32 bit dizgisi olarak döndürür; beklenen değerler Rust'tan türetilmez.

İncelenen kaynak konumu: `net/minecraft/util/Mth.java:634-636`.

Kapsam dışı kalanlar: `square(double)`, `square(int)`, `square(long)`, `cube`, `triangleWave`, `clampedMap`, `map` ve diğer Mth yardımcıları.


## Client-parity doğrulaması

`src/math.rs`, Java `Mth.square(float)` yordamının tek f32 çarpımını taşır. `examples/behavior_adapter.rs` sonucu raw f32 bit dizgesi olarak dışarı verir.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Sıfır, signed zero, negatif ve kesirli değerler, subnormal girdi/sonuç, büyük ve en büyük sonlu değer, Infinity, NaN ve yuvarlama sınırı içeren **15 özgün senaryo** hazırlandı.
- Oracle referansı iki ayrı JVM çalıştırmasıyla yeniden doğrulandı.
- Rust debug ve release adaptörleri güncel **2299/2299** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `9421a309da79905eddc1ce85361a8baef1e5c3a592c7326ac123cb13c20a6508`. Birleşik referans kimliği: `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b`.

Adapter hashleri: debug `6e9d1efd81eb178582a5369aed3a170bdc729264a5335bdded96c50bfb50b3ad`, release `130c76c303ffd5d73c67bb1f8b81e3d59bfb0c7a484c6d747eff32d1dcdfcb828`.

Yerel raporlar: `compare-00tgiqc4` ve `compare-aylr4ozm` (math debug/release), `compare-5bjkz31s` ve `compare-zfkba6ts` (birleşik debug/release), `verify-t1_8mau7` (math) ve `verify-mgwg6r2l` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt `Mth.square(float)` yordamının sabit raw f32 girdilerdeki sonucunu kapsar. Diğer `square` overload'ları, `cube`, `triangleWave`, `clampedMap`, `map`, diğer Mth yardımcıları ve bu yordamı kullanan istemci akışları ayrı dilimlerdir. `.data/` altındaki Java runtime, capture ve çıktılar yalnızca yerel test girdisi olarak kalır.
