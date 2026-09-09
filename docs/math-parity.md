# P01 — Math ailesi doğrulaması

`src/math.rs`, `src/aabb.rs` ve `src/shape.rs`, mevcut Java math ailesi senaryolarının kapsadığı sayısal ve geometrik yardımcıları içerir. Rust JSONL adaptörü bu modülü çağırır; beklenen değer veya oyun algoritması adaptör içinde üretilmez. Yeni bağımlılık eklenmedi.

`Shapes.equal`, iki shape'in `NOT_SAME` birleşiminde dolu hücre kalıp kalmadığını tersleyerek Java geometrik eşitlik sorgusunu korur; ayrıntılı senaryo ve rapor [eşitlik parity belgesindedir](shape-equal-parity.md).

`Shapes.rotate`, signed eksen permütasyonunu, blok merkezi çevresindeki koordinat yansıtmasını ve discrete hücre indislerinin taşınmasını Java ile aynı sırada uygular; ayrıntılı senaryo ve rapor [rotation parity belgesindedir](shape-rotate-parity.md).

`Shapes.rotate` merkez overload'ı, output eksenine taşınan özel merkez bileşenlerini ve merkez farkı kadar koordinat çevirisini korur; ayrıntılı senaryo ve rapor [merkez rotation parity belgesindedir](shape-rotate-center-parity.md).

`Shapes.rotateAll` merkez overload'ı, bu merkez dönüşünü altı yönlü EnumMap sonucuna taşır; ayrıntılı senaryo ve rapor [rotateAll merkez parity belgesindedir](shape-rotate-all-center-parity.md).

## Referans ve davranış

Kaynak köküne göre `net/minecraft/world/phys/Vec2.java`, `net/minecraft/world/phys/Vec3.java` ve `net/minecraft/util/Mth.java` yeniden incelendi. Referans Java 26.2 client JAR'ı; SHA-1 `2dc72797acbc1b63fc16a11c4ac393605f453754`. Mevcut `MathOracle.java` doğrudan orijinal sınıfları çağırır. Rust implementasyonu için senaryolar, Java adaptörü ve beklenen çıktılar değiştirilmedi.

| Rust API | Korunan davranış |
| --- | --- |
| `Vec2::new`, `ZERO` | f32 bileşenler; sıfır vektör pozitif sıfır bileşenleri taşır |
| `Vec2::scale` | Her bileşen f32 hassasiyetinde çarpılır |
| `Vec2::length_squared` | Ayrı çarpımlar ve toplam; ara f32 taşması/yuvarlaması korunur |
| `Vec2::length` | f32 kareler toplamı f64 karekökten sonra f32'ye döner |
| `Vec2::normalized` | Uzunluk 0,0001'den küçükse sıfır; diğer durumda her bileşen uzunluğa ayrı bölünür |
| `Vec3::new`, `ZERO` | f64 bileşenler; sıfır vektör pozitif sıfır bileşenleri taşır |
| `Vec3::scale` | Her bileşen f64 hassasiyetinde çarpılır |
| `Vec3::length_squared` | Ayrı çarpımlar ve toplam; ara f64 taşması/yuvarlaması korunur |
| `Vec3::length` | Double kareler toplamının karekökü |
| `Vec3::normalized` | Uzunluk float `1e-5` literalin double genişlemesinden küçükse sıfır; diğer durumda her bileşen uzunluğa ayrı bölünür |
| `Direction`/`Axis`/`AxisDirection` | Ad, veri değerleri, eksen, adım, karşıt yön ve eksenli döndürmeler; hizalı eksende birim dönüş |
| `floor_f32`, `floor_f64` | Aşağı yuvarlama ve i32 dönüşümü; sınır dışı sonuçlar doyar, NaN sıfıra dönüşür |
| `wrap_degrees_f32`, `wrap_degrees_f64` | İşaretli kalan üzerinden [-180, 180) aralığı; negatif sıfır kaybolmaz |
| `clamp_f32`, `clamp_f64` | Referansın alt sınır karşılaştırması ve üst sınır minimumu; NaN/sıfır davranışı Rust'ın genel yardımcılarına bırakılmaz |
| `block_to_section_coord`, `section_to_block_coord`, `section_relative` | Aritmetik kaydırma ve maskeleme; negatiflerde tabana yuvarlama, `0–15` aralığı |

