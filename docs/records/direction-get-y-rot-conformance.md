# P01.c — Direction get-y-rot referans sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

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


## Client-parity doğrulaması

`crates/photon-core/src/direction.rs` içindeki `Direction::get_y_rot`, Java `Direction.getYRot(Direction)` yatay sabitlerini ve dikey hata yolunu uygular. Rust adapter `direction_get_y_rot` JSONL operation'ı üzerinden başarıda raw f32 bitlerini, hatada Java mesajını dışarı verir; Java oracle static enum metodunu doğrudan çağırır.

Altı yön adı test edildi; beklenen başarı ve hata sonuçları Java oracle çıktısından alındı.

Java math capture kimliği `42462ab197598295116b848be00dc9cb225cc5825dfc9b81fb9db149f7aa9fde`, birleşik capture kimliği `786e7ede06ae88d1ca9909909c689f4a36314a667ae7e86c99d700160cb96022` olarak `.data/` altında, kaynak/girdi/ortam kimliğiyle tutulur.

Math adaptörüyle karşılaştırma:

```bash
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
```

Aynı capture için Java tekrar edilebilirliği ve kayıt bütünlüğü:

```bash
python3 tools/behavior/run.py verify --family math
```

Bu kanıt yalnızca `Direction.getYRot` yolunu kapsar; diğer Direction açı ve entity bakış yolları ayrı davranış sözleşmeleridir.
