# Mth quantize davranış sözleşmesi

Bu dilim, Java Edition 26.2 `net.minecraft.util.Mth` içindeki `quantize(double, int)` yolunu kapsar.

`Mth.quantize`, `value` değerini `quantizeResolution` i32 değerine f64 bölme ile böler. Bölüm sonucuna `Mth.floor(double)` uygulanır; bu çağrı `Math.floor` sonrasında Java'nın double'dan i32'ye daraltma kurallarını kullanır. Sonuç, çözünürlükle Java i32 çarpması olarak çarpılır ve taşma i32 sarmasıyla korunur. Çözünürlük sıfır olduğunda bölme kayan nokta bölmesidir; integer divide-by-zero istisnası oluşmaz.

Her senaryo `value` için raw f64 bit string'i, `resolution` için signed i32 decimal string taşır. Oracle doğrudan sabitlenmiş Java 26.2 client JAR içindeki `Mth.quantize` çağrısını yürütür ve sonucu signed i32 decimal string olarak kaydeder; Rust adaptörü aynı JSON sözleşmesini üretir. Negatif çözünürlük, sıfır çözünürlük, signed zero, subnormal, Infinity, NaN, i32 sınır daraltması ve son çarpma taşması sözleşmeye dahildir. Beklenen değerler Rust'tan türetilmez.

İncelenen kaynak konumu: `net/minecraft/util/Mth.java:62-68,726-728`.

Kapsam dışı kalanlar: `roundToward`, `positiveCeilDiv` ve quantize sonucunu kullanan istemci çağrı zincirleri.
