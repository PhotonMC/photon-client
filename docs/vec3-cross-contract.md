# P01.b — Vec3 cross sözleşmesi

Bu dilimde **5 Java davranış testi hazırdır ve Rust karşılığı doğrulanmıştır** ([kanıt](vec3-cross-parity.md)). Kapsam, Java `Vec3.cross(Vec3)` metodunun yeni bir vektör üretmesini içerir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/Vec3.java:91-93` içindeki `cross(Vec3)` incelendi. Oracle doğrudan Java çağrısını yapar; beklenen sonuç Rust çıktısından türetilmez.

## Davranış sözleşmesi

- `cross(Vec3)`, iki Vec3'ün üç determinant bileşenini üretir ve sonucu yeni bir Vec3 olarak döndürür.
- Her determinant bileşeni iki çarpımın çıkarılmasıyla hesaplanır; x, y ve z bileşenleri kaynakta görülen sırayı korur.
- Girdi vektörleri değişmez; ardışık step çağrılarında bir önceki sonuç sonraki `cross` çağrısının ilk operandıdır.
- Signed zero, NaN, Infinity ve double taşması sonuçları ham IEEE-754 bitleriyle karşılaştırılır; tolerans veya NaN normalizasyonu uygulanmaz.
- Sonuç snapshot'ı her adımda bileşen, uzunluk ve uzunluk karesi bitlerini taşır; türetilmiş ölçümlerdeki taşma ve özel değerler de kapsam içindedir.

## Senaryolar ve JSONL

| Senaryo | Girdi | Karşılaştırılan çıktı |
| --- | --- | --- |
| `basic` | Sonlu, farklı bileşenli iki Vec3 | Cross vektörü ve türetilmiş ölçümler |
| `chain` | İki cross ve sonraki normalize adımı | Sıralı vektör snapshot'ları |
| `signed-zero` | İşaretli sıfır bileşenleri | Bileşen bitleri |
| `overflow` | En büyük sonlu double ile çarpım | Infinity ve vektör bitleri |
| `special-values` | Infinity ve sıfır çarpımı | NaN/Infinity bitleri |

Girdiler `family: "math"`, `operation: "vec3"` ve `steps` içinde `cross` kullanır. İkinci vektör `other` nesnesinde, bileşenler 16 haneli küçük harfli f64 bit string'leri olarak taşınır.

## Rust karşılığı

`src/math.rs` içindeki `Vec3::cross`, üç determinant bileşenini doğrudan kaynak operand sırasıyla hesaplar. `examples/behavior_adapter.rs` bu metodu JSONL `cross` adımına bağlar; `tests/behavior/java/MathOracle.java` beklenen sonucu doğrudan Java `cross` çağrısından alır.
