# P01.c — Direction y-rotation encoder referans sözleşmesi

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
