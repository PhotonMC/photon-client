# P01.a — Vec2 ve sayısal yardımcılar

`src/math.rs`, mevcut Java senaryolarının kapsadığı Vec2 ve f32/f64 yardımcılarını içerir. Rust JSONL adaptörü bu modülü çağırır; beklenen değer veya oyun algoritması adaptör içinde üretilmez. Yeni bağımlılık eklenmedi.

## Referans ve davranış

Kaynak köküne göre `net/minecraft/world/phys/Vec2.java` ve `net/minecraft/util/Mth.java` yeniden incelendi. Referans Java 26.2 client JAR'ı; SHA-1 `2dc72797acbc1b63fc16a11c4ac393605f453754`. Mevcut `MathOracle.java` doğrudan orijinal sınıfları çağırır. Rust implementasyonu için senaryolar, Java adaptörü ve beklenen çıktılar değiştirilmedi.

| Rust API | Korunan davranış |
| --- | --- |
| `Vec2::new`, `ZERO` | f32 bileşenler; sıfır vektör pozitif sıfır bileşenleri taşır |
| `Vec2::scale` | Her bileşen f32 hassasiyetinde çarpılır |
| `Vec2::length_squared` | Ayrı çarpımlar ve toplam; ara f32 taşması/yuvarlaması korunur |
| `Vec2::length` | f32 kareler toplamı f64 karekökten sonra f32'ye döner |
| `Vec2::normalized` | Uzunluk 0,0001'den küçükse sıfır; diğer durumda her bileşen uzunluğa ayrı bölünür |
| `floor_f32`, `floor_f64` | Aşağı yuvarlama ve i32 dönüşümü; sınır dışı sonuçlar doyar, NaN sıfıra dönüşür |
| `wrap_degrees_f32`, `wrap_degrees_f64` | İşaretli kalan üzerinden [-180, 180) aralığı; negatif sıfır kaybolmaz |
| `clamp_f32`, `clamp_f64` | Referansın alt sınır karşılaştırması ve üst sınır minimumu; NaN/sıfır davranışı Rust'ın genel yardımcılarına bırakılmaz |

`hypot`, çarpma-toplama birleştirmesi veya ortak ters uzunlukla çarpma kullanılmadı; matematiksel olarak benzer yazımlar ara yuvarlama veya taşmayı değiştirebilir. İki hassasiyetteki scalar işlevleri küçük bir özel makroyla üretiliyor; genel matematik trait sistemi veya haricî vektör kütüphanesi eklenmedi.

Bu API, Vec2 sınıfının tamamı değildir: dot/add/rotation, Java nesne eşitliği/hash'i ve resource codec'i kapsam dışındaki sonraki ihtiyaçlardır. Mth'nin int/long overload'ları, trigonometrik tablolar ve diğer yardımcıları bu dilimde uygulanmadı.

## Doğrulama

Mevcut 180 matematik senaryosu; sıfır/işaretli sıfır, eksen ve diyagonal vektörler, normalize eşiğinin komşuları, ölçek-normalize zincirleri, büyük/küçük sayılar, f32/f64 sınır komşuları, sonsuzluk ve NaN girdilerini kapsıyor. Clamp sınırları mevcut corpus'ta [-1, 1] ve [0, 1]; ters/NaN sınırlar, bütün NaN payload'ları ve tüm float bit desenleri test edilmiş sayılmaz.

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Kayıtlı 346 senaryoluk Java referansı iki yeni JVM çalıştırmasıyla doğrulandı.
- Debug ve release Rust adaptörleri **346/346** eşleşti: 180 math + 22 timer + 144 wire.
- Vec2'de başlangıç ve her adımın x/y, uzunluk ve uzunluk karesi bitleri; scalar işlevlerde sonuç bitleri veya tamsayı değeri karşılaştırıldı. Float toleransı veya NaN normalizasyonu eklenmedi.
- Yanlış f64 bit formatı, bilinmeyen tür/işlem/aile, eksik clamp sınırı ve bilinmeyen Vec2 adımı reddedildi.
- Biçim, Clippy ve paket kapsamı kontrolleri geçti; `.data/` ve test adaptörü paket dışında kaldı.

Birleşik referans kimliği değişmedi: `243e1051a66efbeb8ee40850dbd3bd36fbed22976fdb288ca4a9bc3df0bcaa60`.

Yerel raporlar: `.data/runs/behavior/verify-4zjrejox/report.json`, `compare-cnc4j91j/report.json` (debug), `compare-62lr3md9/report.json` (release). NaN bit eşitliği bu kayıtlı ortam ve girdilerde ölçülmüştür; başka platformlarda ayrı Java baseline'ı gerekir.

## Yeniden çalıştırma ve sonraki dilim

Depo kökünde, yerel referans ve Cargo önbelleği mevcutken:

```bash
cargo build --locked --offline --example behavior_adapter
python3 tools/behavior/run.py capture --family math --family timer --family wire
python3 tools/behavior/run.py compare --family math --family timer --family wire --adapter target/debug/examples/behavior_adapter
```

`capture` mevcut beklentileri değiştirmez. Kayıtlı Java referansını yeniden ölçmek için `verify` kullanılır. Release için derlemeye `--release` eklenir ve `target/release/examples/behavior_adapter` seçilir.

P01/P02/P03 bölümlerinin tamamı ve karakter hareketi henüz doğrulanmış değildir. P03.b UTF-8 string okuma/yazma da 141 senaryoda doğrulandı; [kanıt](utf8-parity.md). İlgisiz sayısal yardımcılar peşinen genişletilmeyecek.
