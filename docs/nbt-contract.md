# P03.c — NBT referans sözleşmesi (bileşikler, listeler, diziler)

Bu dilimin **31 Java davranış testi hazırdır; iç içe bileşikler, listeler ve diziler için Rust implementasyonu henüz yoktur**. Düz bileşikler için Rust karşılığı mevcuttur ([kanıt](nbt-parity.md)); önceki 18 girdisi ve sonucu değişmedi. Kota/derinlik ve SNBT bu dilime dahil değildir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. Kaynak köküne göre `network/FriendlyByteBuf.java` içindeki `readNbt`/`writeNbt` ve `nbt/NbtIo.java` içindeki `readAnyTag`/`writeAnyTag` incelendi. Adaptör doğrudan orijinal sınıfları çağırır; üst katman paket mantığı veya gerçek ağ bağlantısı bu testte yapılmaz.

Gerekli JAR SHA-1 değerleri `tests/behavior/reference.json` içinde sabitlendi. Yeni oyun bağımlılığı veya Rust kütüphanesi eklenmedi.

## Davranış sözleşmesi

- Kök boş bileşik, düz ilkel alanlı bileşik veya null'dur. Null kök tek `00` baytı olarak taşınır; boş bileşik `0a00` olarak taşınır.
- Alan türleri `byte`, `short`, `int`, `long`, `float`, `double` ve `string` ile sınırlıdır. Tamsayılar işaretli ondalık string, float/double ham bit hex, string düz metin olarak taşınır.
- Okuma `FriendlyByteBuf.readNbt` kullanır: bileşik değilse `DecoderException`, bozuk gövde `ReportedNbtException`, boş girdide `EncoderException` üretilir. `ReportedNbtException` ve `DecoderException` `decoder_error`, `EncoderException` `encoder_error` kategorisidir. Kesik okumalarda tüketim denenen baytları yansıtır.
- Yazma `FriendlyByteBuf.writeNbt` kullanır ve bu dilimde her zaman başarılıdır.
- Okuma sonucu alanlar ada göre sıralıdır. Java exception sınıfı yalnızca `diagnostics` içindedir.

## Senaryolar ve JSONL

| Grup | Sayı | Girdiler |
| --- | ---: | --- |
| NBT yazma | 10 | Boş, 7 ilkel tür ve null kök |
| NBT ağaç yazma | 8 | İç içe bileşik, int/string listeleri, boş liste, bayt/int/long dizileri ve bileşik listesi |
| NBT okuma | 13 | Boş, null, el yapımı ilkel/iç içe/liste yükleri, bileşik-olmayan kök, kesik girdiler |
| NBT okuma | 9 | Boş, null, el yapımı bayt/string yükleri, bileşik-olmayan kök, kesik/etiket/gövde ve boş girdi |

`family: "nbt"`, `operation: "read" | "write"` ortak girdilerdir. Yazmada `fields` dizi veya `null_root` bulunur; okumada `hex` başlık/gövdeyi taşır. Başarılı okuma `fields` veya `null` içerir; her durumda `consumed` ve `remaining_hex` bulunur.

## Çalıştırma ve kanıt

Depo kökünden:

```bash
python3 tools/behavior/run.py capture --family nbt
python3 tools/behavior/run.py verify --family nbt
```

31 senaryo iki ayrı JVM'de aynı sonucu verdi. NBT referans kimliği: `d59af1e80d3b597d7fa2e4c2c4db8efbe3c81de0161503a84f7a334146ae66b2`. Önceki 18 girdisi ve sonucu değişmedi. Ham girdiler/çıktılar yalnızca `.data/` altında tutulur.

Toplam 749 Java senaryosu yakalandı ve doğrulandı. Birleşik referans kimliği: `498d77d92c4ec42c9073256fe4d5ff56259297322c94741f00cfa6b799c9d92f`. Eski kayıtlar değiştirilmedi.

Mevcut Rust adaptörü ağaç/liste/dizi içermediğinden ilgili karşılaştırma başarısız olur. Sonraki adım bu sabit referansa karşı NBT ağaç Rust davranış dilimidir.
