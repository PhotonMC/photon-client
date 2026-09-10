# P01.c — Direction axis-choose referans sözleşmesi

Bu dilim, Java Edition 26.2 / protokol 776 / Java 25 içindeki `Direction.Axis.choose` int, double ve boolean overload'larını ölçer.

## Kaynak ve sınır

Java `core/Direction.java:539-543` incelendi. Her overload seçilen enum eksenine karşılık gelen tek x, y veya z bileşenini döndürür.

## Davranış sözleşmesi

- X ekseni üç overload'da x girdisini seçer.
- Y ekseni üç overload'da y girdisini seçer.
- Z ekseni üç overload'da z girdisini seçer.
- int sonucu signed i32 olarak, double sonucu ham IEEE-754 bitleriyle, boolean sonucu doğrudan korunur.

## JSONL arayüzü

Her istek `family: "math"`, `operation: "direction_axis_choose"` ve `axis` alanını taşır. `int_x/y/z` signed i32 dizgeleri, `double_x/y/z` f64 bit dizgeleri ve `bool_x/y/z` boolean alanlarıdır. Çıktı seçilen `int`, `double` ve `boolean` değerlerini içerir.

Yakalama ve doğrulama depo kökünden çalıştırılır:

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py verify --family math
```

Yeni capture kimliği, kaynak/girdi/ortam değişkenleriyle birlikte runner raporuna kaydedilir.

## Kapsam dışı

Axis predicate'leri, yön eşlemeleri, plane, random seçim ve Direction yönlerinin diğer sorguları bu dilime dahil değildir.
