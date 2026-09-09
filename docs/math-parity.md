# P01 — Math ailesi doğrulaması

`src/math.rs`, `src/aabb.rs` ve `src/shape.rs`, mevcut Java math ailesi senaryolarının kapsadığı sayısal ve geometrik yardımcıları içerir. Rust JSONL adaptörü bu modülü çağırır; beklenen değer veya oyun algoritması adaptör içinde üretilmez. Yeni bağımlılık eklenmedi.

`Shapes.equal`, iki shape'in `NOT_SAME` birleşiminde dolu hücre kalıp kalmadığını tersleyerek Java geometrik eşitlik sorgusunu korur; ayrıntılı senaryo ve rapor [eşitlik parity belgesindedir](shape-equal-parity.md).

`AABB.expandTowards(Vec3)`, Vec3 bileşenlerini scalar genişletme yoluna yönlendirir; signed zero ve ardışık genişletme senaryoları [ayrı parity belgesinde](aabb-expand-overload-parity.md) doğrulanır.

`AABB.equals` ve `hashCode`, endpoint karşılaştırması ile canonical NaN/signed-zero hash kurallarını [ayrı parity belgesinde](aabb-equality-parity.md) doğrular.

`Vec3.equals` ve `hashCode`, üç f64 bileşende Java `Double.compare`, canonical NaN ve signed zero kurallarını korur; altı senaryo [ayrı parity belgesinde](vec3-equality-parity.md) doğrulanır.

`Vec3.toString()`, üç bileşeni Java double metin kurallarıyla sabit `(x, y, z)` biçiminde birleştirir; altı senaryo [ayrı parity belgesinde](vec3-to-string-parity.md) doğrulanır.

`VoxelShape.equals(Object)`, shape geometrisinden bağımsız Java nesne kimliğini ve `Shapes.empty`/`block`/`INFINITY` singleton yollarını korur; `Shapes.equal` geometrik karşılaştırma için ayrı kalır. Senaryolar ve rapor [object-equals parity belgesindedir](shape-object-equals-parity.md).

`VoxelShape.toString()`, boş shape için `EMPTY`, dolu shape için Java AABB bounds metnini üretir; bilimsel sayı, Infinity ve NaN biçimleri [toString parity belgesinde](shape-to-string-parity.md) doğrulanır.

`AABB.Builder`, f32 `Vector3fc` noktalarını Java min/max ve tanımsız build hata kurallarıyla biriktirir; beş senaryo [Builder parity belgesinde](aabb-builder-parity.md) doğrulanır.

`AABB.deflate(double, double, double)`, eksen miktarlarını Java unary negation ile `inflate` yoluna aktarır; beş senaryo [deflate overload parity belgesinde](aabb-deflate-overload-parity.md) doğrulanır.

`Vec3.add` ve `Vec3.subtract` overload'ları, Vec3/scalar/üç bileşenli çağrıları Java'daki yönlendirme ve unary negation sırasıyla uygular; beş senaryo [ayrı parity belgesinde](vec3-add-subtract-parity.md) doğrulanır.

`Vec3.vectorTo`, hedef vektör bileşenlerinden mevcut bileşenleri çıkararak yönlü fark üretir; beş senaryo [ayrı parity belgesinde](vec3-vector-to-parity.md) doğrulanır.

`Vec3.dot`, üç bileşen çarpımını kaynak sırasındaki double toplamıyla üretir; beş senaryo [ayrı parity belgesinde](vec3-dot-parity.md) doğrulanır.

`Vec3.cross`, üç bileşenli vektörel çarpımı Java determinant sırasıyla üretir; beş senaryo [ayrı parity belgesinde](vec3-cross-parity.md) doğrulanır.

