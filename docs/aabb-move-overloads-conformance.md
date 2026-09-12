# P01.d — AABB move overload sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, Java `AABB.move(BlockPos)`, `AABB.move(Vec3)` ve `AABB.move(Vector3f)` overload'larının kutuyu taşımasını kapsar. Altı Java davranış testi hazırlandı ve Rust karşılığı [doğrulandı](#client-parity-doğrulaması).

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/AABB.java:223-239` içindeki üç overload ve scalar `move` çağrısı incelendi. `BlockPos` bileşenlerinin i32, `Vec3` bileşenlerinin f64, `Vector3f` bileşenlerinin f32 olduğu sabitlendi. `MathOracle` her overload'u doğrudan çağırır; beklenen uçları oracle içinde yeniden hesaplamaz.

## Davranış sözleşmesi

- `BlockPos` overload'u üç tamsayı koordinatı Java sayısal genişletmesiyle double toplama ifadelerine taşır.
- `Vec3` overload'u üç double bileşenini scalar `move` yoluna iletir.
- `Vector3f` overload'u üç float bileşenini double toplama öncesinde Java'nın widening dönüşümüyle taşır; f32 bitleri bu nedenle girdinin parçasıdır.
- Her endpoint aynı eksendeki miktarla toplanır ve sonuç yeni `AABB` kurucusundan geçer.
- Signed zero, sonsuzluk, NaN ve büyük değerlerde oluşan IEEE-754 double bitleri gözlenebilir çıktının parçasıdır.
- Overload çağrıları zincirlenebilir; her sonuç sonraki çağrının kutusudur.

## Senaryolar ve JSONL

| Grup | Sayı | Girdiler |
| --- | ---: | --- |
| Temel zincir | 1 | BlockPos, Vec3 ve Vector3f overload'larını sırayla çağırır |
| BlockPos sınırları | 1 | i32 minimum/maksimum koordinatları |
| Vec3 özel değerleri | 1 | Sonsuzluk, NaN ve normal double bileşenleri |
| Vector3f hassasiyeti | 1 | f32 olarak kodlanan kesirli ve büyük bileşenler |
| Signed zero | 1 | Üç overload'da signed zero ve sıfır BlockPos |
| Karışık zincir | 1 | Overload'ları farklı sırada ardışık çağırır |

`family: "math"`, `operation: "aabb"` girdileridir. `steps` içinde `op: "move_block_pos"`, `move_vec3` veya `move_vector3f` bulunur. `position` nesnesi BlockPos için decimal i32 dizgeleri, Vec3 için f64 ve Vector3f için f32 bit dizgeleri taşır. Her adım kutu snapshot'ı ekler.

## Java yakalama

Depo kökünden:

```bash
python3 tools/behavior/run.py capture --family math
```

Overload senaryolarını içeren math referans kimliği `e7ae22f078eb9523961cde345ba3ea72bd43b52904e0c750f321f6b4663fd1cd` oldu. Bu referans iki JVM çalıştırmasıyla kararlı bulundu. Rust ve birleşik corpus kanıtı [parity belgesindedir](#client-parity-doğrulaması).

Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.


## Client-parity doğrulaması

`src/aabb.rs`, Java `AABB.move(BlockPos)`, `AABB.move(Vec3)` ve `AABB.move(Vector3f)` yollarını sırasıyla integer, double ve float bileşenleriyle taşıyıp ortak scalar move hesabına bağlar. `examples/behavior_adapter.rs` üç overload adımını JSONL math senaryolarına bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- 442 senaryoluk math referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **442/442** math senaryosunda eşleşti.
- Altı yeni senaryoda BlockPos i32 sınırları, Vec3 özel değerleri, Vector3f f32 hassasiyeti, signed zero ve overload zincirleri karşılaştırıldı.
- Birleşik altı aile corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `e7ae22f078eb9523961cde345ba3ea72bd43b52904e0c750f321f6b4663fd1cd`.

Birleşik referans kimliği: `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b`.

Yerel raporlar: `compare-4ymeo58v` (math debug), `compare-x3p8pg_j` (math release), `compare-5bjkz31s` (birleşik debug) ve `compare-zfkba6ts` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt yalnızca `AABB.move(BlockPos)`, `AABB.move(Vec3)`, `AABB.move(Vector3f)` ve mevcut AABB adım adaptörüyle sınırlıdır. [`collidedAlongVector`](aabb-collided-along-vector-conformance.md) ayrı dilimde tamamlandı; Çoklu AABB clip sonraki dilimdir.
