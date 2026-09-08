# P01.c — BlockPos ve section-relative referans sözleşmesi

Bu dilimin **10 Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](blockpos-parity.md)). Kapsam; `BlockPos.offset`, bileşen bazlı `BlockPos.min`/`max` ve `SectionPos.sectionRelativePos` ile üç relative bileşenin açılımıdır. BlockPos/SectionPos'in diğer geometri, stream ve codec yardımcıları bu dilime dahil değildir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. Kaynak köküne göre `core/BlockPos.java` içindeki `offset`, `min` ve `max` ile `core/SectionPos.java` içindeki `sectionRelativePos`, `sectionRelativeX`, `sectionRelativeY` ve `sectionRelativeZ` incelendi. Adaptör doğrudan orijinal sınıfları çağırır. Yeni bağımlılık eklenmedi.

## Davranış sözleşmesi

- `BlockPos.offset` üç i32 eksenini Java int toplaması gibi sararak ekler; sıfır offset mevcut değeri korur.
- `BlockPos.min` ve `max` üç ekseni bağımsız karşılaştırır.
- `SectionPos.sectionRelativePos`, x/z/y relative koordinatlarını 12 bitlik `x << 8 | z << 4 | y` düzeninde paketler. Relative x/y/z açıcıları ilgili 4 bit alanı döndürür.
- Sonuçlar işaretli ondalık string olarak taşınır; relative paket değeri bu kapsamda 0–4095 aralığındadır.

## Senaryolar ve JSONL

| Grup | Sayı | Girdiler |
| --- | ---: | --- |
| BlockPos zinciri | 4 | Sıfır offset, negatif section sınırları, i32 taşma sarması ve ters min/max çağrıları |
| Section-relative | 6 | Köken, 15/16 sınırları, negatif koordinatlar ve i32 sınır komşuları |

`family: "math"`, `operation: "blockpos" | "section_relative"` ortak girdilerdir. BlockPos zincirinde başlangıç `x`/`y`/`z` ve sıralı `steps` bulunur; sonuç `events` içinde üç koordinat string'i taşır. Section-relative sonucunda paketlenmiş `relative` ile açılmış `x`, `y`, `z` bulunur.

## Çalıştırma ve kanıt

Depo kökünden:

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py verify --family math
cargo build --locked --offline --example behavior_adapter
python3 tools/behavior/run.py compare --family math --adapter target/debug/examples/behavior_adapter
```

317 math senaryosu iki ayrı JVM'de aynı sonucu verdi. Math referans kimliği: `65c7ce5a63a1d8c7cc7b6ba7c60de04aa1205c8ab89967bcc45e569c80e35788`. Ham girdiler/çıktılar yalnızca `.data/` altında tutulur.

Altı ailenin 884 senaryolu birleşik referansı iki JVM'de doğrulandı. Birleşik referans kimliği: `edc0c8336e5ffc79d1a744d6ddd301865a855ae2601c261b91b46cb5c453ddb9`.

Math debug ve release Rust adaptörleri **317/317** eşleşti; birleşik debug ve release karşılaştırmaları da **884/884** geçti. Yerel raporlar `compare-i09iy6u8` (math debug), `compare-8gjbo7nd` (math release), `compare-cnmcrrgh` (birleşik debug) ve `compare-9l5dyku9` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

Bu doğrulama seçili BlockPos/section-relative corpus'u ile sınırlıdır. BlockPos/SectionPos'in diğer yardımcıları ve istemci içindeki daha geniş koordinat davranışları ayrı eşitlik dilimleridir.
