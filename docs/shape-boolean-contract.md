# P01.d — VoxelShape BooleanOp join sözleşmesi

Bu sözleşmede **8 Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](shape-boolean-parity.md)). Kapsam, `Shapes.join` ve `Shapes.joinIsNotEmpty` çağrılarında Java'nın kabul ettiği `BooleanOp.FALSE`, `OR`, `AND`, `ONLY_FIRST`, `ONLY_SECOND`, `NOT_SAME`, `FIRST` ve `SECOND` dallarını içerir. Box üretimi ve taşıma için [temel shape sözleşmesi](shape-contract.md) geçerlidir.

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

BooleanOp senaryolarını içeren math referans kimliği `a76b7eaed8646c9469410bc3c88b48be54654ae35d49dcea4d88d7e3ba8904b7` oldu. Bu referans iki JVM çalıştırmasıyla kararlı bulundu. Birleşik altı aile referansı `de3c26ebb4097aa764d25839e62f20bee934c42bb19318718b2b46242990a108` kimliğiyle iki JVM'de doğrulandı.

Rust debug ve release adaptörleri math ailesinde 353/353, birleşik corpus'ta 807/807 eşleşti. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.
