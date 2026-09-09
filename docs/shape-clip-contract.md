# P01.d — VoxelShape clip sözleşmesi

Bu dilim Java 26.2 `VoxelShape.clip(Vec3, Vec3, BlockPos)` davranışını kapsar. Boş shape, kısa segment, shape içinden başlayan ışın, BlockPos ofseti, kısmi kutu ve OR ile birleşmiş kutu için 6 özgün senaryo hazırlanmıştır. Rust sonucu [parite kanıtında](shape-clip-parity.md) Java çıktısıyla karşılaştırılmıştır.

## Kaynak ve sınır

Referans Java Edition 26.2, protokol 776 ve Java 25'tir. `VoxelShape.java:163-182`, `DiscreteVoxelShape.java:62-80`, `CubeVoxelShape.java:18-21`, `Direction.java:311-328` ve `BlockHitResult.java:6-65` incelendi. `MathOracle`, referans JAR'daki shape ve hit API'lerini doğrudan çağırır; beklenen sonuç Rust kodundan veya test içinde tekrar yazılmış Java algoritmasından türetilmez.

## Davranış sözleşmesi

- Empty shape doğrudan `null` döndürür.
- Segment farkı `to - from` olarak hesaplanır. Kare uzunluk `1.0E-7` değerinden küçükse hit aranmaz ve sonuç `null` olur. Eşik kesin olarak küçüktür; eşitlik ayrı bir davranış durumudur.
- Diğer durumlarda test noktası `from + 0.001 * (to - from)` olur. Bu noktanın her koordinatından BlockPos çıkarılır ve shape'in üç eksendeki voxel index'i aranır.
- Test noktası dolu bir hücreye denk gelirse doğrudan bir `BlockHitResult` üretilir. Konum test noktasıdır, block position verilen `BlockPos`'tur, `inside` true'dur, world-border hit false'dur ve hit türü BLOCK'tur.
- Doğrudan hit yönü, segment farkına en yüksek dot product veren yönün tersidir. Java bu seçimde girdileri float'a çevirir; yönler `DOWN`, `UP`, `NORTH`, `SOUTH`, `WEST`, `EAST` sırasıyla taranır, başlangıç dot değeri `Float.MIN_VALUE`'dur ve eşitlikte önceki yön korunur.
- Test noktası dolu hücrede değilse shape `toAabbs()` sonucuna çevrilir ve `AABB.clip(Iterable<AABB>, from, to, pos)` yolu kullanılır. Bu fallback hit'i `inside: false` taşır ve en yakın kutu hit sırasını korur.
- `clip_join` iki box'ı mevcut OR BooleanOp yoluyla birleştirir; clip sorgusu birleşmiş shape'in `toAabbs()` sırasını kullanır.

## Senaryolar ve JSONL

| Grup | Senaryo | Kapsam |
| --- | ---: | --- |
| empty | 1 | Empty shape'e segment gönderimi |
| short segment | 1 | Kare uzunluğu `1.0E-7` altında kalan segment |
| inside directions | 1 | Altı eksen yönü ve bir diagonal olmak üzere 7 sorgu; doğrudan hit, yön ve tie sırası |
| outside offset | 1 | BlockPos ofsetiyle AABB fallback ve konum dönüşümü |
| partial box | 1 | Dolu olmayan test hücresinden AABB fallback |
| joined fallback | 1 | OR shape'in box listesi üzerinden fallback |

Girdiler `family: "math"`, `operation: "shape"` ve `kind: "clip"` taşır. `shape` bir box, `queries` ise `from`, `to` ve `pos` alanlı sorgu listesidir. `kind: "clip_join"` aynı sorgu biçimini `first` ve `second` box'larının OR birleşiminde kullanır. Her sorgu `BlockHitResult` snapshot'ı veya `null` olarak `result.events` içine yazılır.

## Java yakalama

Depo kökünden:

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py verify --family math
cargo build --locked --offline --example behavior_adapter
python3 tools/behavior/run.py compare --family math --adapter target/debug/examples/behavior_adapter
```

Clip senaryolarını içeren math referans kimliği `2ccf77d4a335bda76510148b74301ed5ceb8dd2e85877a9e63abbff58b164c50`'dir. Bu kayıt iki ayrı JVM çalıştırmasıyla kararlı bulundu. Birleşik altı aile referansı `a2de3a61075984570c78972cf64abf406bbb6ee6d60136396680e14b82219664`'dir. Ham girdiler ve çıktılar yalnızca yerel `.data/` alanında tutulur.
