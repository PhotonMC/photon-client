# P01.c — Direction axis-direction referans sözleşmesi

Bu dilim, Java Edition 26.2 / protokol 776 / Java 25 içindeki `Direction.fromAxisAndDirection(Axis, AxisDirection)` static yolunu ölçer.

## Kaynak ve sınır

Java `core/Direction.java:295-300` incelendi. Yöntem eksen ve eksen yönü enum çiftini doğrudan altı kardinal yöne çevirir.

## Davranış sözleşmesi

- X pozitif `east`, X negatif `west` yönünü seçer.
- Y pozitif `up`, Y negatif `down` yönünü seçer.
- Z pozitif `south`, Z negatif `north` yönünü seçer.
- Her sonuç, yönün adını, data değerlerini, eksenini, karşıtını ve step bileşenlerini taşıyan enum snapshot'ı olarak gözlenir.

## JSONL arayüzü

Her istek `family: "math"`, `operation: "direction_axis"`, `axis` (`x`, `y`, `z`) ve `direction` (`positive`, `negative`) alanlarını içerir. Çıktı yön snapshot'ıdır.

Yakalama ve doğrulama depo kökünden çalıştırılır:

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py verify --family math
```

Yeni capture kimliği, kaynak/girdi/ortam değişkenleriyle birlikte runner raporuna kaydedilir.

## Kapsam dışı

Yönün yaklaşıksal seçimi, entity bakış yönleri, random seçim, codec'ler ve quaternion dönüşümleri bu dilime dahil değildir.
