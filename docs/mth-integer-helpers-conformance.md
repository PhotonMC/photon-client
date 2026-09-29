# Mth tamsayı yardımcıları davranış sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, Java Edition 26.2 `net.minecraft.util.Mth` içindeki `floorDiv(int,int)`, `positiveModulo(int,int)`, `positiveCeilDiv(int,int)`, `positiveCeilDiv(long,long)`, `roundToward(int,int)` ve `roundToward(long,long)` yollarını kapsar. Her senaryoda bölen sıfır değildir; Java'nın sıfıra bölme istisnası bu dilimin çıktısı değildir.

`floorDiv`, bölüm ve kalan işaretleri farklıysa sıfıra doğru Java bölümünü bir azaltarak matematiksel tabana yuvarlar. `positiveModulo`, aynı `floorDiv` sonucunu kullanarak `input - floorDiv(input, divisor) * divisor` sözleşmesini ve bölenin işaretini korur. `Integer.MIN_VALUE / -1` gibi Java tamsayı taşmaları iki tamamlayıcılı genişlikte sarılır.

`positiveCeilDiv`, Java kaynağındaki `-Math.floorDiv(-input, divisor)` sırasını korur. Özellikle minimum değerin unary negation işlemi sarılır. `roundToward`, bu sonucu `multiple` ile Java int/long çarpması olarak tekrar sarar; sonuç temsil edilebilir aralığa sığmasa da hata üretmez.

Senaryo girdileri tamsayıları decimal string olarak taşır. Oracle doğrudan sabitlenmiş Java 26.2 client JAR içindeki `Mth` çağrılarını yürütür ve sonucu string olarak kaydeder; Rust adaptörü aynı JSON sözleşmesini üretir. Beklenen değerler Rust'tan türetilmez.

İncelenen kaynak konumları: `net/minecraft/util/Mth.java:142-144` (`floorDiv`), `:164-166` (`positiveModulo`), `:674-687` (`roundToward` ve `positiveCeilDiv`).

Kapsam dışı kalanlar: sıfıra bölme istisnasının ayrıntılı hata nesnesi, `Mth`'nin diğer tamsayı overload'ları, float/double yardımcıları ve istemci algoritmalarının bu yardımcıları hangi çağrı zincirinde kullandığı.


## Client-parity doğrulaması

`crates/voxel-core/src/math.rs`, Java `Mth` tamsayı bölme, pozitif modulo ve yuvarlama yardımcılarını Java'nın işaret ve taşma kurallarıyla eşler. `examples/behavior_adapter.rs` bu API'leri JSONL senaryolarına bağlar; beklenen değerler oracle çıktısından gelir.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- `floorDiv`, `positiveModulo`, `positiveCeilDiv` ve `roundToward` için 15 özgün senaryo hazırlandı.
- Negatif kalan ve bölen işaretleri, `Integer.MIN_VALUE`/`Long.MIN_VALUE` unary negation ve bölüm taşmaları ile sonuç çarpması taşmaları karşılaştırıldı.
- Rust debug ve release adaptörleri güncel **2299/2299** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.
- Math ve birleşik `verify` çalıştırmaları Java referansını iki JVM tekrarıyla yeniden doğruladı.

Math referans kimliği: `9421a309da79905eddc1ce85361a8baef1e5c3a592c7326ac123cb13c20a6508`. Birleşik referans kimliği: `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b`.

Adapter hashleri: debug `6e9d1efd81eb178582a5369aed3a170bdc729264a5335bdded96c50bfb50b3ad`, release `130c76c303ffd5d73c67bb1f8b81e3d59bfb0c7a484c6d747eff32d1dcdfcb828`.

Yerel raporlar: `compare-00tgiqc4` ve `compare-aylr4ozm` (math debug/release), `compare-5bjkz31s` ve `compare-zfkba6ts` (birleşik debug/release), `verify-t1_8mau7` (math) ve `verify-mgwg6r2l` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt seçili sıfır olmayan bölenlerle `Mth.floorDiv(int,int)`, `positiveModulo(int,int)`, `positiveCeilDiv(int,int)`, `positiveCeilDiv(long,long)`, `roundToward(int,int)` ve `roundToward(long,long)` sonuçlarıyla sınırlıdır. Sıfıra bölme istisnasının hata biçimi, diğer Mth tamsayı overload'ları ve bu yardımcıları kullanan istemci çağrı zincirleri sonraki dilimlerdir. `.data/` altındaki Java runtime, capture ve çıktılar yalnızca yerel test girdisi olarak kalır.
