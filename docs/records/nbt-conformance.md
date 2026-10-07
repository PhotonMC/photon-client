# P03.c — NBT referans sözleşmesi (bileşikler, listeler, diziler) — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilimin **31 Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](#client-parity-doğrulaması)). Kapsam; null/boş kökleri, ilkel ve string alanları, iç içe bileşikleri, listeleri ve byte/int/long dizilerini içerir. Kota/derinlik sınırları ve SNBT bu dilime dahil değildir.

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

Altı ailenin 2801 senaryolu birleşik referansı iki JVM'de doğrulandı. Birleşik referans kimliği: `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b`.

NBT debug ve release Rust adaptörleri **31/31** eşleşti. Birleşik debug ve release karşılaştırmaları da **2801/2801** geçti. Yerel NBT raporları `compare-77gd9nar` (debug) ve `compare-gkct5sa5` (release); birleşik raporlar `compare-5bjkz31s` (debug) ve `compare-zfkba6ts` (release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

Bu doğrulama seçili NBT corpus'u ile sınırlıdır. Kota, derinlik sınırı, SNBT ve oyun içindeki daha geniş istemci davranışları ayrı eşitlik dilimleridir.


## Client-parity doğrulaması

`crates/voxel-core/src/nbt.rs`, null/boş kökleri, ilkel ve string alanları, iç içe bileşikleri, listeleri ve byte/int/long dizilerini içerir. `examples/behavior_adapter.rs` üretim kütüphanesini mevcut JSONL senaryolarına bağlar; beklenen değer üretmez. Kota/derinlik ve SNBT bu dilime dahil değildir.

## Referans ve davranış

Referans Java 26.2 / protokol 776; `FriendlyByteBuf` NBT yolları ve `NbtIo` ikili formatı kaynak incelemesinde belirlendi. `NbtOracle.java` doğrudan sabitlenen JAR sınıflarını çağırır. Ayrıntılı sözleşme ve 31 senaryoluk kapsam [NBT referans belgesindedir](#sözleşme).

| Rust API | Korunan davranış |
| --- | --- |
| `read_nbt` | Null kökte end marker, bileşik kökte özyineli değer ağacı; bileşik olmayan kökte payload tüketilip `decoder_error`; kesik gövdede denenen baytlar tüketilip `decoder_error` |
| `write_nbt` | Null kök tek `00` baytı; bileşik `0a` + adlandırılmış payload'lar + `00`; verilen alan yazma sırası korunur |
| `FieldValue` | Primitive, string, byte/int/long array, list ve compound değerlerini özyineli olarak taşır |
| Liste kodlaması | Java element türü byte'ı, i32 eleman sayısı, adlandırılmamış payload'lar; boş liste için tür `0` |
| Alan kodlaması | Ağ bayt sırası, string `u16` uzunluk öneki; bu corpus'ta ASCII girdilerde standart/modifiye UTF-8 aynıdır |

Okuma alanları ve iç compound anahtarları ada göre sıralar; yazma alanların verilen sırasını korur. Java `HashMap` sırasının çok alanlı yazma üzerindeki etkisi bu küçük corpus'ta tek alanlı compound'larla sınırlandırılmıştır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- 31 senaryoluk NBT referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Debug ve release Rust adaptörleri **31/31** eşleşti: düz ve iç içe compound'lar, int/string/compound listeleri, boş liste, byte/int/long dizileri ve kesik okumalar.
- Altı ailenin birleşik corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets`, Clippy ve paket kapsamı kontrolleri geçti.

NBT referans kimliği: `d59af1e80d3b597d7fa2e4c2c4db8efbe3c81de0185621a84f7a334146ae66b2`.

Birleşik referans kimliği: `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b`.

Yerel NBT raporları: `compare-77gd9nar` (debug) ve `compare-gkct5sa5` (release); birleşik raporlar `compare-5bjkz31s` (debug) ve `compare-zfkba6ts` (release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Yeniden çalıştırma ve sınırlar

Depo kökünde, yerel referans ve Cargo önbelleği mevcutken:

```bash
cargo build --locked --offline --example behavior_adapter
python3 tools/behavior/run.py compare --family nbt --adapter target/debug/examples/behavior_adapter
```

Kayıtlı Java referansını yeniden ölçmek için aynı aile seçimiyle `verify` kullanılır. Bu kanıt seçili 31 NBT senaryosuyla sınırlıdır; kota, derinlik, SNBT ve oyun içindeki istemci davranışları eşitlik kapsamına dahil değildir.
