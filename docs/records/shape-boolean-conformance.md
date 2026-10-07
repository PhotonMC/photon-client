# P01.d — VoxelShape BooleanOp join sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu sözleşmede **8 Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](#client-parity-doğrulaması)). Kapsam, `Shapes.join` ve `Shapes.joinIsNotEmpty` çağrılarında Java'nın kabul ettiği `BooleanOp.FALSE`, `OR`, `AND`, `ONLY_FIRST`, `ONLY_SECOND`, `NOT_SAME`, `FIRST` ve `SECOND` dallarını içerir. Box üretimi ve taşıma için [temel shape sözleşmesi](shape-conformance.md) geçerlidir. Üç veya daha fazla shape'in varargs OR birleşimi ayrı [varargs sözleşmesinde](shape-or-many-conformance.md) ele alınır. `join` ile `joinUnoptimized` coordinate plane farkı ayrı [sözleşmede](shape-join-unoptimized-conformance.md) ele alınır.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/shapes/BooleanOp.java:3-21`, `Shapes.java:127-232` ve `BitSetDiscreteVoxelShape.java:112-158` incelendi. `MathOracle` BooleanOp değerini Java API'sine geçirir; beklenen kutuları yeniden hesaplamaz.

## Davranış sözleşmesi

- `AND` yalnızca iki girdinin de dolu olduğu ortak hücreleri taşır.
- `ONLY_FIRST`, ilk girdinin ikinciyle paylaşmadığı hücreleri; `ONLY_SECOND` ikinci girdinin ilk girdinin paylaşmadığı hücreleri taşır.
- `NOT_SAME` iki girdiden yalnızca birinin dolu olduğu hücreleri taşır; `FIRST` ve `SECOND` sırasıyla ilk veya ikinci shape'in hücrelerini geçirir. `FALSE` her hücreyi boş bırakır; `OR` iki girdinin dolu hücrelerini birleştirir.
- Join sonrası komşu dolu hücreler Java `optimize`/`toAabbs` sırasına göre AABB listesine çıkarılır. Hiç dolu hücre kalmadığında snapshot `empty: true`, boş `boxes` listesi ve `join_is_not_empty: false` taşır.
- `op.apply(false, false)` sonucu true olan Java `BooleanOp` sabitleri `Shapes.join` tarafından geçersiz kabul edilir; bu sözleşmenin adapter girdisine dahil değildir.

## Senaryolar ve JSONL

| Grup | Sayı | Girdiler |
| --- | ---: | --- |
| AND | 2 | Örtüşen kutular ve ayrık kutular |
| ONLY_FIRST | 1 | İlk kutunun ortasından çıkarılan ikinci kutu |
| ONLY_SECOND | 1 | Ayrık kutularda ikinci taraf |
| FALSE | 1 | Örtüşen kutularda boş sonuç |
| NOT_SAME | 1 | Örtüşen kutularda iki dış parça |
| SECOND | 1 | Ayrık kutularda ikinci shape |
| FIRST | 1 | Ayrık kutularda ilk shape |

`family: "math"`, `operation: "shape"`, `kind: "join"` girdileridir. `op` alanı `or`, `and`, `only_first`, `only_second`, `false`, `not_same`, `second` veya `first` olur. Sonuç `shape` snapshot'ı ile `join_is_not_empty` alanını içerir.

## Java yakalama

Depo kökünden:

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py verify --family math
cargo build --locked --offline --example behavior_adapter
python3 tools/behavior/run.py compare --family math --adapter target/debug/examples/behavior_adapter
```

BooleanOp senaryolarını içeren math referans kimliği `a76b7eaed8646c9469410bc3c88b48be54654ae35d49dcea4d88d7e3ba8904b7` oldu. Bu referans iki JVM çalıştırmasıyla kararlı bulundu. Birleşik altı aile referansı `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b` kimliğiyle iki JVM'de doğrulandı.

Rust debug ve release adaptörleri math ailesinde 361/361, birleşik corpus'ta 2801/2801 eşleşti. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.


## Client-parity doğrulaması

`crates/voxel-core/src/shape.rs`, Java'nın `Shapes.join` tarafından kabul ettiği sekiz BooleanOp sonucunu koordinat gridindeki hücreler üzerinden üretir. `examples/behavior_adapter.rs` aynı `Shapes.join` ve `Shapes.joinIsNotEmpty` gözlemlerini JSONL'e taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- 361 senaryoluk math referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **361/361** math senaryosunda eşleşti.
- Sekiz shape BooleanOp senaryosunda `AND`, `ONLY_FIRST`, `ONLY_SECOND`, `FALSE`, `NOT_SAME`, `SECOND` ve `FIRST` sonuçları ile `joinIsNotEmpty` değeri karşılaştırıldı; OR temel slice içinde ayrıca doğrulandı.
- Birleşik altı aile corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `a76b7eaed8646c9469410bc3c88b48be54654ae35d49dcea4d88d7e3ba8904b7`.

Birleşik referans kimliği: `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b`.

Yerel raporlar: `compare-c_grkisa` (math debug), `compare-ajjamzol` (math release), `compare-5bjkz31s` (birleşik debug) ve `compare-zfkba6ts` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt sekiz sentetik ve box tabanlı BooleanOp senaryosuyla sınırlıdır. `Shapes.join` tarafından reddedilen BooleanOp sabitleri, tam index merger seçenekleri, collision/occlusion ve istemci dünyasındaki shape toplama sonraki dilimlerdir.
