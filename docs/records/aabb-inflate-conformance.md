# P01.d — AABB inflate sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilimin **5 Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](#client-parity-doğrulaması)). Kapsam, Java `AABB.inflate` üç eksenli ve tek miktarlı overload'larının her iki endpoint'i delta kadar dışarı taşımasını içerir. AABB contract, deflate üç eksenli overload, intersect, minmax, move, sizes, hasNaN, positions, axis min/max, distanceToSqr ve ofSize için ayrı sözleşmeler, clip ve diğer kesişim işlemleri ayrı dilimlerdir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/AABB.java:189-201` incelendi. `MathOracle` doğrudan `AABB.inflate` overload'larını çağırır; beklenen uçları yeniden hesaplamaz.

## Davranış sözleşmesi

- Üç eksenli overload'da her eksenin min endpoint'inden delta çıkarılır, max endpoint'ine delta eklenir.
- Scalar overload aynı miktarı üç eksene verir. Negatif miktar kutuyu daraltır; sonuç Java `AABB` kurucusunun endpoint normalizasyonundan geçer.
- Sıfır ve signed zero miktarları endpoint'leri değiştirmez. Eksenler bağımsız işlenir.
- Zincir adımları bir önceki `AABB` sonucunu sonraki overload çağrısına verir.

## Senaryolar ve JSONL

| Grup | Sayı | Girdiler |
| --- | ---: | --- |
| Scalar overload | 2 | Pozitif ve negatif tek miktar |
| Üç eksenli overload | 1 | Karışık işaretli x/y/z delta'ları |
| Sıfır/zincir | 2 | Signed zero ve ardışık overload çağrıları |

`family: "math"`, `operation: "aabb"` girdileridir. `steps` içinde `op: "inflate"` scalar `amount` veya `op: "inflate_xyz"` f64 bit string `x`, `y`, `z` delta'ları bulunur. Her adım sonrası kutu uçları f64 bit string olarak `events` içinde taşınır.

## Java yakalama

Depo kökünden:

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py verify --family math
cargo build --locked --offline --example behavior_adapter
python3 tools/behavior/run.py compare --family math --adapter target/debug/examples/behavior_adapter
```

Inflate senaryolarını içeren math referans kimliği `cc1e4e363fee8a21012ea0d56765f85c05556ced56a4bd8c330f603a71ce5656` oldu. Bu referans iki JVM çalıştırmasıyla kararlı bulundu. Birleşik altı aile referansı `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b` kimliğiyle iki JVM'de doğrulandı.

Rust debug ve release adaptörleri math ailesinde 376/376, birleşik corpus'ta 2801/2801 eşleşti. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.


## Client-parity doğrulaması

`crates/photon-core/src/aabb.rs`, Java `AABB.inflate` overload'larının endpoint hesaplarını ve scalar overload yönlendirmesini uygular. `examples/behavior_adapter.rs` iki inflate adımını JSONL math senaryolarına bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- 376 senaryoluk math referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **376/376** math senaryosunda eşleşti.
- Beş yeni senaryoda scalar pozitif/negatif miktar, üç eksenli karışık delta, signed zero ve zincir sonuçları karşılaştırıldı.
- Birleşik altı aile corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `cc1e4e363fee8a21012ea0d56765f85c05556ced56a4bd8c330f603a71ce5656`.

Birleşik referans kimliği: `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b`.

Yerel raporlar: `compare-7jscyfxl` (math debug), `compare-0cnexscj` (math release), `compare-5bjkz31s` (birleşik debug) ve `compare-zfkba6ts` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt yalnızca `AABB.inflate` overload'ları ve mevcut AABB adım adaptörüyle sınırlıdır. `distanceToSqr` ayrı dilimde tamamlandı; [`collidedAlongVector`](aabb-collided-along-vector-conformance.md) da ayrı dilimde tamamlandı, VoxelShape clip tamamlandı; istemci hedefleme davranışları sonraki dilimlerdir.
