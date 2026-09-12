# P01.d — AABB distanceToSqr sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilimin **6 Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](#client-parity-doğrulaması)). Kapsam, Java `AABB.distanceToSqr(Vec3)` ve `AABB.distanceToSqr(AABB)` overload'larının kutu dışındaki eksen boşluklarını kareli uzaklığa çevirmesini içerir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/AABB.java:419-430` ve üç bileşenli `util/Mth.java:714-716` içindeki `lengthSquared` davranışı incelendi. `MathOracle` her overload'u doğrudan çağırır; beklenen mesafeyi oracle içinde yeniden hesaplamaz.

## Davranış sözleşmesi

- Nokta overload'u her eksende `max(max(min - point, point - max), 0.0)` değerini seçer.
- AABB overload'u her eksende `max(max(this.min - other.max, other.min - this.max), 0.0)` değerini seçer.
- Eksen boşluğu kutunun içinde veya yüzünde sıfırdır; kutular örtüşse ya da yalnızca değse sonuç `0.0` olur.
- Üç eksen sonucu `x * x + y * y + z * z` sırasıyla `Mth.lengthSquared` üzerinden toplanır. IEEE-754 double bitleri, sonsuzluk ve NaN yayılımı gözlenebilir çıktının parçasıdır.
- Girdi kutuları constructor normalizasyonundan geçer; sorgular kutuyu değiştirmez.

## Senaryolar ve JSONL

| Grup | Sayı | Girdiler |
| --- | ---: | --- |
| Nokta içi ve sınır | 1 | İç nokta ile iki karşılıklı sınır noktası |
| Nokta dışı | 1 | Birden çok eksende dışarıda ve tek eksende dışarıda noktalar |
| Nokta özel değerleri | 1 | Pozitif/negatif sonsuzluk ve NaN |
| Kutu örtüşmesi/değmesi | 1 | Örtüşen ve yüzü değen kutular |
| Kutu ayrımı | 1 | Diyagonal ayrım ve negatif yöndeki ayrım |
| Normalizasyon/özel değer | 1 | Ters endpoint'li kutu ve NaN endpoint'li karşı kutu |

`family: "math"`, `operation: "aabb"` girdileridir. `steps` içinde `op: "distance_point"` için `point`, `op: "distance_box"` için `box` bulunur. Her sonuç `value` alanında f64 bit string taşır.

## Java yakalama

Depo kökünden:

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py verify --family math
cargo build --locked --offline --example behavior_adapter
python3 tools/behavior/run.py compare --family math --adapter target/debug/examples/behavior_adapter
```

DistanceToSqr senaryolarını içeren math referans kimliği `09c382b6287c06f691d0d698d1c32b5c89a3a02545ec723d11c894ffc006628c` oldu. Bu referans iki JVM çalıştırmasıyla kararlı bulundu. Rust debug ve release adaptörleri math ailesinde 424/424, birleşik corpus'ta 2801/2801 eşleşti; ayrıntılar [parity belgesindedir](#client-parity-doğrulaması).

Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.


## Client-parity doğrulaması

`src/aabb.rs`, Java `AABB.distanceToSqr(Vec3)` ve `AABB.distanceToSqr(AABB)` overload'larının eksen başına dış boşluk hesabını ve üç bileşenli kare toplamını uygular. `examples/behavior_adapter.rs` `distance_point` ve `distance_box` adımlarını JSONL math senaryolarına bağlar.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- 424 senaryoluk math referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **424/424** math senaryosunda eşleşti.
- Altı yeni senaryoda nokta içi/sınır/dışı sorguları, sonsuzluk ve NaN, örtüşen/değen kutular, diyagonal ayrım ve ters/NaN endpoint'ler karşılaştırıldı.
- Birleşik altı aile corpus'u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.
- `cargo fmt`, `cargo check --locked --offline --all-targets`, `cargo test --locked --offline --all-targets` ve Clippy kontrolleri geçti.

Math referans kimliği: `09c382b6287c06f691d0d698d1c32b5c89a3a02545ec723d11c894ffc006628c`.

Birleşik referans kimliği: `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b`.

Yerel raporlar: `compare-zq6owp5b` (math debug), `compare-yw81ny2k` (math release), `compare-5bjkz31s` (birleşik debug) ve `compare-zfkba6ts` (birleşik release) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Bu kanıt yalnızca `AABB.distanceToSqr` iki overload'ı ve mevcut AABB adım adaptörüyle sınırlıdır. [`collidedAlongVector`](aabb-collided-along-vector-conformance.md) ayrı dilimde tamamlandı; VoxelShape clip tamamlandı; istemci hedefleme davranışları sonraki dilimlerdir.
