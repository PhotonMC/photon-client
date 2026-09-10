# P01.d — AABB distanceToSqr sözleşmesi

Bu dilimin **6 Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](aabb-distance-parity.md)). Kapsam, Java `AABB.distanceToSqr(Vec3)` ve `AABB.distanceToSqr(AABB)` overload'larının kutu dışındaki eksen boşluklarını kareli uzaklığa çevirmesini içerir.

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

DistanceToSqr senaryolarını içeren math referans kimliği `09c382b6287c06f691d0d698d1c32b5c89a3a02545ec723d11c894ffc006628c` oldu. Bu referans iki JVM çalıştırmasıyla kararlı bulundu. Rust debug ve release adaptörleri math ailesinde 424/424, birleşik corpus'ta 2225/2225 eşleşti; ayrıntılar [parity belgesindedir](aabb-distance-parity.md).

Ham girdiler ve Java çıktıları yalnızca `.data/` altında tutulur.
