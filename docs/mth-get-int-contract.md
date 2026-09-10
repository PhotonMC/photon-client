# Mth getInt davranış sözleşmesi

Bu dilim, Java Edition 26.2 `net.minecraft.util.Mth` içindeki `getInt(String, int)` yordamını ve onun pinned Apache Commons Lang 3.20.0 bağımlılığını kapsar.

`getInt`, `NumberUtils.toInt` üzerinden `Integer.parseInt` çağrısını kullanır. Geçerli decimal metin, isteğe bağlı ASCII `+` veya `-` işareti ve Java `Character.digit` tarafından tanınan decimal karakterlerle parse edilir. Boşluklar otomatik temizlenmez. Null, boş metin, tek başına işaret, biçim hatası ve i32 aralığı dışındaki sonuçlarda yardımcı default değeri döndürür.

Behavior runner `input` alanını string veya JSON null, `default` alanını signed decimal i32 string olarak taşır. Java oracle pinned client JAR ve commons-lang3 3.20.0 ile doğrudan `Mth.getInt` çağırır; beklenen sonuçlar Rust uygulamasından türetilmez. Rust tarafı Java'nın decimal digit ve i32 sınırlarını aynı parse sırasıyla uygular.

İncelenen kaynak konumları: `net/minecraft/util/Mth.java:279-281`; bağımlılık kimliği `org.apache.commons:commons-lang3:3.20.0`.

Kapsam dışı kalanlar: `smallestEncompassingPowerOfTwo`, `frac`, `getSeed`, JSON sayısal türlerinin metne dönüştürülmesi ve `getInt` kullanan istemci çağrı zincirleri.