`Vec3.distanceTo`, farkların kareleri toplamının Java `Math.sqrt` sonucunu üretir; beş senaryo [ayrı parity belgesinde](vec3-distance-to-parity.md) doğrulanır. `Vec3.distanceToSqr`, Vec3 ve üç bileşenli overload'larında farkların kareleri toplamını üretir; beş senaryo [ayrı parity belgesinde](vec3-distance-to-sqr-parity.md) doğrulanır. `Vec3.reverse`, `scale(-1.0)` yolunu çağırarak her bileşenin işaretini çevirir; beş senaryo [ayrı parity belgesinde](vec3-reverse-parity.md) doğrulanır. `Vec3.horizontal`, x/z bileşenlerini koruyup y bileşenini pozitif sıfıra çevirir; beş senaryo [ayrı parity belgesinde](vec3-horizontal-parity.md) doğrulanır. `Vec3.horizontalDistance` ve `horizontalDistanceSqr`, yalnızca x/z karelerini kaynak sırasıyla hesaplar; beş senaryo [ayrı parity belgesinde](vec3-horizontal-distances-parity.md) doğrulanır.

`Vec3.closerThan` overload'ları, genel üç boyutlu ve yatay/y strict eşik karşılaştırmalarını Java ile aynı kısa devre sırasıyla üretir; beş senaryo [ayrı parity belgesinde](vec3-closer-than-parity.md) doğrulanır.

`Vec3.offsetRandom` ve `offsetRandomXZ`, `RandomSource.nextFloat()` çekişlerini Java'nın f32 ara hesabı ve overload'a özgü y davranışıyla uygular; beş senaryo [ayrı parity belgesinde](vec3-offset-random-parity.md) doğrulanır.

`Vec3.lerp`, üç bileşende Java `Mth.lerp(double, double, double)` sırasını ve clamp edilmemiş alpha davranışını korur; beş senaryo [ayrı parity belgesinde](vec3-lerp-parity.md) doğrulanır.

`Vec3.rotateClockwise90`, bileşenleri Java'nın `(-z, y, x)` sırasına taşır; beş senaryo [ayrı parity belgesinde](vec3-rotate-clockwise-90-parity.md) doğrulanır.

`Vec3.isFinite`, üç f64 bileşenin Java `Double.isFinite` sorgusunu `&&` sırasıyla uygular; beş senaryo [ayrı parity belgesinde](vec3-is-finite-parity.md) doğrulanır.

`Vec3.get`, `Direction.Axis` seçimine göre x, y veya z bileşenini ham f64 bitleriyle döndürür; beş senaryo [ayrı parity belgesinde](vec3-get-parity.md) doğrulanır.

`Vec3.with`, seçilen eksen bileşenini yeni f64 değeriyle değiştirip diğer bileşenleri ham bitleriyle korur; beş senaryo [ayrı parity belgesinde](vec3-with-parity.md) doğrulanır.

`Vec3.relative`, yön normalini distance ile çarpıp mevcut bileşenlere Java işlem sırasıyla ekler; altı senaryo [ayrı parity belgesinde](vec3-relative-parity.md) doğrulanır.

`Vec3.align`, EnumSet içindeki eksenlere Java `Mth.floor` uygular ve diğer bileşenleri korur; altı senaryo [ayrı parity belgesinde](vec3-align-parity.md) doğrulanır.

`Vec3.projectedOn`, `onto.lengthSqr() == 0.0` kısa yolunu ve `dot`/iki `scale` işlem sırasını korur; altı senaryo [ayrı parity belgesinde](vec3-projected-on-parity.md) doğrulanır.

`Vec3.toVector3f`, her f64 bileşeni bağımsız Java `(float)` dönüşümüyle f32 `Vector3f` bileşenine daraltır; altı senaryo [ayrı parity belgesinde](vec3-to-vector3f-parity.md) doğrulanır.

`Vec3` koordinat yardımcıları, BlockPos bileşenlerini ve sabit/çağrılan offset'leri Vec3'e taşır; altı senaryo [ayrı parity belgesinde](vec3-coordinates-parity.md) doğrulanır.

`Shapes.rotate`, signed eksen permütasyonunu, blok merkezi çevresindeki koordinat yansıtmasını ve discrete hücre indislerinin taşınmasını Java ile aynı sırada uygular; ayrıntılı senaryo ve rapor [rotation parity belgesindedir](shape-rotate-parity.md).

