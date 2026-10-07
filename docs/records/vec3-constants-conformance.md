# P01.b — Vec3 sabitleri sözleşmesi — [client-parity] birleşik kayıt

Bu dosya, aynı davranış diliminin sözleşmesini ve doğrulama kaydını birlikte tutar.

## Sözleşme

Bu dilimde **4 Java davranış testi hazırlanmıştır**. Kapsam, Java `Vec3` sınıfının public static sabitlerinin üç f64 bileşenidir.

## Kaynak ve sınır

Java Edition 26.2 / protokol 776 / Java 25 referansı kullanılır. `world/phys/Vec3.java:37-40` incelendi. Oracle sabitleri doğrudan okur; beklenen sonuçlar Rust çıktısından türetilmez.

## Davranış sözleşmesi

- `ZERO`, üç pozitif sıfır bileşeninden oluşur.
- `X_AXIS`, `Y_AXIS` ve `Z_AXIS`, ilgili eksende `1.0` ve diğer iki eksende pozitif `0.0` taşır.
- Her sabit sorgusu bileşenleri ve mevcut Vec3 snapshot’ındaki uzunluk değerlerini ham f64 bitleriyle karşılaştırır.

## Senaryolar ve JSONL

| Senaryo | Girdi | Karşılaştırılan çıktı |
| --- | --- | --- |
| `zero` | `Vec3.ZERO` adı | Vec3 snapshot’ı |
| `x_axis` | `Vec3.X_AXIS` adı | Vec3 snapshot’ı |
| `y_axis` | `Vec3.Y_AXIS` adı | Vec3 snapshot’ı |
| `z_axis` | `Vec3.Z_AXIS` adı | Vec3 snapshot’ı |

Girdiler `family: "math"`, `operation: "vec3_constant"` ve `name` alanını kullanır. Sonuç `value` içinde f64 `x`, `y`, `z`, `length` ve `length_squared` bitlerini taşır.

## Rust karşılığı

`crates/voxel-core/src/math.rs` içindeki `Vec3` associated constant’ları Java değerlerini taşır. `examples/behavior_adapter.rs` sabiti doğrudan Rust API’sinden sorgular; `tests/behavior/java/MathOracle.java` aynı adı doğrudan Java sabitine bağlar.


## Client-parity doğrulaması

`crates/voxel-core/src/math.rs`, Java `Vec3.ZERO`, `X_AXIS`, `Y_AXIS` ve `Z_AXIS` sabitlerini associated constant olarak sunar. Java oracle sabitleri doğrudan okur; Rust adaptörü aynı değerleri üretim API’sinden taşır.

## Doğrulama

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Math ailesinin 2299 senaryoluk referansı iki ayrı JVM çalıştırmasıyla kararlı bulundu.
- Rust debug ve release adaptörleri **2299/2299** math senaryosunda eşleşti.
- Dört yeni senaryoda `ZERO` ve üç pozitif birim eksen sabitinin f64 bileşenleri karşılaştırıldı.
- Birleşik altı aile corpus’u **2801/2801** senaryoda hem debug hem release adaptörüyle eşleşti.
- `verify` çalıştırması kayıtlı 2801 senaryoluk Java referansını yeniden doğruladı.

Math referansı `9421a309da79905eddc1ce85361a8baef1e5c3a592c7326ac123cb13c20a6508`, birleşik referans `8c4789d8c4953d1e6851df238f157e9ea125bc1a32a2eb87d50f55beeda3476b` kimliklerindedir.

Yerel raporlar: `compare-00tgiqc4` (math debug), `compare-aylr4ozm` (math release), `compare-5bjkz31s` (birleşik debug), `compare-zfkba6ts` (birleşik release) ve `verify-mgwg6r2l` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Sınırlar

Kanıt dört özgün sabit sorgusu ve Vec3 snapshot sonuçlarıyla sınırlıdır. Sabitlerin oyun davranışındaki tüm tüketicileri sonraki istemci dilimleridir.
