# Mth quantize davranış sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, Java Edition 26.2 `net.minecraft.util.Mth` içindeki `quantize(double, int)` yolunu kapsar.

`Mth.quantize`, `value` değerini `quantizeResolution` i32 değerine f64 bölme ile böler. Bölüm sonucuna `Mth.floor(double)` uygulanır; bu çağrı `Math.floor` sonrasında Java'nın double'dan i32'ye daraltma kurallarını kullanır. Sonuç, çözünürlükle Java i32 çarpması olarak çarpılır ve taşma i32 sarmasıyla korunur. Çözünürlük sıfır olduğunda bölme kayan nokta bölmesidir; integer divide-by-zero istisnası oluşmaz.

Her senaryo `value` için raw f64 bit string'i, `resolution` için signed i32 decimal string taşır. Oracle doğrudan sabitlenmiş Java 26.2 client JAR içindeki `Mth.quantize` çağrısını yürütür ve sonucu signed i32 decimal string olarak kaydeder; Rust adaptörü aynı JSON sözleşmesini üretir. Negatif çözünürlük, sıfır çözünürlük, signed zero, subnormal, Infinity, NaN, i32 sınır daraltması ve son çarpma taşması sözleşmeye dahildir. Beklenen değerler Rust'tan türetilmez.

İncelenen kaynak konumu: `net/minecraft/util/Mth.java:62-68,726-728`.

Kapsam dışı kalanlar: `roundToward`, `positiveCeilDiv` ve quantize sonucunu kullanan istemci çağrı zincirleri.


## Client-parity doğrulaması

`crates/voxel-core/src/math.rs`, Java `Mth.quantize(double, int)` yolunu f64 bölme, Java `floor` i32 daraltması ve wrapping i32 çarpımıyla uygular. `examples/behavior_adapter.rs` sonucu Java oracle ile aynı signed i32 decimal biçiminde dışarı verir.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Pozitif/negatif kesir, negatif ve minimum çözünürlük, sıfır ve signed zero, subnormal, Infinity, NaN, i32 sınır daraltması, sıfır çözünürlük ve çarpma taşmasını kapsayan toplam **19 özgün senaryo** hazırlandı.
- Oracle referansı iki ayrı JVM çalıştırmasıyla yeniden doğrulandı.
- Rust debug ve release adaptörleri güncel **2299/2299** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.

Math referans kimliği: `9421a309da79905eddc1ce85361a8baef1e5c3a592c7326ac123cb13c20a6508`. Birleşik referans kimliği: `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b`.

Adapter hashleri: debug `6e9d1efd81eb178582a5369aed3a170bdc729264a5335bdded96c50bfb50b3ad`, release `130c76c303ffd5d73c67bb1f8b81e3d59bfb0c7a484c6d747eff32d1dcdfcb828`.

Yerel raporlar: `compare-00tgiqc4` ve `compare-aylr4ozm` (math debug/release), `compare-5bjkz31s` ve `compare-zfkba6ts` (birleşik debug/release), `verify-t1_8mau7` (math) ve `verify-mgwg6r2l` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt yalnızca `Mth.quantize(double, int)` için sabit girdili signed i32 sonuçlarına ve mevcut corpus regresyonuna aittir. Diğer yuvarlama/ceil yardımcıları ve quantize sonucunu kullanan istemci çağrı zincirleri sonraki dilimlerdir. `.data/` altındaki Java runtime, capture ve çıktılar yalnızca yerel test girdisi olarak kalır.
