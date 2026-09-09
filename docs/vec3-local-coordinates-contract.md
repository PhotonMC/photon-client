# P01.b — Vec3 yerel koordinat yardımcıları sözleşmesi

Bu dilimde **6 Java davranış testi hazırlanmıştır**. Kapsam, Java `Vec3.applyLocalCoordinatesToRotation` statik yardımcı metodu ile `addLocalCoordinates` instance metodunun yerel yönü dünya koordinatlarına çevirmesidir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/Vec3.java:327-349` ve trigonometrik bağımlılık olarak `util/Mth.java:29-55` incelendi. `addLocalCoordinates`, önce `this.rotation()` sonucunu üretip aynı statik yardımcıyı çağırır. Oracle doğrudan Java metodlarını çağırır; beklenen sonuçlar Rust çıktısından türetilmez.

## Davranış sözleşmesi

- `applyLocalCoordinatesToRotation`, pitch/yaw Vec2'sinden ileri, yukarı ve sol f64 Vec3 tabanlarını Java `Mth` sin/cos lookup sonuçlarıyla kurar.
- Açı dönüşümü ve taban vektörlerinin çarpımları Java'daki f32 ara değerleriyle yapılır; taban bileşenleri Vec3 constructor'ına geçerken f64'e genişletilir.
- Sol taban, ileri ve yukarı vektörlerinin cross sonucunun `-1.0` ile ölçeklenmesiyle elde edilir.
- Yerel yönün x/y/z bileşenleri sırasıyla sol, yukarı ve ileri tabanlarıyla f64 çarpılıp Java kaynak sırasıyla toplanır.
- `addLocalCoordinates`, mevcut vektörün `rotation()` sonucunu kullanır ve verilen yönü aynı dönüşümden geçirir.

## Senaryolar ve JSONL

| Senaryo | Girdi | Karşılaştırılan çıktı |
| --- | --- | --- |
| `apply-basic` | Sıfır pitch/yaw ve sonlu yön | Vec3 snapshot'ı |
| `apply-combined` | Birleşik pitch/yaw ve negatif/pozitif yön bileşenleri | Vec3 snapshot'ı |
| `apply-axis-turns` | 90/180 derece eksen dönüşleri | Vec3 snapshot'ı |
| `apply-signed-zero` | Signed-zero açı ve subnormal yön bileşenleri | Vec3 snapshot'ı |
| `add-axis` | Z ekseni yönelimi üzerinden instance çağrısı | Vec3 snapshot'ı |
| `add-combined` | Negatif/non-unit mevcut vektör ve birleşik yerel yön | Vec3 snapshot'ı |

Girdiler `family: "math"`, `operation: "vec3_local_coordinates"` ve `kind` alanını kullanır. `apply` için f32 bit dizgeli `rotation` Vec2'si ve f64 bit dizgeli `direction` Vec3'ü; `add` için f64 bit dizgeli `first` ve `direction` Vec3'leri kullanılır. Sonuç `value` içinde f64 `x`, `y`, `z`, `length` ve `length_squared` bitlerini taşır.

## Rust karşılığı

`src/math.rs` içindeki `Vec3::apply_local_coordinates_to_rotation` aynı f32 trigonometrik ara yolu ve f64 toplama sırasını taşır. `Vec3::add_local_coordinates`, `rotation()` sonucunu bu yardımcıya verir. `examples/behavior_adapter.rs` statik ve instance yollarını doğrudan bu API'lerden sorgular; `tests/behavior/java/MathOracle.java` aynı yolları Java sınıfına bağlar.
