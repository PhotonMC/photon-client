# P03.c — NBT Rust davranış dilimi (düz bileşikler)

`src/nbt.rs`, null/boş kökler ve ilkel/string alanlı düz bileşikleri içerir. `examples/behavior_adapter.rs` üretim kütüphanesini mevcut JSONL senaryolarına bağlar; beklenen değer üretmez. Listeler, diziler, iç içe bileşikler, kota/derinlik ve SNBT bu dilime dahil değildir.

## Referans ve davranış

Referans Java 26.2 / protokol 776; `FriendlyByteBuf` NBT yolları ve `NbtIo` ikili formatı kaynak incelemesinde belirlendi. `NbtOracle.java` doğrudan orijinal sınıfları çağırır. Ayrıntılı sözleşme ve 18 senaryoluk kapsam [NBT referans belgesindedir](nbt-contract.md).

| Rust API | Korunan davranış |
| --- | --- |
| `read_nbt` | Boş girdide `encoder_error`; bileşik-olmayan kökte gövde tüketilip `decoder_error`; kesik gövdede denenen baytlar tüketilip `decoder_error` |
| `write_nbt` | Null kök tek `00` baytı; bileşik `0a` + alanlar + `00`; alanlar verilen sırada yazılır |
| Alan kodlaması | Ağ bayt sırası, string `u16` uzunluk öneki; bu dilimde ASCII girdilerde standart/modifiye UTF-8 aynıdır |

Çok alanlı yazımda Java `HashMap` sırası kullanılmaz; yazma senaryoları tek alanlıdır, okuma sonucu ada göre sıralıdır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- 655 senaryoluk kayıtlı Java referansı iki yeni JVM çalıştırmasıyla doğrulandı.
- Debug ve release Rust adaptörleri **655/655** eşleşti: 238 math + 22 timer + 144 wire + 141 utf8 + 92 binary + 18 nbt.
- Biçim, Clippy ve paket kapsamı kontrolleri geçti; `.data/` ve test adaptörü paket dışında kaldı.

Birleşik referans kimliği: `f14752fc7c658af95024aea3bbe6011ebc3423db5a01698e6178e0f8ce765081`. NBT ailesi kimliği: `62d5373d5e6a9b1bbca591baca712012461907aa21f3dcc62d267434af797920`.

Yerel raporlar `.data/runs/behavior/` altında bulunur.

## Yeniden çalıştırma ve sonraki dilim

Depo kökünde, yerel referans ve Cargo önbelleği mevcutken:

```bash
cargo build --locked --offline --example behavior_adapter
python3 tools/behavior/run.py compare --adapter target/debug/examples/behavior_adapter
```

Kayıtlı Java referansını yeniden ölçmek için `verify` kullanılır. P03.c kapsamında ağaç/liste/dizi için [13 Java referans testi](nbt-contract.md) hazır; sıradaki küçük dilim Rust implementasyonudur. Sonraki dilim bağımlılık sırasına göre seçilir.
