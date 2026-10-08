# P01.d — AABB minmax sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilimin **6 Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](#client-parity-doğrulaması)). Kapsam, Java `AABB.minmax(AABB)` çağrısının iki kutuyu her eksende kapsayan yeni AABB'yi üretmesini ve sonucu AABB kurucusundan geçirmesini içerir. AABB contract, inflate, intersect, move, sizes, hasNaN, positions, axis min/max, distanceToSqr, ofSize ve clip ayrı dilimlerdir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/AABB.java:213-221` incelendi. `MathOracle` doğrudan `AABB.minmax` çağrısını yapar; beklenen uçları yeniden hesaplamaz.

## Davranış sözleşmesi

- Her eksenin min endpoint'i iki kutunun min değerlerinin `Math.min` sonucudur.
- Her eksenin max endpoint'i iki kutunun max değerlerinin `Math.max` sonucudur.
- Sonuç Java `AABB` kurucusundan geçer; ayrık kutuların union'ı aradaki boşluğu da kapsayan tek kutuya dönüşür.
- Yüz yüze temas eden kutuların union'ı temas eksenindeki dış uçları korur. Signed zero seçimleri Java `Math.min`/`Math.max` ve kurucu semantiğine bağlıdır.
- Zincir adımları bir önceki union sonucunu sonraki `minmax` çağrısına verir.

## Senaryolar ve JSONL

| Grup | Sayı | Girdiler |
| --- | ---: | --- |
| Örtüşme | 2 | İç içe ve kısmi eksen örtüşmesi |
| Sınır | 2 | Ayrık kutu ve yüz teması |
| Sayısal ayrıntı | 1 | Signed zero endpoint'leri |
| Zincir | 1 | Ardışık union çağrıları |

`family: "math"`, `operation: "aabb"` girdileridir. `steps` içinde `op: "minmax"` ve `box` nesnesi bulunur. Kutu uçları f64 bit string olarak verilir; her adım sonrası sonuç `events` içinde taşınır.

## Java yakalama

Depo kökünden:

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py verify --family math
cargo build --locked --offline --example behavior_adapter
python3 tools/behavior/run.py compare --family math --adapter target/debug/examples/behavior_adapter
```

Minmax senaryolarını içeren math referans kimliği `a5af108bd9af6e253a66e4f7d2ec37609cab22b050af5da64c032db59e8c71c2` oldu. Bu referans iki JVM çalıştırmasıyla kararlı bulundu. Birleşik altı aile referansı `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b` kimliğiyle iki JVM'de doğrulandı.

Rust debug ve release adaptörleri math ailesinde 388/388, birleşik corpus'ta 2801/2801 eşleşti. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.


## Client-parity doğrulaması

`crates/photon-core/src/aabb.rs`, Java `AABB.minmax` metodunun eksen başına `Math.min`/`Math.max` endpoint hesaplarını ve sonuç kurucusunun normalizasyonunu uygular. `examples/behavior_adapter.rs` `minmax` adımını JSONL math senaryolarına bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- 388 senaryoluk math referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **388/388** math senaryosunda eşleşti.
- Altı yeni senaryoda iç içe/kısmi örtüşme, ayrık kutu union'ı, yüz teması, signed zero ve zincir sonuçları karşılaştırıldı.
- Birleşik altı aile corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `a5af108bd9af6e253a66e4f7d2ec37609cab22b050af5da64c032db59e8c71c2`.

Birleşik referans kimliği: `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b`.

Yerel raporlar: `compare-5tbvg_e5` (math debug), `compare-z90g_mxx` (math release), `compare-5bjkz31s` (birleşik debug) ve `compare-zfkba6ts` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt yalnızca `AABB.minmax(AABB)` ve mevcut AABB adım adaptörüyle sınırlıdır. `distanceToSqr` ayrı dilimde tamamlandı; [`collidedAlongVector`](aabb-collided-along-vector-conformance.md) ayrı dilimde tamamlandı ve VoxelShape clip tamamlandı; istemci hedefleme davranışları sonraki dilimlerdir.
