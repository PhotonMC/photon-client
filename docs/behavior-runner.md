# Yerel davranış testlerini çalıştırma

Python 3.10+ standart kütüphanesi ve Java 25 JDK kullanılır; yeni Python paketi veya test framework'ü yok. Rust JSONL adaptörü `serde_json` geliştirme bağımlılığını kullanır; üretim kütüphanesi haricî bağımlılık kullanmaz. Java adaptörleri doğrudan sabitlenen Minecraft JAR'ındaki sınıfları çağırır. Beklenen sonuçlar Rust'tan veya test içinde yeniden yazılmış algoritmalardan üretilmez.

Depo kökünden:

```bash
python3 tools/behavior/run.py preflight
python3 tools/behavior/run.py capture
python3 tools/behavior/run.py verify
python3 tools/behavior/test_runner.py
```

- `preflight`: client ve ilgili JAR hash'lerini, Java/javac sürümünü kontrol eder. Oyun veya test çalıştırmaz.
- `capture`: senaryoları iki ayrı Java sürecinde çalıştırır, çıktıları karşılaştırır ve kararlı referansı kaydeder. Aynı kimlikteki kaydı değiştirmez; mevcut kayıt farklı/bozuksa başarısız olur.
- `verify`: kayıtlı referansı değiştirmeden iki yeni Java çalıştırmasıyla karşılaştırır. Önceden `capture` yapılmış olmalı.
- `compare --adapter /mutlak/yol/program`: aynı girdileri belirtilen executable'a verir ve kayıtlı Java sonucuyla karşılaştırır. Rust wire/timer/math/utf8/binary adaptörü mevcut; bu komut adaptör verilmediğinde **2** ile çıkar. Derleme ve karşılaştırma komutları [binary dilim belgesindedir](binary-parity.md).
- `test_runner.py`: yalnızca yanlış başarı, bozuk referans ve hatalı süreç davranışlarını denetleyen beş küçük sentetik altyapı testi. Java veya asset gerektirmez.

Tek aile seçmek için `--family wire`, `--family math`, `--family timer`, `--family utf8`, `--family binary` veya `--family nbt` kullanılır; seçenek tekrarlanabilir. Seçilmezse mevcut bütün aileler çalışır. Aile seçimi referans kimliğinin parçasıdır; örneğin `verify --family timer` öncesinde aynı seçimle `capture --family timer` çalıştırılmalıdır. `--runtime /yerel/runtime` alternatif client/`libs` dizinini, `--jdk-home /jdk/koku` Java araçlarını seçer. Bu yollar yalnızca okunur. Gerekli referanslar yoksa indirme yapılmaz.

## Sonuç ve çıkış kodları

| Sonuç | Kod | Anlam |
| --- | ---: | --- |
| `preflight_ready` | 0 | Girdi ön kontrolü geçti; davranış doğrulanmadı |
| `reference_ready`, `rust_status=not_compared` | 0 | Java referansı kararlı/kayıtla eşit; Rust karşılaştırılmadı |
| `parity_passed` | 0 | Yalnızca seçilen senaryolarda verilen adaptör Java çıktısıyla eşleşti |
| `failed` | 1 | Çalıştırma/derleme/format hatası veya davranış farkı |
| `blocked` | 2 | Eksik/yanlış referans, yanlış JDK, eksik capture veya adaptör |

Referans komutunun başarılı olması istemcinin eşitliğini göstermez. Seçili bir ailede adaptörün geçmesi de diğer bölümlerin geçtiği anlamına gelmez. Genel CI'a yerel referans veya capture yüklenmez.

## Dosyalar ve kimlik

İzlenen dosyalar: `tools/behavior/` çalıştırıcı ve özgün girdi üreticisi; `tests/behavior/java/` özgün Java çağrı adaptörleri; `tests/behavior/reference.json` sürüm ve gerekli artifact hash'leri. Runtime için yalnızca seçili ailenin JAR'ları kullanılır; assetler, GPU, sunucu ve decompile kaynak ağacı bu ilk testlerin ön koşulu değildir.