`Shapes.rotate` merkez overload'ı, output eksenine taşınan özel merkez bileşenlerini ve merkez farkı kadar koordinat çevirisini korur; ayrıntılı senaryo ve rapor [merkez rotation parity belgesindedir](shape-rotate-center-parity.md).

`Shapes.rotateAll` merkez overload'ı, bu merkez dönüşünü altı yönlü EnumMap sonucuna taşır; ayrıntılı senaryo ve rapor [rotateAll merkez parity belgesindedir](shape-rotate-all-center-parity.md).

`Shapes.rotateHorizontalAxis` merkez overload'ı, özel merkezi X dönüşüne iletip Z shape'ini korur; ayrıntılı senaryo ve rapor [yatay eksen merkez parity belgesindedir](shape-rotate-horizontal-axis-center-parity.md).

`Shapes.rotateAllAxis` merkez overload'ı, özel merkezi X/Y dönüşlerine iletip Z shape'ini korur; ayrıntılı senaryo ve rapor [all-axis merkez parity belgesindedir](shape-rotate-all-axis-center-parity.md).

Shapes.rotateAttachFace, varsayılan identity başlangıcıyla floor, wall ve ceiling için yatay yön map'leri üretir; bileşik eksen dönüşleri ve enum sırası [attach-face parity belgesinde](shape-rotate-attach-face-parity.md) doğrulanır.

Shapes.rotateHorizontal'ın desteklenen initial overload'ı, başlangıç dönüşü üzerine yatay yön bileşiklerini uygular; seçili başlangıçlar [initial parity belgesinde](shape-rotate-horizontal-initial-parity.md) doğrulanır.

Shapes.rotateAll'ın desteklenen initial ve rotationCenter overload'ı, aynı Java bileşimlerini altı yön map'ine taşır; seçili başlangıçlar [rotateAll initial parity belgesinde](shape-rotate-all-initial-parity.md) doğrulanır.

Shapes.rotateAttachFace'in desteklenen initial overload'ı, floor/wall/ceiling bileşiklerini aynı Java yatay map yoluna taşır; seçili başlangıçlar [attach-face initial parity belgesinde](shape-rotate-attach-face-initial-parity.md) doğrulanır.

## Referans ve davranış

