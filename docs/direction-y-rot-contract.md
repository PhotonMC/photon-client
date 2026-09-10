# P01.c — Direction y-rotation referans sözleşmesi

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
