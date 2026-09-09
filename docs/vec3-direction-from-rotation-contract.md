# P01.b — Vec3 directionFromRotation sözleşmesi

Bu dilimde **18 Java davranış testi hazırlanmıştır**. Kapsam `Vec3.directionFromRotation(float, float)` ve `Vec3.directionFromRotation(Vec2)` overload'larının pitch/yaw derecelerinden yeni Vec3 üretmesidir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/Vec3.java:267-279`, `world/phys/Vec2.java:18-27` ve `util/Mth.java:29-55` incelendi. Oracle doğrudan Java overload'larını çağırır; beklenen sonuç Rust çıktısından türetilmez.

## Davranış sözleşmesi

- `float, float` overload'u pitch (`rotX`) ve yaw (`rotY`) değerlerini Java'nın f32 derece→radyan sabitiyle f32 ara işlemlerinde dönüştürür.
- `Vec2` overload'u `rotation.x` ve `rotation.y` f32 bileşenlerini aynı `float, float` yoluna verir.
- Dönüşümde `Mth.sin`/`Mth.cos` 16-bit lookup quantization'ı ve f32 sonuçları korunur. Son üç yön bileşeni f32 çarpımları olarak hesaplanır ve Vec3'e f64 olarak genişletilir.
- Sıfır, signed zero, küçük açı, 90/180 derece, büyük açı, NaN, Infinity ve f32 taşma sınırları lookup index ve çıktı bitleriyle gözlenebilir.

## Senaryolar ve JSONL

Her girdi `family: "math"`, `operation: "vec3_direction_from_rotation"` ve `kind: "components"` veya `"vec2"` taşır. İlk biçim f32 bit dizgeli `rot_x`/`rot_y`, ikinci biçim f32 bit dizgeli `rotation` Vec2 kullanır. Sonuç `value` içinde Vec3 snapshot'ıdır. Temel, cardinal pitch/yaw, birleşik ve sarılmış açılar, küçük quantized açı, signed zero ve özel açı yolları iki overload arasında dağıtılmış 18 senaryo ile sınanır.

## Rust karşılığı

Rust karşılığı `src/math.rs` içindeki iki `Vec3` yordamında açı dönüşümünü, Java f32 ara işlemlerini, Mth lookup yolunu ve f32 ürünlerini korur. `examples/behavior_adapter.rs` iki JSON biçimini üretim API'sine taşır; `tests/behavior/java/MathOracle.java` Java overload'larını doğrudan çağırır.
