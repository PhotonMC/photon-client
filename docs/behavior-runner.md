# Yerel davranış testlerini çalıştırma

Python 3 standart kütüphanesi ve Java 25 JDK kullanılır; yeni Python paketi, Rust bağımlılığı veya test framework'ü yok. Java adaptörleri doğrudan sabitlenen Minecraft JAR'ındaki sınıfları çağırır. Beklenen sonuçlar Rust'tan veya test içinde yeniden yazılmış algoritmalardan üretilmez.

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
- `compare --adapter /mutlak/yol/program`: aynı girdileri belirtilen executable'a verir ve kayıtlı Java sonucuyla karşılaştırır. Henüz Rust adaptörü yok; bu komut adaptör verilmediğinde **2** ile çıkar.
- `test_runner.py`: yalnızca yanlış başarı, bozuk referans ve hatalı süreç davranışlarını denetleyen beş küçük sentetik altyapı testi. Java veya asset gerektirmez.

Tek aile seçmek için `--family wire` kullanılır; seçenek tekrarlanabilir. Seçilmezse mevcut bütün aileler çalışır. `--runtime /yerel/runtime` alternatif client/`libs` dizinini, `--jdk-home /jdk/koku` Java araçlarını seçer. Bu yollar yalnızca okunur. Gerekli referanslar yoksa indirme yapılmaz.

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

Her koşu `.data/runs/behavior/<komut>-<benzersiz>/` altında input, stdout/stderr, derlenen sınıflar, kimlik ve `report.json` tutar. Her capture `.data/captures/behavior/<kimlik>/` altında `inputs.jsonl`, `outputs.jsonl`, `manifest.json` tutar. Referans kimliği client/dependency hash'leri, Java ve javac sürüm çıktısı, OS/mimari, seçilen aileler, Java adaptör/çalıştırıcı kodu ve senaryo girdilerini kapsar. Girdi veya çıktı dosyasının bozulması hash denetimiyle reddedilir.

Kaynak, senaryo veya ortam değişirse yeni kimlik için açıkça `capture` çalıştırılır; eski kayıtlar korunur. Bu kayıtlar kriptografik imzalı güven zinciri değildir; yerel test girdilerini ve kazara değişiklikleri izler. Değişen baseline'ı otomatik kabul eden bir test koşusu yoktur.

## Adaptör protokolü

Executable, stdin'den UTF-8 JSONL alır; her girdi için aynı sırayla stdout'a tek JSON nesnesi döndürür. stdout'a log yazmaz; loglar stderr'e gider. Süre sınırı 60 saniyedir, hata veya eksik/bozuk çıktı başarı sayılmaz. Süreç shell üzerinden başlatılmaz; çalışma dizini ilgili yerel koşu dizinidir.

Her girdi `schema_version: 1`, `case_id`, `family`, `operation` ve operasyona özgü alanlar içerir. Her çıktı aynı `schema_version`/`case_id` ve bir `result` nesnesi içerir. İsteğe bağlı `diagnostics` nesnesi (örneğin Java exception sınıfı) karşılaştırılmaz; bütün `result` alanları tür ve değer açısından kesin karşılaştırılır. Liste/olay sırası korunur; JSON nesnesindeki alan yazım sırası önemli değildir. Eksik/fazla/tekrarlı/sırası değişmiş kayıt başarısızdır.

`wire` ailesi: `codec` alanı `varint` veya `varlong`; `write` girdisindeki `value` işaretli ondalık **string**, `read` girdisindeki `hex` küçük harfli bayt dizisidir. Yazma sonucunda `outcome`, `size`, `hex`, `decoded`, `consumed`, `remaining_hex`; okuma sonucunda `outcome`, `value` veya `error`, `consumed`, `remaining_hex` bulunur. `buffer_underflow` ve `too_big` Java'dan gözlenen hatanın dil bağımsız kategorileridir; sınıf adı yalnızca tanısal kayıttadır.

Wire corpus'u: 144 senaryo; işaretli min/max, 7 bit sınırları, negatif değerler, kesilmiş/fazla uzun/noncanonical kodlamalar ve kalan baytlar. Her `write` çağrısında orijinal decoder da çağrılır; bağımsız sentetik `read` girdileri ayrıca bulunur. Diğer P03 biçimleri henüz kapsanmıyor.

JSON protokolünü konuşan bir executable'ın programlama dili araç tarafından kanıtlanmaz. Gelecekte gerçek Rust adaptörü verilecek; rapor executable SHA-256'sını ve seçili kapsamı kaydeder. Altyapı denemelerinde kullanılan sentetik programlar oyun eşitlik kanıtı değildir.