Her koşu `.data/runs/behavior/<komut>-<benzersiz>/` altında input, stdout/stderr, derlenen sınıflar, kimlik ve `report.json` tutar. Rapordaki `reproduce` komutu `invocation_cwd` dizininden yeniden çalıştırılabilir. Her capture `.data/captures/behavior/<kimlik>/` altında `inputs.jsonl`, `outputs.jsonl`, `manifest.json` tutar. Referans kimliği client/dependency hash'leri, Java ve javac sürüm çıktısı, OS/mimari, seçilen aileler, Java adaptör/çalıştırıcı kodu ve senaryo girdilerini kapsar. Girdi veya çıktı dosyasının bozulması hash denetimiyle reddedilir.

Kaynak, senaryo veya ortam değişirse yeni kimlik için açıkça `capture` çalıştırılır; eski kayıtlar korunur. Bu kayıtlar kriptografik imzalı güven zinciri değildir; yerel test girdilerini ve kazara değişiklikleri izler. Değişen baseline'ı otomatik kabul eden bir test koşusu yoktur.

## Adaptör protokolü

Executable, stdin'den UTF-8 JSONL alır; her girdi için aynı sırayla stdout'a tek JSON nesnesi döndürür. stdout'a log yazmaz; loglar stderr'e gider. Süre sınırı 60 saniyedir, hata veya eksik/bozuk çıktı başarı sayılmaz. Süreç shell üzerinden başlatılmaz; çalışma dizini ilgili yerel koşu dizinidir.

Her girdi `schema_version: 1`, `case_id`, `family`, `operation` ve operasyona özgü alanlar içerir. Her çıktı aynı `schema_version`/`case_id` ve bir `result` nesnesi içerir. İsteğe bağlı `diagnostics` nesnesi (örneğin Java exception sınıfı) karşılaştırılmaz; bütün `result` alanları tür ve değer açısından kesin karşılaştırılır. Liste/olay sırası korunur; JSON nesnesindeki alan yazım sırası önemli değildir. Eksik/fazla/tekrarlı/sırası değişmiş kayıt başarısızdır.

`wire` ailesi: `codec` alanı `varint` veya `varlong`; `write` girdisindeki `value` işaretli ondalık **string**, `read` girdisindeki `hex` küçük harfli bayt dizisidir. Yazma sonucunda `outcome`, `size`, `hex`, `decoded`, `consumed`, `remaining_hex`; okuma sonucunda `outcome`, `value` veya `error`, `consumed`, `remaining_hex` bulunur. `buffer_underflow` ve `too_big` Java'dan gözlenen hatanın dil bağımsız kategorileridir; sınıf adı yalnızca tanısal kayıttadır.

Wire corpus'u: 144 senaryo; işaretli min/max, 7 bit sınırları, negatif değerler, kesilmiş/fazla uzun/noncanonical kodlamalar ve kalan baytlar. Her `write` çağrısında orijinal decoder da çağrılır; bağımsız sentetik `read` girdileri ayrıca bulunur. UTF-8 ayrı ailede Java referansı olarak hazır; diğer P03 biçimleri henüz kapsanmıyor.

JSON protokolünü konuşan bir executable'ın programlama dili araç tarafından kanıtlanmaz. `examples/behavior_adapter.rs` mevcut gerçek Rust wire/timer/math adaptörüdür; rapor executable SHA-256'sını ve seçili kapsamı kaydeder. Altyapı denemelerinde kullanılan sentetik programlar oyun eşitlik kanıtı değildir.

## Sayısal ve zamanlayıcı protokolü

`math/vec2`: girdide `x`, `y` ve `steps` bulunur. Float girdiler 8 haneli küçük harfli IEEE-754 hex string'leridir. `steps` içindeki `normalize` vektör normalizasyonunu, `scale` ise aynı kayıttaki `factor` çarpanıyla ölçeklemeyi çağırır. `result.events`, başlangıçta ve her işlemden sonra `x`, `y`, `length`, `length_squared` bitlerini içerir.

Diğer `math` operasyonları `floor`, `wrap_degrees`, `clamp`; `type` alanı `f32` veya `f64`'tür. `value` ve clamp'in `min`/`max` alanları türe göre 8/16 haneli hex bit string'idir. `floor` her iki overload için de i32 sonucu `result.value` ondalık string olarak döndürür; diğerleri `result.bits` üretir. Int/long overload'ları bu dilime dahil değil.

