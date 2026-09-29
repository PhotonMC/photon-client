# P01.c — BlockPos ve section-relative referans sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilimin **10 Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](#client-parity-doğrulaması)). Kapsam; `BlockPos.offset`, bileşen bazlı `BlockPos.min`/`max` ve `SectionPos.sectionRelativePos` ile üç relative bileşenin açılımıdır. BlockPos/SectionPos'in diğer geometri, stream ve codec yardımcıları bu dilime dahil değildir.

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

Altı ailenin 2801 senaryolu birleşik referansı iki JVM'de doğrulandı. Birleşik referans kimliği: `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b`.

Math debug ve release Rust adaptörleri **317/317** eşleşti; birleşik debug ve release karşılaştırmaları da **2801/2801** geçti. Yerel raporlar `compare-i09iy6u8` (math debug), `compare-8gjbo7nd` (math release), `compare-5bjkz31s` (birleşik debug) ve `compare-zfkba6ts` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

Bu doğrulama seçili BlockPos/section-relative corpus'u ile sınırlıdır. BlockPos/SectionPos'in diğer yardımcıları ve istemci içindeki daha geniş koordinat davranışları ayrı eşitlik dilimleridir.


## Client-parity doğrulaması

`crates/voxel-core/src/binary.rs`, mevcut BlockPos/SectionPos paketleme tiplerine Java uyumlu offset, min/max ve section-relative bit alanı işlemlerini ekler. `examples/behavior_adapter.rs` aynı API'leri JSONL senaryolarına bağlar; beklenen değer üretmez.

## Referans ve davranış

Referans Java 26.2 / protokol 776; `BlockPos.java` ve `SectionPos.java` kaynak dalları incelendi. Math oracle doğrudan sabitlenen Java JAR sınıflarını çağırır. Ayrıntılı sözleşme ve 10 senaryoluk kapsam [referans belgesindedir](#sözleşme).

| Rust API | Korunan davranış |
| --- | --- |
| `BlockPos::offset` | Java i32 toplama sarması; sıfır offset'te aynı koordinatlar |
| `BlockPos::min` / `max` | Üç eksende bağımsız min/max |
| `SectionPos::section_relative_pos` | `x << 8 | z << 4 | y` 12 bit paketi |
| `SectionPos::section_relative_x/y/z` | Paketlenmiş değerin ilgili 4 bit alanları |

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- 317 senaryoluk math referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Debug ve release Rust adaptörleri **317/317** eşleşti: önceki 307 math senaryosu ve 10 yeni BlockPos/section-relative senaryosu.
- Altı ailenin birleşik corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `65c7ce5a63a1d8c7cc7b6ba7c60de04aa1205c8ab89967bcc45e569c80e35788`.

Birleşik referans kimliği: `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b`.

Yerel raporlar: `compare-i09iy6u8` (math debug), `compare-8gjbo7nd` (math release), `compare-5bjkz31s` (birleşik debug) ve `compare-zfkba6ts` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt 10 yeni senaryoyla sınırlıdır. BlockPos/SectionPos stream, rotation, relative direction, world bounds ve istemciye bağlı koordinat davranışları sonraki dilimlerdir.
