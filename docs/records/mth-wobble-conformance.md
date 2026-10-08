# Mth wobble(double) davranış sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, Java Edition 26.2 `net.minecraft.util.Mth` içindeki `wobble(double)` yordamını kapsar. 20 sentetik senaryo, koordinatın sabit seed türetmesini, tek çekişli gürültüyü, sınır değerlerini ve özel f64 girdileri kapsar.

Yordam önce `coord * 3000.0` çarpımını yapar ve `Mth.floor(double)` sonucunu `int` seed olarak kullanır. Bu seed ile `RandomSource.createThreadLocalInstance(long)` üzerinden `SingleThreadedRandomSource` kurulur. Kaynak sonraki tek `nextDouble()` çekişinden sonra şu ifadeyi kaynak sırasıyla hesaplar:

`coord + (2.0 * random - 1.0) * 1.0E-7 / 2.0`

Seed türetimi nedeniyle aynı koordinat deterministik bir gürültü sonucu üretir; koordinatın pozitif/negatif kesirli olması floor yönünü ve dolayısıyla seed'i değiştirir. `Mth.floor` dönüşümünde Java'nın `double` → `int` daraltma sınırları korunur. F64 çarpma, çıkarma, çarpma, bölme ve toplama sırası değiştirilemez; signed zero, subnormal, Infinity ve NaN sonuçları raw bitlerle gözlenir.

Oracle, Java `Mth.wobble(double)` çağrısını doğrudan yapar ve sonucu `result.bits` altında raw f64 bit string olarak döndürür. Beklenen değerler Rust uygulamasından türetilmez.

İncelenen kaynak konumları: `net/minecraft/util/Mth.java:670-672`, `net/minecraft/util/Mth.java:66-68`, `net/minecraft/util/RandomSource.java:31-32`, `net/minecraft/world/level/levelgen/SingleThreadedRandomSource.java:15-44` ve `net/minecraft/world/level/levelgen/BitRandomSource.java:47-54`.

Kapsam dışı kalanlar: no-arg `createThreadLocalInstance()`, diğer `RandomSource` implementasyonları, `Mth` random helper overload'ları ve `wobble` sonucunu kullanan istemci çağrı zincirleri.


## Client-parity doğrulaması

`crates/photon-core/src/math.rs` içindeki `mth_wobble_f64`, Java `Mth.wobble(double)` yordamının floor-derived seed, tek legacy LCG `nextDouble()` çekişi ve kaynak sıralı f64 gürültü eklemesini taşır. `examples/behavior_adapter.rs` aynı üretim API'sini JSONL senaryosuna bağlar; beklenen değerleri üretmez.

- Math ailesinin 2299 senaryoluk Java referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **2299/2299** math senaryosunda eşleşti.
- Birleşik altı aile corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.
- Wobble için 20 sentetik koordinat; signed zero, floor sınırı, subnormal, int daraltma sınırı, Infinity ve NaN payload'ı içerir.

Math referans kimliği: `9421a309da79905eddc1ce85361a8baef1e5c3a592c7326ac123cb13c20a6508`. Birleşik referans kimliği: `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b`.

Adapter hashleri: debug `6e9d1efd81eb178582a5369aed3a170bdc729264a5335bdded96c50bfb50b3ad`, release `130c76c303ffd5d73c67bb1f8b81e3d59bfb0c7a484c6d747eff32d1dcdfcb828`.

Yerel raporlar: `compare-00tgiqc4` ve `compare-aylr4ozm` (math debug/release), `compare-5bjkz31s` ve `compare-zfkba6ts` (birleşik debug/release), `verify-t1_8mau7` (math) ve `verify-mgwg6r2l` (birleşik) `.data/runs/behavior/` altında bulunur.

Bu kanıt Java 25.0.4.1 / Minecraft 26.2 referansında `Mth.wobble(double)` ve bu dilimin 20 girdisiyle sınırlıdır. No-arg thread-local random üretimi ve diğer random source implementasyonları bu dilimin dışındadır. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.
