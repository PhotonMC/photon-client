# P01.c — Direction y-rotation referans sözleşmesi

Bu dilim, Java Edition 26.2 / protokol 776 / Java 25 içindeki eksensiz `Direction.getClockWise()` ve `getCounterClockWise()` yollarını ölçer.

## Kaynak ve sınır

Java `core/Direction.java:195-253` incelendi. Bu yöntemler yalnızca yatay yönler için Y ekseninde dönüş tanımlar; dikey yönleri hata olarak bildirir.

## Davranış sözleşmesi

- Clockwise yatay çevrim NORTH→EAST→SOUTH→WEST→NORTH sırasını kullanır.
- Counter-clockwise çevrim NORTH→WEST→SOUTH→EAST→NORTH sırasını kullanır.
- DOWN ve UP için clockwise hatası `Unable to get Y-rotated facing of ...`, counter-clockwise hatası `Unable to get CCW facing of ...` mesajını taşır.
- Başarılı yön snapshot'ları ve iki hata sonucu Java oracle çıktısıyla karşılaştırılır.

## JSONL arayüzü

Her istek `family: "math"`, `operation: "direction_y_rotation"` ve yön adı taşıyan `name` alanını içerir. Çıktı `clockwise` ve `counter_clockwise` altında value veya error outcome'larını içerir.

Yakalama ve doğrulama depo kökünden çalıştırılır:

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py verify --family math
```

Yeni capture kimliği, kaynak/girdi/ortam değişkenleriyle birlikte runner raporuna kaydedilir.

## Kapsam dışı

Eksen parametreli rotation, Y rotasyon açısı encoder'ı, matrix rotation ve entity bakış yönü ayrı davranış sözleşmeleridir.