Kaynak köküne göre `net/minecraft/world/phys/Vec2.java`, `net/minecraft/world/phys/Vec3.java`, `net/minecraft/util/RandomSource.java` ve `net/minecraft/util/Mth.java` yeniden incelendi. Referans Java 26.2 client JAR'ı; SHA-1 `2dc72797acbc1b63fc16a11c4ac393605f453754`. Mevcut `MathOracle.java` doğrudan orijinal sınıfları çağırır. Vec3 add/subtract, vectorTo, dot, cross, distanceTo, distanceToSqr, reverse, multiply, horizontal, horizontalDistance, horizontalDistanceSqr, closerThan, offsetRandom, lerp ve rotateClockwise90 çağrıları ile isFinite, get, with, relative, align, projectedOn, toVector3f, equals/hashCode, toString ve koordinat yardımcıları sorgusu senaryoları Java adaptörüne eklendi; beklenen çıktılar Java yakalamasından üretildi.

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
| `Vec3::add_components`, `add_scalar`, `add_vec3` | Üç bileşenli, scalar ve Vec3 add overload'ları Java yönlendirme sırasını korur |
| `Vec3::subtract_components`, `subtract_scalar`, `subtract_vec3` | Subtract miktarlarını Java unary negation ile tersleyip add yoluna verir |
| `Vec3::vector_to` | Hedef bileşenlerinden mevcut bileşenleri doğrudan çıkarır |
| `Vec3::dot` | Üç bileşen çarpımını kaynak sırasıyla toplar |
| `Vec3::cross` | Üç determinant bileşenini kaynak operand sırasıyla çıkarır |
| `Vec3::distance_to` | Fark kareleri toplamını kaynak sırasıyla toplar ve karekökünü alır |
| `Vec3::distance_to_sqr`, `distance_to_sqr_components` | Fark bileşenlerinin karelerini kaynak sırasıyla toplar |
| `Vec3::reverse` | `scale(-1.0)` yönlendirmesiyle her bileşenin işaretini çevirir |
| `Vec3::multiply_vec3`, `multiply_components` | Vec3 ve üç bileşenli multiply overload'larının yönlendirmesini ve x/y/z çarpım sırasını korur |
| `Vec3::horizontal` | x ve z bitlerini koruyup y bileşenini pozitif `0.0` yapar; girdi değişmez |
| `Vec3::horizontal_distance`, `horizontal_distance_squared` | Yalnızca x/z karelerini kaynak sırasıyla hesaplar; biri `sqrt` uygular |
| `Vec3::closer_than_position`, `closer_than_xz_y` | Strict genel ve xz/y eşiklerini, kısa devre sırasını korur |
| `Vec3::offset_random`, `offset_random_xz` | Random çekiş sırasını, f32 offset hesabını ve XZ y pozitif sıfırını korur |
| `Vec3::lerp` | Her bileşende `p0 + alpha * (p1 - p0)` f64 işlem sırasını ve clamp edilmemiş alpha'yı korur |
| `Vec3::rotate_clockwise_90` | `(-z, y, x)` eksen permütasyonunu ve unary negation işaret bitini korur |
| `Vec3::is_finite` | Üç bileşenin sonluluğunu Java `Double.isFinite` sonucu ile bağlar |
| `Vec3::get` | Direction.Axis X/Y/Z seçimini doğrudan x/y/z bileşenine bağlar; ham f64 bitlerini korur |
| `Vec3::with` | Seçilen eksen bileşenini değiştirir, diğer bileşenleri ve ham f64 bitlerini korur |
| `Vec3::relative` | Direction normalini distance ile çarpar ve bileşenlere Java işlem sırasıyla ekler |
| `Vec3::align` | EnumSet eksenlerinde Java floor/cast yolunu, seçilmeyen bileşenlerde ham bit korumasını sürdürür |
| `Vec3::projected_on` | Sıfır uzunluk karesi kısa yolunu ve `dot` → `scale` → `scale` işlem sırasını korur |
| `Vec3::to_vector3f` | Üç f64 bileşeni bağımsız Java f32 daraltmasıyla `Vec3f` değerine dönüştürür |
| `Vec3::equals_java`, `hash_code_java` | Java `Double.compare`, NaN canonicalization ve 31 tabanlı wrapping int hash aritmetiğini korur |
| `Vec3::to_string_java` | Java `Double.toString` bileşen yazımını ve `(x, y, z)` ayraçlarını korur |
| `Direction`/`Axis`/`AxisDirection` | Ad, veri değerleri, eksen, adım, karşıt yön ve eksenli döndürmeler; hizalı eksende birim dönüş |
| `floor_f32`, `floor_f64` | Aşağı yuvarlama ve i32 dönüşümü; sınır dışı sonuçlar doyar, NaN sıfıra dönüşür |
| `wrap_degrees_f32`, `wrap_degrees_f64` | İşaretli kalan üzerinden [-180, 180) aralığı; negatif sıfır kaybolmaz |
| `clamp_f32`, `clamp_f64` | Referansın alt sınır karşılaştırması ve üst sınır minimumu; NaN/sıfır davranışı Rust'ın genel yardımcılarına bırakılmaz |
| `block_to_section_coord`, `section_to_block_coord`, `section_relative` | Aritmetik kaydırma ve maskeleme; negatiflerde tabana yuvarlama, `0–15` aralığı |

`hypot`, çarpma-toplama birleştirmesi veya ortak ters uzunlukla çarpma kullanılmadı; matematiksel olarak benzer yazımlar ara yuvarlama veya taşmayı değiştirebilir. İki hassasiyetteki scalar işlevleri küçük bir özel makroyla üretiliyor; genel matematik trait sistemi veya haricî vektör kütüphanesi eklenmedi.

