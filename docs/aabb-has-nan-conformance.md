# P01.d — AABB hasNaN sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilimin **6 Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](#client-parity-doğrulaması)). Kapsam, Java `AABB.hasNaN()` sorgusunun kutuda saklanan altı endpoint'in herhangi birinde NaN bulunup bulunmadığını bildirmesini içerir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/AABB.java:438-445` incelendi. `MathOracle` `hasNaN()` metodunu doğrudan çağırır; endpoint'leri yeniden tarayan bir oracle hesabı kullanmaz.

## Davranış sözleşmesi

- `minX`, `minY`, `minZ`, `maxX`, `maxY` ve `maxZ` sırayla `Double.isNaN` ile kontrol edilir.
- Herhangi bir saklanan endpoint NaN ise sonuç `true`, altı endpoint de sonlu veya sonsuz fakat NaN değilse sonuç `false` olur.
- AABB kurucusunun `Math.min`/`Math.max` normalizasyonu korunur; bir eksenin giriş uçlarından biri NaN olduğunda o eksenin saklanan uçları Java davranışına göre NaN olabilir.
- Signed zero ve pozitif/negatif sonsuz NaN sayılmaz. Sorgu kutuyu değiştirmez.

## Senaryolar ve JSONL

| Grup | Sayı | Girdiler |
| --- | ---: | --- |
| Sonlu | 1 | NaN içermeyen birim kutu |
| NaN yayılımı | 3 | X alt ucu, Y üst ucu ve Z'nin iki ucu |
| Sonsuzluk | 2 | Pozitif ve negatif sonsuz uçlar |

`family: "math"`, `operation: "aabb"` girdileridir. `steps` içinde `op: "has_nan"` bulunur. Her sonuç `has_nan` boolean alanını taşır.

## Java yakalama

Depo kökünden:

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py verify --family math
cargo build --locked --offline --example behavior_adapter
python3 tools/behavior/run.py compare --family math --adapter target/debug/examples/behavior_adapter
```

HasNaN senaryolarını içeren math referans kimliği `1b628a5ed1aa6d5d7f23933491eed86b44ffdbc00f80560b0ddbe51afb57e990` oldu. Bu referans iki JVM çalıştırmasıyla kararlı bulundu. Birleşik altı aile referansı `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b` kimliğiyle iki JVM'de doğrulandı.

Rust debug ve release adaptörleri math ailesinde 406/406, birleşik corpus'ta 2801/2801 eşleşti. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.


## Client-parity doğrulaması

`crates/voxel-core/src/aabb.rs`, Java `AABB.hasNaN()` sorgusunun altı saklanan endpoint'i `is_nan` ile kontrol etmesini uygular. `examples/behavior_adapter.rs` `has_nan` adımını JSONL math senaryolarına bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- 406 senaryoluk math referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **406/406** math senaryosunda eşleşti.
- Altı yeni senaryoda sonlu kutu, üç NaN yerleşimi ve pozitif/negatif sonsuzluk karşılaştırıldı.
- Birleşik altı aile corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `1b628a5ed1aa6d5d7f23933491eed86b44ffdbc00f80560b0ddbe51afb57e990`.

Birleşik referans kimliği: `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b`.

Yerel raporlar: `compare-cvrcdo06` (math debug), `compare-t0m86qa6` (math release), `compare-5bjkz31s` (birleşik debug) ve `compare-zfkba6ts` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt yalnızca `AABB.hasNaN()` ve mevcut AABB adım adaptörüyle sınırlıdır. `distanceToSqr` ayrı dilimde tamamlandı; AABB move overload'ları [ayrı kanıtta tamamlandı](aabb-move-overloads-conformance.md); [`collidedAlongVector`](aabb-collided-along-vector-conformance.md) ayrı dilimde tamamlandı ve VoxelShape clip tamamlandı; istemci hedefleme davranışları sonraki dilimlerdir.
