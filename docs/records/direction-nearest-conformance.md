# P01.c — Direction nearest referans sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

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


## Client-parity doğrulaması

`crates/voxel-core/src/direction.rs` içindeki `get_nearest`, Java `Direction.getNearest` üç bileşenli overload'unun strict eksen karşılaştırmasını, işaret yönünü ve nullable fallback davranışını uygular. Rust adapter `direction_nearest` JSONL operation'ı ile hem bileşen hem Vec3i forwarding senaryolarını karşılaştırır; Java oracle iki overload'u doğrudan çağırır.

Senaryolar baskın eksenleri, eşit büyüklükleri, null fallback'i ve i32 minimum değerindeki Java mutlak değer davranışını kapsar. Beklenen yön snapshot'ları Java oracle çıktısından alındı.

Java math capture kimliği `1dfb7a76f46a9ea8ea2868e2cbbe435f134086ea822738929c5e6533081458fa`, birleşik capture kimliği `e8b519abb2579853c8aca622c3159adbd5f756f45e6ea05504029c364a49a25f` olarak `.data/` altında, kaynak/girdi/ortam kimliğiyle tutulur.

Math adaptörüyle karşılaştırma:

```bash
python3 tools/behavior/run.py compare --family math \
  --adapter target/debug/examples/behavior_adapter
```

Aynı capture için Java tekrar edilebilirliği ve kayıt bütünlüğü:

```bash
python3 tools/behavior/run.py verify --family math
```

Bu kanıt yalnızca `Direction.getNearest` overload'larının yön sonucunu kapsar; approximate-nearest ve entity bakış akışları ayrı davranış sözleşmeleridir.
