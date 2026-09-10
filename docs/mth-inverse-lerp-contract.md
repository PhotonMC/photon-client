# Mth inverseLerp davranış sözleşmesi

Bu dilim, Java Edition 26.2 `net.minecraft.util.Mth` içindeki `inverseLerp` f32 ve f64 overload'larını kapsar.

Her overload, `value - min` payını ve `max - min` paydasını kendi kayan nokta hassasiyetinde hesaplar ve payı paydaya böler. Sonuç aralığa sıkıştırılmaz; sınır dışı faktörler, ters sınırlar, sıfır aralık ve özel değer sonuçları doğrudan Java aritmetiğinden gelir.

Behavior runner `value`, `min` ve `max` alanlarını ilgili hassasiyetin sekiz veya on altı haneli lowercase raw IEEE-754 bit string'leri olarak taşır. Oracle, Java `Mth.inverseLerp` overload'unu doğrudan çağırır ve sonucu `result.bits` altında aynı hassasiyetin raw bit string'i olarak döndürür; beklenen değerler Rust'tan türetilmez.

İncelenen kaynak konumu: `net/minecraft/util/Mth.java:338-344`.

Kapsam dışı kalanlar: `lerp`, `clampedLerp`, `map`/`clampedMap`, vektör overload'ları ve bu helper'ı kullanan istemci akışları.