Bu API, Vec2/Vec3/Direction sınıflarının tamamı değildir: yön bulma, kuaterniyon ve resource codec'i kapsam dışındaki sonraki ihtiyaçlardır. Mth'nin int/long overload'ları, trigonometrik tablolar ve diğer yardımcıları bu dilimde uygulanmadı.

## Doğrulama

Mevcut 777 matematik senaryosu; bu corpus içinde Vec2 için sıfır/işaretli sıfır, eksen ve diyagonal vektörler, normalize eşiğinin komşuları ve ölçek-normalize zincirleri; Vec3 için 27 yön, işaretli sıfır, subnormal, en büyük sonlu, karışık bileşen, 1e-5 eşiğinin bit komşuları ve ölçek-normalize zincirleri; add/subtract için Vec3, scalar ve üç bileşenli zincirler, signed zero, Infinity ve NaN senaryoları; vectorTo için sonlu fark, zincir, signed zero, double taşması, Infinity ve NaN senaryoları; dot için temel çarpım, evaluation order/cancellation, signed zero, double overflow ve özel değer senaryoları; cross için determinant bileşenleri, zincir, signed zero, double overflow ve özel değer senaryoları; distanceTo için temel uzaklık, evaluation order, signed zero, subnormal underflow ve overflow/özel değer senaryoları; distanceToSqr için Vec3 ve üç bileşenli overload, evaluation order, signed zero ve overflow/NaN senaryoları; reverse için `scale(-1.0)` yönlendirmesi, signed zero, subnormal, büyük sonlu ve Infinity senaryoları; multiply için Vec3 ve üç bileşenli overload, overload zinciri, signed zero ve overflow/özel değer senaryoları; horizontal için x/z koruması, pozitif y sıfırı, signed zero, subnormal, büyük sonlu ve Infinity senaryoları; horizontalDistance için x/z kareleri, kaynak toplama sırası, signed zero, subnormal ve Infinity senaryoları; closerThan için strict sınır, y kapısı, negatif sıfır ve özel değer senaryoları; offsetRandom için üç çekiş, XZ iki çekiş, negatif/signed-zero/subnormal offset ve özel Vec3 değer senaryoları; lerp için temel, iki yönde extrapolation, signed zero/subnormal ve özel değer senaryoları; rotateClockwise90 için eksen permütasyonu, signed zero, sınır ve özel değer senaryoları; isFinite için sonlu sınırlar, signed zero, Infinity ve NaN senaryoları; get için x/y/z seçimi, signed zero, subnormal, sınır ve özel f64 değerleri; with için eksen bileşeni değişimi ve korunan ham bitler; relative için altı yön, işlem sırası ve özel f64 sonuçları; align için eksen üyeliği, negatif floor/cast sınırları, signed zero, subnormal, Infinity ve NaN senaryoları; projectedOn için sıfır uzunluk karesi kısa yolu, projection işlem sırası ve özel f64 sonuçları; toVector3f için bağımsız f64 → f32 daraltması, signed zero, subnormal, taşma ve özel değer sonuçları; koordinat yardımcıları için BlockPos alt köşesi, offset, merkez, taban merkezi, y offset'i ve özel f64 değerleri; equals/hashCode için temel, farklı bileşen, signed zero, NaN payload, Infinity ve özel değer senaryoları; toString için temel biçim, gösterim eşikleri, signed zero, Infinity ve NaN senaryoları; ayrıca büyük/küçük sayılar, f32/f64 sınır komşuları, sonsuzluk ve NaN girdilerini kapsıyor. Clamp sınırları mevcut corpus'ta [-1, 1] ve [0, 1]; ters/NaN sınırlar, bütün NaN payload'ları ve tüm float bit desenleri test edilmiş sayılmaz. Shape dönüşümleri için identity, dokuz signed 90/180 derece dönüş, farklı grid boyutu ve OR birleşimi senaryoları; merkez overload için özel merkezli dört dönüş senaryosu; `rotateHorizontalAxis` için X/Z map ve özel merkez overload için merkezli X/Z map, `rotateAllAxis` için X/Y/Z map ve özel merkez overload için merkezli X/Y/Z map, `rotateHorizontal` için dört yön map ve `rotateAll` için altı yön map; `rotateAll` merkez overload için özel merkezli üç map senaryosu; `rotateAttachFace` için floor, wall ve ceiling iç map'lerini karşılaştıran üç senaryo; `rotateHorizontal` desteklenen initial overload için altı senaryo; singleEncompassing için boş, temas eden ve ayrık birleşimden tek kapsama box'ı üreten üç senaryo; axis min/max için empty, ayrık ve negatif koordinatlı üç senaryo; axis min/max sütun sorguları için empty, ayrık ve negatif koordinatlı üç senaryo; VoxelShape move overload'ları için Vec3, Vec3i ve empty shape üç senaryosu; VoxelShape optimize/getCoords için empty, temas eden satır ve ayrık shape üç senaryosu; VoxelShape forAllEdges için empty, box ve birleşik 3B shape üç senaryosu; `Shapes.join`/`joinUnoptimized` coordinate plane ayrımı için üç senaryo; VoxelShape forAllBoxes callback'i için üç senaryo; `Shapes.create(AABB)` için üç senaryo; `Shapes.create(double...)` için üç senaryo; `Shapes.INFINITY` için üç senaryo; `Shapes.EPSILON` ve `Shapes.BIG_EPSILON` için üç senaryo; `AABB.expandTowards(Vec3)` için üç senaryo; `AABB.equals`/`hashCode` için beş senaryo; `AABB.toString()` için beş senaryo; `AABB.Builder` için beş senaryo; `AABB.deflate(double, double, double)` için beş senaryo; `VoxelShape.equals(Object)` için üç senaryo; `VoxelShape.toString()` için beş senaryo; grid ve empty senaryoları da corpus'a dahildir.

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Kayıtlı 1231 senaryoluk birleşik Java referansı iki ayrı JVM çalıştırmasıyla doğrulandı.
- Debug ve release Rust adaptörleri **1231/1231** eşleşti: 777 math + 22 timer + 144 wire + 141 utf8 + 116 binary + 31 nbt.
- Vec2/Vec3'te başlangıç ve her adımın bileşen, uzunluk ve uzunluk karesi bitleri; scalar işlevlerde sonuç bitleri veya tamsayı değeri karşılaştırıldı. Float toleransı veya NaN normalizasyonu eklenmedi.
- Yanlış f64 bit formatı, bilinmeyen tür/işlem/aile, eksik clamp sınırı ve bilinmeyen Vec2/Vec3 adımı reddedildi.
- Biçim, Clippy ve paket kapsamı kontrolleri geçti; `.data/` ve test adaptörü paket dışında kaldı.

