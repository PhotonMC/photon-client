# Mth frac yardımcıları davranış sözleşmesi

Bu dilim, Java Edition 26.2 `net.minecraft.util.Mth` içindeki `frac(float)` ve `frac(double)` yordamlarını kapsar. Her overload için 12 özgün sentetik senaryo hazırlanır.

`frac(float)`, girdiden `Mth.floor(float)` sonucunu çıkarır. Floor sonucu Java `int` olarak üretilir ve çıkarma float hassasiyetinde yapılır; bu yüzden negatif kesirler `[0, 1)` aralığına taşınır, signed zero ve int daraltma sınırları sonuç bitlerinde görünür. `frac(double)`, aynı işlemi `Mth.lfloor(double)` sonucu olan Java `long` ile ve double hassasiyetinde yürütür.

Senaryolar pozitif/negatif sıfır, pozitif/negatif kesir, tam sayılar, f32/f64 subnormal değerler, int/long üst sınır daraltması, Infinity ve NaN yollarını raw floating-point bitleriyle taşır. Java oracle `Mth.frac` yordamlarını doğrudan çağırır; beklenen bitler Rust uygulamasından türetilmez.

İncelenen kaynak konumu: `net/minecraft/util/Mth.java:62-72,318-324`.

Kapsam dışı kalanlar: `floor`, `lfloor`, `ceil`, `ceilLong` ve `getSeed`; bunların mevcut ayrı dilim sözleşmeleri korunur.
