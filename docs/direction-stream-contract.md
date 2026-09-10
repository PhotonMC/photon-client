# P01.c — Direction stream referans sözleşmesi

Bu dilim, Java Edition 26.2 / protokol 776 / Java 25 içindeki `Direction.stream()` static yolunu ölçer.

## Kaynak ve sınır

Java `core/Direction.java:130-132` incelendi. Yöntem statik Direction değerleri dizisi üzerinden stream oluşturur.

## Davranış sözleşmesi

- Çıktı enum bildirim sırasını korur: down, up, north, south, west, east.
- Her çağrı aynı statik değer sırasını gözlemler; değerler kopyalanıp değiştirilmez.
- Rust iterator çıktısı Java stream eleman adlarıyla karşılaştırılır.

## JSONL arayüzü

İstek `family: "math"` ve `operation: "direction_stream"` alanlarını taşır. Çıktı sıralı `faces` ad dizisini içerir.

Yakalama ve doğrulama depo kökünden çalıştırılır:

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py verify --family math
```

Yeni capture kimliği, kaynak/girdi/ortam değişkenleriyle birlikte runner raporuna kaydedilir.

## Kapsam dışı

`Direction.allShuffled`, Plane iterator/stream yolları ve diğer Direction yardımcıları ayrı davranış sözleşmeleridir.
