# P01.d — AABB sizes sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilimin **6 Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](#client-parity-doğrulaması)). Kapsam, Java `AABB.getXsize()`, `getYsize()`, `getZsize()` ve `getSize()` sorgularının normalize edilmiş kutu uçlarından kenar uzunluklarını ve üç uzunluğun aritmetik ortalamasını üretmesini içerir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/AABB.java:267-284` incelendi. `MathOracle` dört getter'ı doğrudan çağırır; beklenen ölçüleri yeniden hesaplamaz.

## Davranış sözleşmesi

- `getXsize`, `getYsize` ve `getZsize` sırasıyla `max - min` çıkarımını kendi eksenlerinde yapar.
- `getSize`, üç kenar uzunluğunu `((x + y) + z) / 3.0` sırasıyla toplar ve böler.
- Kutunun uçları kurucuda normalize edildiği için ters verilen sonlu uçlar ölçü sorgularında non-negative uzunluklar üretir.
- IEEE-754 double çıkarma, toplama ve bölme bitleri; signed zero ve büyük değerlerdeki yuvarlama gözlenebilir çıktının parçasıdır.
- Sorgu kutuyu değiştirmez; sonuçlar her adımda `events` içinde taşınır.

## Senaryolar ve JSONL

| Grup | Sayı | Girdiler |
| --- | ---: | --- |
| Temel ölçü | 2 | Birim ve dikdörtgen kutular |
| Kurucu sınırı | 2 | Ters uçların normalizasyonu ve sıfır uzunluk |
| Sayısal ayrıntı | 2 | Signed zero ve büyük toplam yuvarlaması |

`family: "math"`, `operation: "aabb"` girdileridir. `steps` içinde `op: "sizes"` bulunur. Her sonuç `x_size`, `y_size`, `z_size` ve `size` alanlarını f64 bit string olarak taşır.

## Java yakalama

Depo kökünden:

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py verify --family math
cargo build --locked --offline --example behavior_adapter
python3 tools/behavior/run.py compare --family math --adapter target/debug/examples/behavior_adapter
```

Sizes senaryolarını içeren math referans kimliği `576e0822a567519f05faf72cba233f0591fa0c7e07cf41b43f37b88eecd0b61d` oldu. Bu referans iki JVM çalıştırmasıyla kararlı bulundu. Birleşik altı aile referansı `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b` kimliğiyle iki JVM'de doğrulandı.

Rust debug ve release adaptörleri math ailesinde 400/400, birleşik corpus'ta 2801/2801 eşleşti. Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.


## Client-parity doğrulaması

`crates/voxel-core/src/aabb.rs`, Java `AABB.getXsize()`, `getYsize()`, `getZsize()` ve `getSize()` sorgularının çıkarma, toplama ve bölme sırasını uygular. `examples/behavior_adapter.rs` `sizes` adımını JSONL math senaryolarına bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- 400 senaryoluk math referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **400/400** math senaryosunda eşleşti.
- Altı yeni senaryoda birim/dikdörtgen ölçüleri, ters uç normalizasyonu, sıfır uzunluk, signed zero ve büyük toplam yuvarlaması karşılaştırıldı.
- Birleşik altı aile corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `576e0822a567519f05faf72cba233f0591fa0c7e07cf41b43f37b88eecd0b61d`.

Birleşik referans kimliği: `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b`.

Yerel raporlar: `compare-_41j11nv` (math debug), `compare-xbtyhre_` (math release), `compare-5bjkz31s` (birleşik debug) ve `compare-zfkba6ts` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt yalnızca dört AABB ölçü getter'ı ve mevcut AABB adım adaptörüyle sınırlıdır. BlockPos, Vec3 ve Vector3f move overload'ları [ayrı dilimde tamamlandı](aabb-move-overloads-conformance.md); [`collidedAlongVector`](aabb-collided-along-vector-conformance.md) ayrı dilimde tamamlandı ve VoxelShape clip tamamlandı; istemci hedefleme davranışları sonraki dilimlerdir.
