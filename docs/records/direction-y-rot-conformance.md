# P01.c — Direction y-rotation referans sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, Java Edition 26.2 / protokol 776 / Java 25 içindeki `Direction.fromYRot(double)` static yolunu ölçer.

## Kaynak ve sınır

Java `core/Direction.java:291-293` ve `util/Mth.java:66-68` incelendi. Y rotasyonu double olarak işlenir; sonuç `Mth.floor` ile Java i32 değerine daraltılır ve 2D Direction data yoluna aktarılır.

## Davranış sözleşmesi

- Hesap sırası `yRot / 90.0 + 0.5`, ardından `floor`, ardından i32 bit maskesi `& 3` şeklindedir.
- Maskelenen değer `from2DDataValue` içine girer; yatay yön sırası `0: south`, `1: west`, `2: north`, `3: east` olarak korunur.
- 45 derecelik eşikler floor öncesindeki `+ 0.5` nedeniyle sınırın kendisinde bir sonraki 2D data değerine geçer; negatif açıların floor ve mask sırası aynıdır.
- Signed zero, subnormal, sonsuzluk, NaN ve i32 daraltma sınırları doğrudan Java oracle çıktısından karşılaştırılır.

## JSONL arayüzü

Her istek `family: "math"`, `operation: "direction_y_rot"` ve IEEE-754 f64 girdisini 16 hexadecimal bit olarak taşıyan `value` alanlarını içerir. Çıktı yön snapshot'ıdır.

Yakalama ve doğrulama depo kökünden çalıştırılır:

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py verify --family math
```

Yeni capture kimliği, kaynak/girdi/ortam değişkenleriyle birlikte runner raporuna kaydedilir.

## Kapsam dışı

`fromAxisAndDirection`, nearest direction seçimleri, entity bakış sıralaması, random direction ve codec yolları bu dilime dahil değildir.


## Client-parity doğrulaması

`crates/photon-core/src/direction.rs` içindeki `from_y_rot`, Java `Direction.fromYRot(double)` hesabının bölme, `+ 0.5`, floor, `& 3` ve 2D lookup sırasını taşır. Rust adapter `direction_y_rot` JSONL operation'ı üzerinden bu yöntemi çağırır; Java oracle doğrudan `Direction.fromYRot` kullanır.

Senaryolar eşik açılarını, negatif değerleri, tekrar eden tam turları, signed zero/subnormal girdilerini ve özel double değerlerini kapsar. Beklenen yön snapshot'ları Java oracle çıktısından alındı.

Java math capture kimliği `a346f61e33f8502c48ca81dee6ee5c27df2c177dc9e95443316cafa4187378ba`, birleşik capture kimliği `ffca09f72f4f265ba069d418009ef267c2669772a52f3ea107cf2037d7a93194` olarak `.data/` altında, kaynak/girdi/ortam kimliğiyle tutulur.

Math adaptörüyle karşılaştırma:

```bash
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
```

Aynı capture için Java tekrar edilebilirliği ve kayıt bütünlüğü:

```bash
python3 tools/behavior/run.py verify --family math
```

Bu kanıt yalnızca `Direction.fromYRot` sonucunu kapsar; data decoder, axis-direction constructor ve entity bakış akışları ayrı davranış sözleşmeleridir.
