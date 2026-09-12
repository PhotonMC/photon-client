# P01.c — Direction y-rotation referans sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

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


## Client-parity doğrulaması

`src/direction.rs` içindeki `Direction::clockwise_y` ve `counter_clockwise_y`, Java eksensiz Y dönüş tablolarını ve dikey hata yollarını uygular. Rust adapter `direction_y_rotation` JSONL operation'ı ile iki sonucu dışarı verir; Java oracle iki enum metodunu doğrudan çağırır.

Altı yön girdisi, yatay dönüşler ve dikey hata mesajlarıyla Java oracle çıktısına karşılaştırılır.

Java math capture kimliği `ce6d3c2901a0c4eeb11a92ff58b6267db8d84bcdd4706068f6a906caab0e6d62`, birleşik capture kimliği `b9fe38e6188e7c1423ff30e78934b6daa161a516e44f8566db2d409d01148bae` olarak `.data/` altında, kaynak/girdi/ortam kimliğiyle tutulur.

Math adaptörüyle karşılaştırma:

```bash
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
```

Aynı capture için Java tekrar edilebilirliği ve kayıt bütünlüğü:

```bash
python3 tools/behavior/run.py verify --family math
```

Bu kanıt yalnızca eksensiz `getClockWise` ve `getCounterClockWise` yollarını kapsar; eksen parametreli dönüşler ayrı davranış sözleşmesidir.
