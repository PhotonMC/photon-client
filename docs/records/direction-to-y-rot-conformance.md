# P01.c — Direction y-rotation encoder referans sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilim, Java Edition 26.2 / protokol 776 / Java 25 içindeki `Direction.toYRot()` instance yolunu ölçer.

## Kaynak ve sınır

Java `core/Direction.java:303-305` incelendi. Yöntem yönün data2d değerini maskeler ve 90 derecelik float açısına çevirir.

## Davranış sözleşmesi

- Hesap sırası `(data2d & 3) * 90` şeklindedir; maske ve çarpma i32 olarak tamamlanır, sonuç f32 olarak döner.
- `south` 0, `west` 90, `north` 180 değerlerini üretir.
- `east`, `down` ve `up` yönleri 270 derece üretir; dikey yönlerdeki `data2d = -1` değeri maskelenince 3 olur.
- Çıktı, Java float raw bitleriyle karşılaştırılır.

## JSONL arayüzü

Her istek `family: "math"`, `operation: "direction_to_y_rot"` ve yön adı taşıyan `name` alanlarını içerir. Çıktı sekiz hexadecimal f32 bitidir.

Yakalama ve doğrulama depo kökünden çalıştırılır:

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py verify --family math
```

Yeni capture kimliği, kaynak/girdi/ortam değişkenleriyle birlikte runner raporuna kaydedilir.

## Kapsam dışı

Y yönünden Direction seçimi, entity bakış yönleri, random seçim, codec'ler ve diğer açı yardımcıları bu dilime dahil değildir.


## Client-parity doğrulaması

`crates/voxel-core/src/direction.rs` içindeki `to_y_rot`, Java `Direction.toYRot()` data2d maskesi ve 90 derecelik f32 dönüşümünü uygular. Rust adapter `direction_to_y_rot` JSONL operation'ı üzerinden sonucu raw bitlerle dışarı verir; Java oracle doğrudan enum metodunu çağırır.

Altı yön adı test edildi. Beklenen f32 bitleri Java oracle çıktısından alındı.

Java math capture kimliği `6333b31ddd0c9e44a85eb6b8008b8eac82df5edff51920f8c6c32da5d9b690b4`, birleşik capture kimliği `3db4b23f685b322fabe26e2db418bfb98be927e352d42d87c2953a34a5c3688b` olarak `.data/` altında, kaynak/girdi/ortam kimliğiyle tutulur.

Math adaptörüyle karşılaştırma:

```bash
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
```

Aynı capture için Java tekrar edilebilirliği ve kayıt bütünlüğü:

```bash
python3 tools/behavior/run.py verify --family math
```

Bu kanıt yalnızca `Direction.toYRot` sonucunu kapsar; `fromYRot` seçimi ve diğer Direction dönüşümleri ayrı davranış sözleşmeleridir.
