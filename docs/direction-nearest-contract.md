# P01.c — Direction nearest referans sözleşmesi

Bu dilim, Java Edition 26.2 / protokol 776 / Java 25 içindeki `Direction.getNearest(int, int, int, Direction)` ve `getNearest(Vec3i, Direction)` yollarını ölçer.

## Kaynak ve sınır

Java `core/Direction.java:334-353` incelendi. Üç bileşenli overload eksen büyüklüklerini `Math.abs` ile hesaplar ve yalnızca bir eksen diğer iki eksenden strict büyükse yön döndürür; eşitlikte `orElse` değerini korur. `Vec3i` overload'u aynı bileşen yoluna forwarding yapar.

## Davranış sözleşmesi

- X baskın pozitif/negatif değerler `east`/`west`, Z baskın değerler `south`/`north`, Y baskın değerler `up`/`down` yönünü seçer.
- İki veya üç eksen eşitse sonuç nullable `orElse` değeridir; `none` fallback'inde Java oracle `value` alanını kaldırarak boş sonuç nesnesi üretir.
- Java `Math.abs(Integer.MIN_VALUE)` taşması korunur; bu değer mutlak büyüklük karşılaştırmasında pozitif bir baskın değer gibi davranmaz.
- Vec3i overload'unda BlockPos bileşenleri aynı forwarding sonucunu üretir.

## JSONL arayüzü

Her istek `family: "math"`, `operation: "direction_nearest"`, `kind` (`components` veya `vec3i`), signed i32 `x`, `y`, `z` ve `or_else` (`none` veya yön adı) alanlarını içerir. Çıktı yön snapshot'ı veya nullable fallback'te boş JSON nesnesidir.

Yakalama ve doğrulama depo kökünden çalıştırılır:

```bash
python3 tools/behavior/run.py capture --family math
python3 tools/behavior/run.py verify --family math
```

Yeni capture kimliği, kaynak/girdi/ortam değişkenleriyle birlikte runner raporuna kaydedilir.

## Kapsam dışı

`getApproximateNearest`, axis-step order, entity bakış sıralaması, random seçim, codec'ler ve Direction dönüşümleri bu dilime dahil değildir.
