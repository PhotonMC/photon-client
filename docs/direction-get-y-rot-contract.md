# P01.c — Direction get-y-rot referans sözleşmesi

Bu dilim, Java Edition 26.2 / protokol 776 / Java 25 içindeki `Direction.getYRot(Direction)` static yolunu ölçer.

## Kaynak ve sınır

Java `core/Direction.java:134-141` incelendi. Yöntem yalnızca yatay yönler için geçerli y rotasyonunu verir; dikey yönleri ayrı bir hata yoluna ayırır.

## Davranış sözleşmesi

- `north`, `south`, `west` ve `east` sırasıyla 180, 0, 90 ve -90 f32 değerlerini üretir.
- `down` ve `up`, yön adını içeren `No y-Rot for vertical axis: ...` mesajıyla `IllegalStateException` üretir.
- Başarılı sonuçlar Java float raw bitleriyle, hata sonuçları outcome ve mesajla karşılaştırılır.

## JSONL arayüzü

Her istek `family: "math"`, `operation: "direction_get_y_rot"` ve yön adı taşıyan `name` alanlarını içerir. Çıktı ya `outcome: "value"` ile sekiz hexadecimal f32 biti ya da `outcome: "error"` ile Java hata mesajıdır.

Yakalama ve doğrulama depo kökünden çalıştırılır:

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py verify --family math
```

Yeni capture kimliği, kaynak/girdi/ortam değişkenleriyle birlikte runner raporuna kaydedilir.

## Kapsam dışı

`Direction.toYRot`, `Direction.fromYRot`, entity bakış yönü ve diğer Direction açı yardımcıları ayrı davranış sözleşmeleridir.
