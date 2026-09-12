# P01.d — AABB kutu kurucuları ve üreticileri sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, Java `AABB(BlockPos)`, `AABB(Vec3, Vec3)`, `AABB.unitCubeFromLowerCorner(Vec3)` ve `AABB.encapsulatingFullBlocks(BlockPos, BlockPos)` yollarını kapsar. Altı Java davranış testi hazırlandı ve Rust karşılığı [doğrulandı](#client-parity-doğrulaması).

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/AABB.java:22-55` içindeki dört kutu kurucusu/üreticisi incelendi. `MathOracle` her adımda Java çağrısını doğrudan yapar; beklenen endpoint'leri oracle içinde yeniden hesaplamaz.

## Davranış sözleşmesi

- `AABB(BlockPos)` BlockPos origin'i ile origin + 1 arasındaki unit box'ı üretir. Origin + 1 Java int işlemidir; i32 sınırında taşma gerçekleşmeden double kurucuya geçilmez.
- `AABB(Vec3, Vec3)` iki Vec3 bileşenini altı endpoint'li kurucuya aktarır. Kurucu her eksende `Math.min`/`Math.max` ile endpoint'leri normalize eder; NaN ve signed zero bu semantiği izler.
- `unitCubeFromLowerCorner(Vec3)` alt köşe bileşenlerini doğrudan min endpoint'lerine, her bileşene double `1.0` eklenmiş değerleri max endpoint'lerine verir.
- `encapsulatingFullBlocks(BlockPos, BlockPos)` her eksende iki integer origin'in min değerini alt, max değerine Java int `+1` uygulanmış değeri üst endpoint olarak seçer.
- Kurucular yeni AABB sonucu döndürür; sorgu zincirinde önceki kutunun yerini bu sonuç alır.

## Senaryolar ve JSONL

| Grup | Sayı | Girdiler |
| --- | ---: | --- |
| BlockPos kurucusu | 1 | Negatif koordinatlar ve i32 uç değerleri |
| Vec3 kurucusu | 1 | Ters endpoint'ler, signed zero ve özel değerler |
| Unit cube | 1 | Normal, signed zero, sonsuzluk ve NaN alt köşeleri |
| Full block kapsamı | 1 | Ters sıradaki iki BlockPos ve negatif koordinatlar |
| Full block taşması | 1 | Integer minimum/maksimumda üst endpoint `+1` davranışı |
| Kurucu zinciri | 1 | Dört üretici yolunun sıralı çağrıları |

`family: "math"`, `operation: "aabb"` girdileridir. `steps` içinde `op: "from_block_pos"`, `from_vec3`, `unit_cube_from_lower_corner` veya `encapsulating_full_blocks` bulunur. BlockPos koordinatları decimal i32 dizgeleri, Vec3 koordinatları f64 bit dizgeleri taşır. Her adım kutu endpoint'lerinin f64 bit snapshot'ını ekler.

## Java yakalama

Depo kökünden:

```bash
python3 tools/behavior/run.py capture --family math
```

Bu dilimin math referans kimliği `5dc02c58d7cf60cb11342c786c6a4d91d4d727099b7ac1faef9f51a5b5f192df` oldu. Bu referans iki JVM çalıştırmasıyla kararlı bulundu. Rust ve birleşik corpus kanıtı [parity belgesindedir](#client-parity-doğrulaması).

Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.


## Client-parity doğrulaması

`src/aabb.rs`, Java `AABB(BlockPos)`, `AABB(Vec3, Vec3)`, `AABB.unitCubeFromLowerCorner(Vec3)` ve `AABB.encapsulatingFullBlocks(BlockPos, BlockPos)` yollarını uygular. `examples/behavior_adapter.rs` bu üreticileri JSONL math adımlarına bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- 462 senaryoluk math referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **462/462** math senaryosunda eşleşti.
- Altı yeni senaryoda BlockPos ve Vec3 kurucuları, unit cube ve full block kapsamı; ters endpoint, signed zero, NaN, sonsuzluk ve integer `+1` taşması karşılaştırıldı.
- Birleşik altı aile corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `5dc02c58d7cf60cb11342c786c6a4d91d4d727099b7ac1faef9f51a5b5f192df`.

Birleşik referans kimliği: `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b`.

Yerel raporlar: `compare-a1guzop4` (math debug), `compare-40tsawyv` (math release), `compare-5bjkz31s` (birleşik debug) ve `compare-zfkba6ts` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt dört AABB kurucusu/üreticisi ve mevcut AABB adım adaptörüyle sınırlıdır. Çoklu AABB clip sonraki dilimdir.
