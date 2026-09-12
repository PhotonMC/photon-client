# P01.d — AABB predicate overload sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, Java `AABB.contains(Vec3)`, `AABB.intersects(double...)`, `AABB.intersects(Vec3, Vec3)` ve `AABB.intersects(BlockPos)` overload'larını kapsar. Altı Java davranış testi hazırlandı ve Rust karşılığı [doğrulandı](#client-parity-doğrulaması).

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/AABB.java:241-263` içindeki dört overload ve bunların scalar yolları incelendi. `MathOracle` her overload'u doğrudan çağırır; beklenen predicate sonucunu oracle içinde yeniden hesaplamaz.

## Davranış sözleşmesi

- `contains(Vec3)` üç double bileşenini scalar contains kuralına iletir; alt sınır dahil, üst sınır hariçtir.
- `intersects(double...)` verilen altı endpoint'i normalize etmez ve üç eksende strict overlap karşılaştırması yapar; yüz teması kesişim değildir.
- `intersects(Vec3, Vec3)` her eksende `Math.min`/`Math.max` ile endpoint'leri normalize ettikten sonra strict overlap yoluna gider.
- `intersects(BlockPos)` BlockPos origin'i ile origin + 1 arasındaki unit block'u karşılaştırır. `+1` Java int işlemidir; i32 taşması double karşılaştırmasından önce gerçekleşir.
- NaN, sonsuzluk ve signed zero predicate sonuçlarının parçası olarak doğrudan Java karşılaştırma ve min/max semantiğinden geçer.

## Senaryolar ve JSONL

| Grup | Sayı | Girdiler |
| --- | ---: | --- |
| Temel overload'lar | 1 | Dört predicate overload'unu aynı kutuda çağırır |
| Vec3 contains sınırları | 1 | Alt sınır, üst sınır, NaN ve sonsuzluk |
| Ham koordinatlar | 1 | Örtüşme, yüz teması, ters endpoint ve NaN |
| Vec3 endpoint'leri | 1 | Ters endpoint, signed zero ve NaN normalizasyonu |
| BlockPos sınırları | 1 | Integer minimum/maksimum ve `+1` taşması |
| Özel değer zinciri | 1 | Overload'ları ardışık predicate sorgularında çağırır |

`family: "math"`, `operation: "aabb"` girdileridir. `steps` içinde `op: "contains_vec3"`, `intersects_coords`, `intersects_vec3` veya `intersects_block_pos` bulunur. Vec3 endpoint'leri f64 bit dizgeleri, ham koordinatlar f64 endpoint alanları, BlockPos koordinatları decimal i32 dizgeleridir. Her adım `hit` boolean snapshot'ı ekler.

## Java yakalama

Depo kökünden:

```bash
python3 tools/behavior/run.py capture --family math
```

Predicate overload senaryolarını içeren math referans kimliği `88113a66373c63d82668d5e2c26347bce90060d5e60c0b41a5a1ffc6127377a7` oldu. Bu referans iki JVM çalıştırmasıyla kararlı bulundu. Rust ve birleşik corpus kanıtı [parity belgesindedir](#client-parity-doğrulaması).

Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.


## Client-parity doğrulaması

`src/aabb.rs`, Java `AABB.contains(Vec3)`, `AABB.intersects(double...)`, `AABB.intersects(Vec3, Vec3)` ve `AABB.intersects(BlockPos)` yollarını uygular. `examples/behavior_adapter.rs` dört predicate adımını JSONL math senaryolarına bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- 448 senaryoluk math referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **448/448** math senaryosunda eşleşti.
- Altı yeni senaryoda Vec3 contains sınırları, ham endpoint karşılaştırması, Vec3 normalizasyonu, BlockPos `+1` overflow'u, signed zero, NaN ve sonsuzluk karşılaştırıldı.
- Birleşik altı aile corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `88113a66373c63d82668d5e2c26347bce90060d5e60c0b41a5a1ffc6127377a7`.

Birleşik referans kimliği: `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b`.

Yerel raporlar: `compare-rnhr_0c0` (math debug), `compare-0r8ijg22` (math release), `compare-5bjkz31s` (birleşik debug) ve `compare-zfkba6ts` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt yalnızca dört AABB predicate overload'u ve mevcut AABB adım adaptörüyle sınırlıdır. [`collidedAlongVector`](aabb-collided-along-vector-conformance.md) ayrı dilimde tamamlandı; Çoklu AABB clip sonraki dilimdir.