Birleşik referans kimliği: `0f128e8e7fb68abc882891752c85767e02d3b02a6b9f83fff7f87b45ab6f2afe`. Math ailesi kimliği: `d6723874f1f8073346a0f729f2c06b2e00e2dcd34baad1a33733698d293350d3`.

Yerel raporlar: `compare-da6k05zu` (debug 777), `compare-jem8yqw3` (release 777) ve `verify-04cazxan` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Yeniden çalıştırma ve sonraki dilim

Depo kökünde, yerel referans ve Cargo önbelleği mevcutken:

```bash
cargo build --locked --offline --example behavior_adapter
python3 tools/behavior/run.py capture --family math --family timer --family wire
python3 tools/behavior/run.py compare --family math --family timer --family wire --adapter target/debug/examples/behavior_adapter
```

`capture` mevcut beklentileri değiştirmez. Kayıtlı Java referansını yeniden ölçmek için `verify` kullanılır. Release için derlemeye `--release` eklenir ve `target/release/examples/behavior_adapter` seçilir.

P01/P02/P03 bölümlerinin tamamı ve karakter hareketi henüz doğrulanmış değildir. Koordinat dönüşümleri 57 senaryoda doğrulandı; [sözleşme](coords-contract.md). P01.d/e random/AABB için 12 Java referans senaryosu hazır ([sözleşme](random-aabb-contract.md)). İlgisiz sayısal yardımcılar peşinen genişletilmeyecek.
