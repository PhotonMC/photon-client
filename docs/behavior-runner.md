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
- `compare --adapter /mutlak/yol/program`: aynı girdileri belirtilen executable'a verir ve kayıtlı Java sonucuyla karşılaştırır. Rust wire/timer/math/utf8/binary/NBT adaptörü mevcut; bu komut adaptör verilmediğinde **2** ile çıkar. Derleme ve karşılaştırma komutları [binary dilim belgesindedir](binary-parity.md).
- `test_runner.py`: yalnızca yanlış başarı, bozuk referans ve hatalı süreç davranışlarını denetleyen sentetik altyapı kontrolleri. Java veya asset gerektirmez.

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

Wire kapsamı işaretli min/max, 7 bit sınırları, negatif değerler, kesilmiş/fazla uzun/noncanonical kodlamalar ve kalan baytları içerir. Her `write` çağrısında orijinal decoder da çağrılır; bağımsız sentetik `read` girdileri ayrıca bulunur. UTF-8 ayrı ailede Java referansı olarak hazır; diğer P03 biçimleri henüz kapsanmıyor.

JSON protokolünü konuşan bir executable'ın programlama dili araç tarafından kanıtlanmaz. `examples/behavior_adapter.rs` mevcut gerçek Rust wire/timer/math adaptörüdür; rapor executable SHA-256'sını ve seçili kapsamı kaydeder. Altyapı denemelerinde kullanılan sentetik programlar oyun eşitlik kanıtı değildir.

## Aile ve operation sözleşmeleri

Runner'ın ortak JSONL ve kesin karşılaştırma kuralları yukarıdaki protokolde tanımlıdır. Aileye ve operation'a özgü alanlar, Java kaynak incelemesiyle birlikte ilgili contract/parity belgesinde tutulur; senaryo girdilerinin tek üretim noktası tools/behavior/cases.py dosyasıdır.

| Aile | Sözleşme sınırı |
| --- | --- |
| math | Sayısal, geometrik, koordinat, shape, random ve Mth dilimleri |
| timer | Tick, artık zaman, hız ve pause/freeze davranışı |
| wire | VarInt/VarLong encode/decode ve hata/tüketim davranışı |
| utf8 | UTF-8 sınırları, bozuk diziler ve hata tüketimi |
| binary | Vec3/LP Vec3, UUID, konum, chunk/section, array ve bitset kodlamaları |
| nbt | Bileşik, liste ve primitive array okuma/yazma davranışı |

Yeni bir operation eklendiğinde kendi kaynak incelemesi, oracle branch'i, senaryoları ve contract/parity belgesi eklenir. Bu genel runner belgesinde hareketli senaryo toplamı veya operation başına sayaç tutulmaz.

## Doğrulanan mevcut kapsam

Runner altı aileyi aynı JSONL sözleşmesiyle işler: math, timer, wire, utf8, binary ve nbt. Her seçili capture iki Java çalıştırmasıyla tekrarlanabilirliği, adapter karşılaştırması ise kayıtlı Java çıktısıyla kesin eşitliği denetler.

| Aile | Gözlenen kapsam |
| --- | --- |
| math | Sayısal ve geometrik yardımcılar, koordinat/shape yolları, legacy random akışları ve ilgili Mth dilimleri |
| timer | Tick üretimi, artık zaman, hız, pause/freeze ve sınır koşulları |
| wire | VarInt/VarLong encode/decode, sınırlar, tüketim ve hatalar |
| utf8 | UTF-8/UTF-16 sınırları, bozuk diziler, başlıklar ve hata tüketimi |
| binary | Vec3/LP Vec3, UUID, konum, chunk/section, long array ve bitset kodlamaları |
| nbt | Bileşik, liste ve primitive array okuma/yazma ile sınır davranışı |

Kesin senaryo listesi generator'dan, referans kimliği capture manifest'inden, dilim sınırları ise ilgili contract/parity belgelerinden okunur. Bu bölüm hareketli corpus toplamını tekrar etmez.

Bu kapsam headless yardımcı ve veri davranışlarıyla sınırlıdır. Oynanabilir istemci, pencere/input, gerçek Pumpkin oturumu, GPU frame yakalama, audio capture ve tam user-facing akışlar henüz doğrulanmış değildir.
