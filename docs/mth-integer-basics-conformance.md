# Mth tamsayı temel yardımcıları davranış sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, Java Edition 26.2 `net.minecraft.util.Mth` içindeki `abs(int)`, `clamp(int,int,int)`, `clamp(long,long,long)`, `isMultipleOf(int,int)` ve `wrapDegrees(int)` yollarını kapsar. `isMultipleOf` senaryolarında bölen sıfır değildir; Java'nın sıfıra bölme istisnası bu dilimin çıktısı değildir.

`abs(int)`, Java `Math.abs` davranışını izler; `Integer.MIN_VALUE` için temsil edilemeyen pozitif değer iki tamamlayıcılı int genişliğinde yine `Integer.MIN_VALUE` olur. `clamp` önce değeri alt sınıra göre `Math.max`, sonra üst sınıra göre `Math.min` ile sınırlar. Sınırların ters verilmesi hata üretmez; iki `Math` çağrısının sırası korunur.

`isMultipleOf`, signed int kalanını sıfırla karşılaştırır. Java'nın `Integer.MIN_VALUE % -1` kenarı da sarılı sonuç olan sıfırı verir. `wrapDegrees(int)` önce `angle % 360` ile Java signed kalanını alır; `180` ve üstünü 360 azaltır, `-180` altını 360 artırır ve sonucu `[-180, 180)` aralığına taşır.

Senaryo girdileri tamsayıları decimal string olarak taşır. Oracle doğrudan sabitlenmiş Java 26.2 client JAR içindeki `Mth` çağrılarını yürütür ve sonucu string veya boolean olarak kaydeder; Rust adaptörü aynı JSON sözleşmesini üretir. Beklenen değerler Rust'tan türetilmez.

İncelenen kaynak konumları: `net/minecraft/util/Mth.java:74-100` (`abs`, `clamp`), `:178-202` (`isMultipleOf`, `wrapDegrees(int)`).

Kapsam dışı kalanlar: sıfıra bölme istisnasının ayrıntılı hata nesnesi, `wrapDegrees` long/float/double overload'ları, `packDegrees`/`unpackDegrees`, float/double clamp ve diğer Mth yardımcıları.


## Client-parity doğrulaması

`src/math.rs`, Java `Mth` mutlak değer, tamsayı clamp, katlık ve int açı sarmalama yardımcılarını Java'nın sınır ve signed kalan kurallarıyla eşler. `examples/behavior_adapter.rs` bu API'leri JSONL senaryolarına bağlar; beklenen değerler oracle çıktısından gelir.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- `abs`, `clamp`, `isMultipleOf` ve `wrapDegrees(int)` için 25 özgün senaryo hazırlandı.
- `Integer.MIN_VALUE`, `Long.MIN_VALUE`, ters clamp sınırları, negatif katlık ve `[-180, 180)` açı uçları karşılaştırıldı.
- Rust debug ve release adaptörleri güncel **2299/2299** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.
- Math ve birleşik `verify` çalıştırmaları Java referansını iki JVM tekrarıyla yeniden doğruladı.

Math referans kimliği: `9421a309da79905eddc1ce85361a8baef1e5c3a592c7326ac123cb13c20a6508`. Birleşik referans kimliği: `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b`.

Adapter hashleri: debug `6e9d1efd81eb178582a5369aed3a170bdc729264a5335bdded96c50bfb50b3ad`, release `130c76c303ffd5d73c67bb1f8b81e3d59bfb0c7a484c6d747eff32d1dcdfcb828`.

Yerel raporlar: `compare-00tgiqc4` ve `compare-aylr4ozm` (math debug/release), `compare-5bjkz31s` ve `compare-zfkba6ts` (birleşik debug/release), `verify-t1_8mau7` (math) ve `verify-mgwg6r2l` (birleşik) `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt seçili sıfır olmayan bölenlerle `Mth.abs(int)`, `clamp(int,int,int)`, `clamp(long,long,long)`, `isMultipleOf(int,int)` ve `wrapDegrees(int)` sonuçlarıyla sınırlıdır. Sıfıra bölme istisnasının hata biçimi, açı ve tamsayı overload'larının geri kalanı ve bu yardımcıları kullanan istemci çağrı zincirleri sonraki dilimlerdir. `.data/` altındaki Java runtime, capture ve çıktılar yalnızca yerel test girdisi olarak kalır.
