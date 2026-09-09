# P03.c — NBT Rust davranış dilimi

`src/nbt.rs`, null/boş kökleri, ilkel ve string alanları, iç içe bileşikleri, listeleri ve byte/int/long dizilerini içerir. `examples/behavior_adapter.rs` üretim kütüphanesini mevcut JSONL senaryolarına bağlar; beklenen değer üretmez. Kota/derinlik ve SNBT bu dilime dahil değildir.

## Referans ve davranış

Referans Java 26.2 / protokol 776; `FriendlyByteBuf` NBT yolları ve `NbtIo` ikili formatı kaynak incelemesinde belirlendi. `NbtOracle.java` doğrudan sabitlenen JAR sınıflarını çağırır. Ayrıntılı sözleşme ve 31 senaryoluk kapsam [NBT referans belgesindedir](nbt-contract.md).

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
- Altı ailenin birleşik corpus'u **1521/1521** senaryoda hem debug hem release adaptörüyle eşleşti.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets`, Clippy ve paket kapsamı kontrolleri geçti.

NBT referans kimliği: `d59af1e80d3b597d7fa2e4c2c4db8efbe3c81de0161521a84f7a334146ae66b2`.

Birleşik referans kimliği: `53cb6b5a4397567e26419e7466bcd8577c97308e626648d85f2a40bb55fb2406`.

Yerel NBT raporları: `compare-77gd9nar` (debug) ve `compare-gkct5sa5` (release); birleşik raporlar `compare-uh_quimp` (debug) ve `compare-233_yr3a` (release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Yeniden çalıştırma ve sınırlar

Depo kökünde, yerel referans ve Cargo önbelleği mevcutken:

```bash
cargo build --locked --offline --example behavior_adapter
python3 tools/behavior/run.py compare --family nbt --adapter target/debug/examples/behavior_adapter
```

Kayıtlı Java referansını yeniden ölçmek için aynı aile seçimiyle `verify` kullanılır. Bu kanıt seçili 31 NBT senaryosuyla sınırlıdır; kota, derinlik, SNBT ve oyun içindeki istemci davranışları eşitlik kapsamına dahil değildir.