`timer/trace`: `tps`, `target_mspt` f32 bit string; `start_ms` i64 ondalık string'dir. Sıralı `steps`: `game`/`real` adımları `ms` i64 string; `pause`/`freeze` adımları `value` boolean; `target` adımı `value` f32 bit string alır. Sağlayıcı başlangıçta `target_mspt`, değişiklikten sonra verilen `target` değerini döndürür. Bu, gerçek Timer API'sine kontrollü girdi sağlar.

`result.events`, başlangıçta ve her adımdan sonra `game_delta`, `partial`, `partial_ignore_frozen`, `real_delta` alanlarını f32 bit string olarak taşır. Yalnızca `game` adımının sonucunda ayrıca `ticks` JSON integer vardır. Timer'ın iç alanları refleksiyonla değiştirilmez; yalnızca constructor ve gerçek public API çağrılır. Frame başına 10 tick sınırı bu Timer testine eklenmez.

## Doğrulanan mevcut kapsam

| Aile | Senaryo | Kapsam |
| --- | ---: | --- |
| `math` | 307 | Vec2/Vec3 zincirleri ve f32/f64 yardımcıları; Direction ve koordinat dönüşümleri; random dizileri ve AABB |
| `timer` | 22 | Tick sınırları/artık zaman, kontrollü hız değişimi, pause/freeze kombinasyonları, gerçek zaman sınırı, uzun frame ve 2^53 üzerindeki başlangıç saati |
| `wire` | 144 | VarInt/VarLong yazma/okuma, sınırlar, hatalar, tüketim ve kalan veri |
| `utf8` | 141 | UTF-16 birim sınırları, UTF-8 okuma/yazma, bozuk diziler, VarInt başlıkları ve hata tüketimi |
| `binary` | 116 | UUID/BlockPos/SectionPos/long array/BitSet okuma/yazma; sabit BitSet boyut sınırları |
| `nbt` | 31 | Düz/iç içe bileşik, int/string/bileşik listeleri, boş liste, bayt/int/long dizileri; son 13 senaryo yalnızca Java referansı |
| **Toplam** | **761** | P01.a/P01.b/P01.c/P01.d/P01.e/P02.a/P03.a/P03.b/P03.c için mevcut referans corpus'u |

7 Eylül 2026'da Java 25.0.4.1 / Linux x86_64 üzerinde ilk 346 senaryoluk corpus iki ayrı JVM sürecinde aynı sonuçları verdi. Kayıtlı referansa karşı `verify` de çalıştırıldı. Beş sentetik altyapı testi geçti. Komut düzeyinde ayrıca kontrollü replay, tek alan mutasyonu, boş adaptör çıktısı, eksik adaptör, eksik runtime ve yanlış JAR çıkışları sınandı. Bu replay programı yalnızca karşılaştırıcı kontrolüdür; Rust veya oyun implementasyonu değildir.

P00 bu ilk aileler için hazırdır; asset/GPU/gerçek ağ test altyapısı henüz yoktur. P01'in tüm matematik kapsamı, P02'nin oyun döngüsü ve P03'ün diğer veri biçimleri tamamlanmadı. Rust wire adaptörü debug/release profillerinde 144 senaryoda eşleşti; [kanıt ve sınırlar](wire-parity.md). Zamanlayıcı da 22 senaryoda eşleşti; [kanıt](timer-parity.md). Math ailesi 307 senaryoda eşleşti; [kanıt ve sınırlar](math-parity.md), [random/AABB kanıtı](random-aabb-parity.md). UTF-8 ailesi de 141 senaryoda eşleşti; [kanıt](utf8-parity.md). Binary ailesi 116 senaryoda eşleşti; [kanıt](binary-parity.md). NBT ailesinin düz bileşik kapsamı 18 senaryoda eşleşti; [kanıt](nbt-parity.md). Uygulanan kapsamda 748 senaryo debug/release profillerinde geçti. NBT ağaç/liste/dizi için 13 Java referans senaryosu hazır; bu kapsamda Rust eşitliği iddiası yok. [Binary sözleşmesi](binary-contract.md). [Vec3 sözleşmesi](vec3-contract.md).
