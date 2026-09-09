# P01.c — BlockPos ve section-relative Rust davranış dilimi

`src/binary.rs`, mevcut BlockPos/SectionPos paketleme tiplerine Java uyumlu offset, min/max ve section-relative bit alanı işlemlerini ekler. `examples/behavior_adapter.rs` aynı API'leri JSONL senaryolarına bağlar; beklenen değer üretmez.

## Referans ve davranış

Referans Java 26.2 / protokol 776; `BlockPos.java` ve `SectionPos.java` kaynak dalları incelendi. Math oracle doğrudan sabitlenen Java JAR sınıflarını çağırır. Ayrıntılı sözleşme ve 10 senaryoluk kapsam [referans belgesindedir](blockpos-contract.md).

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
- Altı ailenin birleşik corpus'u **1467/1467** senaryoda hem debug hem release adaptörüyle eşleşti.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `65c7ce5a63a1d8c7cc7b6ba7c60de04aa1205c8ab89967bcc45e569c80e35788`.

Birleşik referans kimliği: `5cb121b3bc1e12571ad171166a785300fd6e581cf78813e4ddaf2b99d19ab7d1`.

Yerel raporlar: `compare-i09iy6u8` (math debug), `compare-8gjbo7nd` (math release), `compare-lsdsnr89` (birleşik debug) ve `compare-b5y_ka7p` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt 10 yeni senaryoyla sınırlıdır. BlockPos/SectionPos stream, rotation, relative direction, world bounds ve istemciye bağlı koordinat davranışları sonraki dilimlerdir.
