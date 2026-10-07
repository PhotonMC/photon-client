# P01.d — VoxelShape clip sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim Java 26.2 `VoxelShape.clip(Vec3, Vec3, BlockPos)` davranışını kapsar. Boş shape, kısa segment, shape içinden başlayan ışın, BlockPos ofseti, kısmi kutu ve OR ile birleşmiş kutu için 6 özgün senaryo hazırlanmıştır. Rust sonucu [parite kanıtında](#client-parity-doğrulaması) Java çıktısıyla karşılaştırılmıştır.

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

Clip senaryolarını içeren math referans kimliği `2ccf77d4a335bda76510148b74301ed5ceb8dd2e85877a9e63abbff58b164c50`'dir. Bu kayıt iki ayrı JVM çalıştırmasıyla kararlı bulundu. Birleşik altı aile referansı `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b`'dir. Ham girdiler ve çıktılar yalnızca yerel `.data/` alanında tutulur.


## Client-parity doğrulaması

`crates/voxel-core/src/shape.rs`, Java `VoxelShape.clip` çağrısındaki empty/kısa segment kontrollerini, test noktasıyla dolu hücre hızlı yolunu ve AABB fallback'ini uygular. `crates/voxel-core/src/direction.rs` Java'nın float dot product yön seçimini taşır. `examples/behavior_adapter.rs` ve `tests/behavior/java/MathOracle.java` aynı JSONL sorgularını Rust ve Java tarafına bağlar. Davranış sınırları [clip sözleşmesinde](#sözleşme) kayıtlıdır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- 528 senaryoluk math referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri math ailesinde **528/528** eşleşti.
- Altı yeni clip senaryosu ve içindeki 7 yön sorgusu; empty ve kısa segment sonuçlarını, doğrudan inside hit yönlerini, BlockPos ofsetini, kısmi kutu fallback'ini ve OR shape fallback'ini karşılaştırdı.
- Birleşik altı aile corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `2ccf77d4a335bda76510148b74301ed5ceb8dd2e85877a9e63abbff58b164c50`.

Birleşik referans kimliği: `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b`.

Yerel raporlar: `compare-al__d8s2` (math debug), `compare-q__13d9v` (math release), `compare-5bjkz31s` (birleşik debug) ve `compare-zfkba6ts` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt `VoxelShape.clip` ve mevcut box/OR üretim yollarındaki hit sonuçlarıyla sınırlıdır. Occlusion, entity hedefleme, collider toplama, hit adaylarının dünya içindeki seçimi ve diğer shape factory/merger dalları sonraki dilimlerde doğrulanacaktır.
