# P01.b — Vec3 eksen dönüşü sözleşmesi

Bu dilimde **18 Java davranış testi hazırlanmıştır**. Kapsam `Vec3.xRot`, `Vec3.yRot` ve `Vec3.zRot` yöntemlerinin tek bir f32 radyan girdisiyle yeni Vec3 üretmesidir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/Vec3.java:232-261` ve `util/Mth.java:29-55` incelendi. Oracle doğrudan Java yöntemlerini çağırır; beklenen sonuç Rust çıktısından türetilmez.

## Davranış sözleşmesi

- Radyan parametresi f32'dür ve `Mth.sin`/`Mth.cos` lookup tablosuna double olarak genişletilir. Tablo 16-bit açı quantization'ı ve f32 sonuçları korur.
- `xRot`, X bileşenini koruyup Y/Z düzleminde; `yRot`, Y bileşenini koruyup X/Z düzleminde; `zRot`, Z bileşenini koruyup X/Y düzleminde kaynak Java formülleriyle dönüş yapar.
- Sin/cos f32 değerleri Vec3 f64 bileşenleriyle işleme girdiğinde f64'e yükseltilir. Çarpma ve toplama/çıkarma sırası gözlenebilir.
- Signed zero, subnormal/sonlu sınırlar, Infinity, NaN ve NaN/Infinity radyanlarının lookup index davranışı sonuçta görünür.

## Senaryolar ve JSONL

Her girdi `family: "math"`, `operation: "vec3_rotate"`, `axis: "x"|"y"|"z"`, f64 bit dizgeli `first` Vec3 ve f32 bit dizgeli `radians` taşır. Sonuç `value` içinde Vec3 snapshot'ıdır. Temel, negatif açı, küçük quantized açı, çeyrek dönüş, signed zero, özel vektör ve özel açı yolları üç eksen arasında dağıtılmış 18 senaryo ile sınanır.

## Rust karşılığı

Rust karşılığı, Java `Mth` tablosunun üretim ve indexleme sözleşmesini `src/math.rs` içinde koruyup `Vec3` dönüş yöntemlerini aynı f32/f64 sınırında uygular. `examples/behavior_adapter.rs` sonucu taşır; `tests/behavior/java/MathOracle.java` Java `xRot`/`yRot`/`zRot` çağrısını yapar.
