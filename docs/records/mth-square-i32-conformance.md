# Mth square(int) davranış sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, Java Edition 26.2 `net.minecraft.util.Mth` içindeki `square(int)` yordamını kapsar.

Yordam kaynakta tek bir i32 çarpımıyla `x * x` sonucunu döndürür. Java int aritmetiğinin 32 bit iki'nin tümleyeni sarması korunur; Rust tarafında genişletme, clamp veya tanımsız taşma kullanılmaz.

Behavior runner `x` alanını signed decimal i32 string olarak taşır. Oracle, Java `Mth.square(int)` overload'unu doğrudan çağırır ve sonucu `result.value` altında signed decimal i32 string olarak döndürür; beklenen değerler Rust'tan türetilmez.

İncelenen kaynak konumu: `net/minecraft/util/Mth.java:646-648`.

Kapsam dışı kalanlar: `square(float)`, `square(double)`, `square(long)`, `cube`, `clampedMap`, `map` ve diğer Mth yardımcıları.


## Client-parity doğrulaması

`crates/voxel-core/src/math.rs`, Java `Mth.square(int)` yordamının tek i32 çarpımını Java'nın 32 bit sarmasıyla taşır. `examples/behavior_adapter.rs` sonucu signed decimal i32 string olarak dışarı verir.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Sıfır, birim ve negatif girdiler, 46340/46341 overflow sınırı, iki'nin kuvvetiyle tam sarmalar ve i32 minimum/maksimum sınırlarını içeren **15 özgün senaryo** hazırlandı.
- Oracle referansı iki ayrı JVM çalıştırmasıyla yeniden doğrulandı.
- Rust debug ve release adaptörleri güncel **2299/2299** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `9421a309da79905eddc1ce85361a8baef1e5c3a592c7326ac123cb13c20a6508`. Birleşik referans kimliği: `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b`.

Adapter hashleri: debug `6e9d1efd81eb178582a5369aed3a170bdc729264a5335bdded96c50bfb50b3ad`, release `130c76c303ffd5d73c67bb1f8b81e3d59bfb0c7a484c6d747eff32d1dcdfcb828`.

Yerel raporlar: `compare-00tgiqc4` ve `compare-aylr4ozm` (math debug/release), `compare-5bjkz31s` ve `compare-zfkba6ts` (birleşik debug/release), `verify-t1_8mau7` (math) ve `verify-mgwg6r2l` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt `Mth.square(int)` yordamının sabit signed i32 girdilerdeki sonucunu kapsar. Diğer `square` overload'ları, `cube`, `clampedMap`, `map`, diğer Mth yardımcıları ve bu yordamı kullanan istemci akışları ayrı dilimlerdir. `.data/` altındaki Java runtime, capture ve çıktılar yalnızca yerel test girdisi olarak kalır.
