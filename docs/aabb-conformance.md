# P01.d — AABB davranış sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilimin **5 Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](#client-parity-doğrulaması)). Kapsam, hareket delta'sının işaretine göre Java `AABB.contract(double,double,double)` çağrısının kutunun tek uç noktasını içeri taşımasını içerir. AABB inflate için ayrı [sözleşme](aabb-inflate-conformance.md), intersect için ayrı [sözleşme](aabb-intersect-conformance.md), move için ayrı [sözleşme](aabb-move-conformance.md), sizes için ayrı [sözleşme](aabb-sizes-conformance.md) ve hasNaN için ayrı [sözleşme](aabb-has-nan-conformance.md), positions için ayrı [sözleşme](aabb-positions-conformance.md), axis min/max için ayrı [sözleşme](aabb-axis-limits-conformance.md), distanceToSqr için ayrı [sözleşme](aabb-distance-conformance.md), ofSize için ayrı [sözleşme](aabb-of-size-conformance.md), endpoint setter'ları için ayrı [sözleşme](aabb-set-endpoint-conformance.md) vardır; clip ve diğer genişletme/kesişim işlemleri ayrı dilimlerdir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/AABB.java:129-155` incelendi. `MathOracle` doğrudan `AABB.contract` çağırır; beklenen uçları yeniden hesaplamaz.

## Davranış sözleşmesi

- Negatif delta alt endpoint'i `min -= delta` ile içeri taşır; pozitif delta üst endpoint'i `max -= delta` ile içeri taşır.
- Sıfır ve signed zero delta'ları hiçbir endpoint'i değiştirmez. Her eksen bağımsız işlenir.
- Sonuç Java `AABB` kurucusundan geçer; endpoint'lerin eksen bazlı min/max normalizasyonu korunur.
- Bu dilimde zincir adımları bir önceki `AABB` sonucunu sonraki çağrıya verir.

## Senaryolar ve JSONL

| Grup | Sayı | Girdiler |
| --- | ---: | --- |
| Tek yön | 2 | Tüm eksenlerde pozitif veya negatif contract delta'sı |
| Karışık işaret | 1 | Aynı çağrıda negatif, sıfır ve pozitif delta |
| Sıfır/zincir | 2 | Signed zero ve ardışık contract çağrıları |

`family: "math"`, `operation: "aabb"` girdileridir. `steps` içinde `op: "contract"` ve f64 bit string `x`, `y`, `z` delta'ları bulunur. Her adım sonrası kutu uçları f64 bit string olarak `events` içinde taşınır.

## Java yakalama

Depo kökünden:

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py verify --family math
cargo build --locked --offline --example behavior_adapter
python3 tools/behavior/run.py compare --family math --adapter target/debug/examples/behavior_adapter
```

Contract senaryolarını içeren math referans kimliği `3854d607b2b0b2e782a153bffb5f46918565f2df40d4a855940d960959a37113` oldu. Bu referans iki JVM çalıştırmasıyla kararlı bulundu. Birleşik altı aile referansı `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b` kimliğiyle iki JVM'de doğrulandı.

Rust debug ve release adaptörleri math ailesinde 371/371, birleşik corpus'ta 2801/2801 eşleşti. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.


## Client-parity doğrulaması

`crates/voxel-core/src/aabb.rs`, Java `AABB.contract` çağrısında delta işaretine göre min veya max endpoint'inin içeri taşınmasını uygular. `examples/behavior_adapter.rs` contract adımını JSONL math senaryolarına bağlar. Inflate overload'ları ve intersect işlemi ayrı [inflate kanıtında](aabb-inflate-conformance.md) ve [intersect kanıtında](aabb-intersect-conformance.md) doğrulanır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- 371 senaryoluk math referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **371/371** math senaryosunda eşleşti.
- Beş yeni senaryoda pozitif/negatif yön, karışık delta işaretleri, signed zero ve zincir sonuçları karşılaştırıldı.
- Birleşik altı aile corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `3854d607b2b0b2e782a153bffb5f46918565f2df40d4a855940d960959a37113`.

Birleşik referans kimliği: `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b`.

Yerel raporlar: `compare-cwwah065` (math debug), `compare-g3g2zbrw` (math release), `compare-5bjkz31s` (birleşik debug) ve `compare-zfkba6ts` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt yalnızca `AABB.contract` ve mevcut AABB adım adaptörüyle sınırlıdır. `inflate`, `intersect`, `minmax`, `distanceToSqr` ve `ofSize` ayrı kanıt belgelerindedir; [`collidedAlongVector`](aabb-collided-along-vector-conformance.md) ayrı dilimde tamamlandı ve VoxelShape clip tamamlandı; istemci hedefleme davranışları sonraki dilimlerdir.
