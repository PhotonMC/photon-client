# P03.c — NBT referans sözleşmesi (düz bileşikler)

Bu dilimin **19 Java davranış testi hazırdır; Rust implementasyonu henüz yoktur**. Listeler, diziler, iç içe bileşikler, kota/derinlik ve SNBT bu dilime dahil değildir.

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
| NBT yazma | 10 | Boş, 7 ilkel tür, tam düz bileşik ve null kök |
| NBT okuma | 9 | Boş, null, el yapımı bayt/string yükleri, bileşik-olmayan kök, kesik/etiket/gövde ve boş girdi |

`family: "nbt"`, `operation: "read" | "write"` ortak girdilerdir. Yazmada `fields` dizi veya `null_root` bulunur; okumada `hex` başlık/gövdeyi taşır. Başarılı okuma `fields` veya `null` içerir; her durumda `consumed` ve `remaining_hex` bulunur.

## Çalıştırma ve kanıt

Depo kökünden:

```bash
python3 tools/behavior/run.py capture --family nbt
python3 tools/behavior/run.py verify --family nbt
```

19 senaryo iki ayrı JVM'de aynı sonucu verdi. NBT referans kimliği: `fd31f8c9934365989b3f3e6062d860c084495b2fa4d1eb7f7671b96ae784539a`. Ham girdiler/çıktılar yalnızca `.data/` altında tutulur.

Toplam 656 Java senaryosu yakalandı ve doğrulandı. Birleşik referans kimliği: `6037eec255f7ddd5cb1976fbf9a20bdc4120168f5395e1dcca9aa6164c8f0630`. Eski kayıtlar değiştirilmedi.

Mevcut Rust adaptörü NBT içermediğinden varsayılan bütün-aile koşusu başarısız olur. Sonraki adım bu sabit referansa karşı NBT Rust davranış dilimidir.
