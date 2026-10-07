# P01.d — AABB intersect sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilimin **6 Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](#client-parity-doğrulaması)). Kapsam, Java `AABB.intersect(AABB)` çağrısının iki kutunun her eksendeki ortak aralığını üretmesini ve sonucu AABB kurucusundan geçirmesini içerir. Predicate overload'ları için ayrı [sözleşme](aabb-predicate-overloads-conformance.md) vardır; AABB contract, inflate, minmax, move, sizes, hasNaN, positions, axis min/max, distanceToSqr, ofSize ve clip ayrı dilimlerdir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/AABB.java:203-211` incelendi. `MathOracle` doğrudan `AABB.intersect` çağrısını yapar; beklenen uçları yeniden hesaplamaz.

## Davranış sözleşmesi

- Her eksenin min endpoint'i iki kutunun min değerlerinin `Math.max` sonucudur.
- Her eksenin max endpoint'i iki kutunun max değerlerinin `Math.min` sonucudur.
- Ham ortak aralık ters dönse bile sonuç Java `AABB` kurucusunun endpoint normalizasyonundan geçer; ayrık kutular bu nedenle aradaki aralığı temsil eden normalize edilmiş degenerate olmayan kutu üretebilir.
- Yüz yüze temas eden kutular ilgili eksende min ve max'ın eşit olduğu degenerate bir sonuç üretir. Signed zero seçimleri Java `Math.min`/`Math.max` ve kurucu semantiğine bağlıdır.
- Zincir adımları bir önceki kesişim sonucunu sonraki `intersect` çağrısına verir.

## Senaryolar ve JSONL

| Grup | Sayı | Girdiler |
| --- | ---: | --- |
| Örtüşme | 2 | İç içe ve kısmi eksen örtüşmesi |
| Sınır | 2 | Ayrık kutunun normalizasyonu ve yüz teması |
| Sayısal ayrıntı | 1 | Signed zero endpoint'leri |
| Zincir | 1 | Ardışık kesişim çağrıları |

`family: "math"`, `operation: "aabb"` girdileridir. `steps` içinde `op: "intersect"` ve `box` nesnesi bulunur. Kutu uçları f64 bit string olarak verilir; her adım sonrası sonuç `events` içinde taşınır.

## Java yakalama

Depo kökünden:

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py verify --family math
cargo build --locked --offline --example behavior_adapter
python3 tools/behavior/run.py compare --family math --adapter target/debug/examples/behavior_adapter
```

Intersect senaryolarını içeren math referans kimliği `067168f58b7d3c483bc26a542577bb0191d5cf0956a168754322b8805241bae5` oldu. Bu referans iki JVM çalıştırmasıyla kararlı bulundu. Birleşik altı aile referansı `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b` kimliğiyle iki JVM'de doğrulandı.

Rust debug ve release adaptörleri math ailesinde 382/382, birleşik corpus'ta 2801/2801 eşleşti. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.


## Client-parity doğrulaması

`crates/voxel-core/src/aabb.rs`, Java `AABB.intersect` metodunun eksen başına `Math.max`/`Math.min` endpoint hesaplarını ve sonuç kurucusunun normalizasyonunu uygular. `examples/behavior_adapter.rs` `intersect` adımını JSONL math senaryolarına bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- 382 senaryoluk math referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **382/382** math senaryosunda eşleşti.
- Altı yeni senaryoda iç içe/kısmi örtüşme, ayrık kutu normalizasyonu, yüz teması, signed zero ve zincir sonuçları karşılaştırıldı.
- Birleşik altı aile corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `067168f58b7d3c483bc26a542577bb0191d5cf0956a168754322b8805241bae5`.

Birleşik referans kimliği: `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b`.

Yerel raporlar: `compare-tgse4o_r` (math debug), `compare-t286vz_w` (math release), `compare-5bjkz31s` (birleşik debug) ve `compare-zfkba6ts` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt yalnızca `AABB.intersect(AABB)` ve mevcut AABB adım adaptörüyle sınırlıdır. `distanceToSqr` ayrı dilimde tamamlandı; [`collidedAlongVector`](aabb-collided-along-vector-conformance.md) ayrı dilimde tamamlandı ve VoxelShape clip tamamlandı; istemci hedefleme davranışları sonraki dilimlerdir.
