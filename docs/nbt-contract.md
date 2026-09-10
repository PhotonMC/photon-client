# P03.c — NBT referans sözleşmesi (bileşikler, listeler, diziler)

Bu dilimin **31 Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](nbt-parity.md)). Kapsam; null/boş kökleri, ilkel ve string alanları, iç içe bileşikleri, listeleri ve byte/int/long dizilerini içerir. Kota/derinlik sınırları ve SNBT bu dilime dahil değildir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. Kaynak köküne göre `network/FriendlyByteBuf.java` içindeki `readNbt`/`writeNbt` ve `nbt/NbtIo.java` içindeki `readAnyTag`/`writeAnyTag` incelendi. Adaptör doğrudan orijinal sınıfları çağırır; üst katman paket mantığı veya gerçek ağ bağlantısı bu testte yapılmaz.

Gerekli JAR SHA-1 değerleri `tests/behavior/reference.json` içinde sabitlendi. Yeni oyun bağımlılığı veya Rust kütüphanesi eklenmedi.

## Davranış sözleşmesi

- Kök boş bileşik, ilkel alanlı bileşik, iç içe bileşik veya null olabilir. Null kök tek `00` baytı, boş bileşik `0a00` olarak taşınır.
- Alan türleri `byte`, `short`, `int`, `long`, `float`, `double`, `string`, `byte-array`, `list`, `compound`, `int-array` ve `long-array` olarak taşınır. Tamsayılar işaretli ondalık string, float/double ham bit hex, diziler string dizisi olarak gözlenir.
- Liste yükü element türü, eleman sayısı ve adlandırılmamış payload'lardan oluşur. Boş listede element türü `0` yazılır; mevcut testlerdeki listeler tek türden veya bileşik elemanlardan oluşur.
- Okuma `FriendlyByteBuf.readNbt` kullanır: bileşik değilse payload tüketildikten sonra `DecoderException`, bozuk gövde `ReportedNbtException`, boş girdide `EncoderException` üretilir. Hata kategorileri sırasıyla `decoder_error` ve `encoder_error` olarak kaydedilir. Kesik okumalarda tüketim denenen baytları yansıtır.
- Okuma sonucu kök ve iç bileşik alanları ada göre sıralanır. Java exception sınıfı yalnızca `diagnostics` içindedir.

## Senaryolar ve JSONL

| Grup | Sayı | Girdiler |
| --- | ---: | --- |
| NBT yazma | 9 | Boş kök, 7 ilkel tür ve null kök |
| NBT ağaç yazma | 8 | İç içe bileşik, int/string listeleri, boş liste, byte/int/long dizileri ve bileşik listesi |
| NBT okuma | 14 | Boş, null, el yapımı ilkel/iç içe/liste yükleri, bileşik olmayan kök, kesik girdiler ve boş girdi |

`family: "nbt"`, `operation: "read" | "write"` ortak girdilerdir. Düz yazmada `fields` dizisi, ağaç yazmada `tree`, null kökte `null_root` bulunur. Okumada `hex` başlık/gövdeyi taşır. Başarılı okuma `fields` veya `null` içerir; her durumda `consumed` ve `remaining_hex` bulunur.

## Çalıştırma ve kanıt

Depo kökünden:

```bash
python3 tools/behavior/run.py capture --family nbt
python3 tools/behavior/run.py verify --family nbt
cargo build --locked --offline --example behavior_adapter
python3 tools/behavior/run.py compare --family nbt --adapter target/debug/examples/behavior_adapter
```

31 senaryo iki ayrı JVM'de aynı sonucu verdi. NBT referans kimliği: `d59af1e80d3b597d7fa2e4c2c4db8efbe3c81de0185621a84f7a334146ae66b2`. Ham girdiler/çıktılar yalnızca `.data/` altında tutulur.

Altı ailenin 2549 senaryolu birleşik referansı iki JVM'de doğrulandı. Birleşik referans kimliği: `510e1d4575f7dd92a09a6d3f896c2b8077a8fb2c01dd57ea32b9b48ce94a7583`.

NBT debug ve release Rust adaptörleri **31/31** eşleşti. Birleşik debug ve release karşılaştırmaları da **2549/2549** geçti. Yerel NBT raporları `compare-77gd9nar` (debug) ve `compare-gkct5sa5` (release); birleşik raporlar `compare-qz7665of` (debug) ve `compare-2e2muf1s` (release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

Bu doğrulama seçili NBT corpus'u ile sınırlıdır. Kota, derinlik sınırı, SNBT ve oyun içindeki daha geniş istemci davranışları ayrı eşitlik dilimleridir.