`hypot`, çarpma-toplama birleştirmesi veya ortak ters uzunlukla çarpma kullanılmadı; matematiksel olarak benzer yazımlar ara yuvarlama veya taşmayı değiştirebilir. İki hassasiyetteki scalar işlevleri küçük bir özel makroyla üretiliyor; genel matematik trait sistemi veya haricî vektör kütüphanesi eklenmedi.

Bu API, Vec2/Vec3/Direction sınıflarının tamamı değildir: dot/cross/add/subtract, yön bulma, kuaterniyon, Java nesne eşitliği/hash'i ve resource codec'i kapsam dışındaki sonraki ihtiyaçlardır. Mth'nin int/long overload'ları, trigonometrik tablolar ve diğer yardımcıları bu dilimde uygulanmadı.

## Doğrulama

Mevcut 559 matematik senaryosu; bu corpus içinde Vec2 için sıfır/işaretli sıfır, eksen ve diyagonal vektörler, normalize eşiğinin komşuları ve ölçek-normalize zincirleri; Vec3 için 27 yön, işaretli sıfır, subnormal, en büyük sonlu, karışık bileşen, 1e-5 eşiğinin bit komşuları ve ölçek-normalize zincirleri; ayrıca büyük/küçük sayılar, f32/f64 sınır komşuları, sonsuzluk ve NaN girdilerini kapsıyor. Clamp sınırları mevcut corpus'ta [-1, 1] ve [0, 1]; ters/NaN sınırlar, bütün NaN payload'ları ve tüm float bit desenleri test edilmiş sayılmaz. Shape dönüşümleri için identity, dokuz signed 90/180 derece dönüş, farklı grid boyutu ve OR birleşimi senaryoları; merkez overload için özel merkezli dört dönüş senaryosu; `rotateHorizontalAxis` için X/Z map, `rotateAllAxis` için X/Y/Z map, `rotateHorizontal` için dört yön map ve `rotateAll` için altı yön map; `rotateAll` merkez overload için özel merkezli üç map senaryosu; grid ve empty senaryoları da corpus'a dahildir.

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Kayıtlı 1013 senaryoluk birleşik Java referansı iki ayrı JVM çalıştırmasıyla doğrulandı.
- Debug ve release Rust adaptörleri **1013/1013** eşleşti: 559 math + 22 timer + 144 wire + 141 utf8 + 116 binary + 31 nbt.
- Vec2/Vec3'te başlangıç ve her adımın bileşen, uzunluk ve uzunluk karesi bitleri; scalar işlevlerde sonuç bitleri veya tamsayı değeri karşılaştırıldı. Float toleransı veya NaN normalizasyonu eklenmedi.
- Yanlış f64 bit formatı, bilinmeyen tür/işlem/aile, eksik clamp sınırı ve bilinmeyen Vec2/Vec3 adımı reddedildi.
- Biçim, Clippy ve paket kapsamı kontrolleri geçti; `.data/` ve test adaptörü paket dışında kaldı.

Birleşik referans kimliği: `8bd229a580fc01d7c478c33923afb04c0fee7803869b7423f2d3ad4587821b9f`. Math ailesi kimliği: `2809154f292cf47ecd8113f8c4766b7a0af297d2ab88e185e421bdfac514e9c9`.

Yerel raporlar: `compare-tl6kn3g9` (debug 559), `compare-uges__2a` (release 559) dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Yeniden çalıştırma ve sonraki dilim

Depo kökünde, yerel referans ve Cargo önbelleği mevcutken:

```bash
cargo build --locked --offline --example behavior_adapter
python3 tools/behavior/run.py capture --family math --family timer --family wire
python3 tools/behavior/run.py compare --family math --family timer --family wire --adapter target/debug/examples/behavior_adapter
```

`capture` mevcut beklentileri değiştirmez. Kayıtlı Java referansını yeniden ölçmek için `verify` kullanılır. Release için derlemeye `--release` eklenir ve `target/release/examples/behavior_adapter` seçilir.

P01/P02/P03 bölümlerinin tamamı ve karakter hareketi henüz doğrulanmış değildir. Koordinat dönüşümleri 57 senaryoda doğrulandı; [sözleşme](coords-contract.md). P01.d/e random/AABB için 12 Java referans senaryosu hazır ([sözleşme](random-aabb-contract.md)). İlgisiz sayısal yardımcılar peşinen genişletilmeyecek.
