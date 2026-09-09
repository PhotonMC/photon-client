# P01.b — Vec3 sabitleri sözleşmesi

Bu dilimde **4 Java davranış testi hazırlanmıştır**. Kapsam, Java `Vec3` sınıfının public static sabitlerinin üç f64 bileşenidir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/Vec3.java:37-40` incelendi. Oracle sabitleri doğrudan okur; beklenen sonuçlar Rust çıktısından türetilmez.

## Davranış sözleşmesi

- `ZERO`, üç pozitif sıfır bileşeninden oluşur.
- `X_AXIS`, `Y_AXIS` ve `Z_AXIS`, ilgili eksende `1.0` ve diğer iki eksende pozitif `0.0` taşır.
- Her sabit sorgusu bileşenleri ve mevcut Vec3 snapshot’ındaki uzunluk değerlerini ham f64 bitleriyle karşılaştırır.

## Senaryolar ve JSONL

| Senaryo | Girdi | Karşılaştırılan çıktı |
| --- | --- | --- |
| `zero` | `Vec3.ZERO` adı | Vec3 snapshot’ı |
| `x_axis` | `Vec3.X_AXIS` adı | Vec3 snapshot’ı |
| `y_axis` | `Vec3.Y_AXIS` adı | Vec3 snapshot’ı |
| `z_axis` | `Vec3.Z_AXIS` adı | Vec3 snapshot’ı |

Girdiler `family: "math"`, `operation: "vec3_constant"` ve `name` alanını kullanır. Sonuç `value` içinde f64 `x`, `y`, `z`, `length` ve `length_squared` bitlerini taşır.

## Rust karşılığı

`src/math.rs` içindeki `Vec3` associated constant’ları Java değerlerini taşır. `examples/behavior_adapter.rs` sabiti doğrudan Rust API’sinden sorgular; `tests/behavior/java/MathOracle.java` aynı adı doğrudan Java sabitine bağlar.
