# P01 — Math ailesi doğrulaması

`src/math.rs`, `src/aabb.rs` ve `src/shape.rs`, mevcut Java math ailesi senaryolarının kapsadığı sayısal ve geometrik yardımcıları içerir. Rust JSONL adaptörü bu modülü çağırır; beklenen değer veya oyun algoritması adaptör içinde üretilmez. Yeni bağımlılık eklenmedi.

`Vec3.CODEC`, tam üç double öğeli listeyi x/y/z sırasıyla Vec3'e dönüştürür; kısa listelerde partial üretmez, uzun listelerde ilk üç öğeyi partial olarak taşır. 12 codec senaryosu [ayrı sözleşme ve parity belgelerinde](vec3-codec-contract.md) doğrulanır.
`ChunkPos.CODEC`, tam iki signed int öğeli listeyi x/z sırasıyla ChunkPos'a dönüştürür; kısa listelerde partial üretmez, uzun listelerde ilk iki öğeyi partial olarak taşır. 12 codec senaryosu [ayrı sözleşme ve parity belgelerinde](chunkpos-codec-contract.md) doğrulanır.
`ChunkPos` blok koordinatı yardımcıları, chunk x/z değerini Java'nın 4 bit shift yoluyla blok min/middle/max ve offset'li sonuçlara taşır; 6 senaryo [ayrı sözleşme ve parity belgelerinde](chunkpos-block-coordinates-contract.md) doğrulanır.
`ChunkPos` region koordinatı yardımcıları, signed chunk x/z değerlerini 5 bit aritmetik kaydırma ve düşük 5 bit maskesiyle region/local sonuçlara taşır; region min/max üretimi 6 senaryo [ayrı sözleşme ve parity belgelerinde](chunkpos-region-coordinates-contract.md) doğrulanır.
`ChunkPos` BlockPos yardımcıları, chunk offset'lerinden BlockPos üretir ve Java'nın x/z inclusive containment kontrolünü Y eksenini yok sayarak uygular; 6 senaryo [ayrı sözleşme ve parity belgelerinde](chunkpos-block-positions-contract.md) doğrulanır.
`ChunkPos` temsil ve hash yardımcıları, `[x, z]` metin biçimini ve Java'nın wrapping hash dönüşümünü taşır; 6 senaryo [ayrı sözleşme ve parity belgelerinde](chunkpos-representation-contract.md) doğrulanır.
`ChunkPos` BlockPos'tan containing/pack yardımcıları, BlockPos x/z değerlerini arithmetic shift ile chunk koordinatına çevirir ve packed long'un iki 32 bit alanına koyar; 6 senaryo [ayrı sözleşme ve parity belgelerinde](chunkpos-from-blockpos-contract.md) doğrulanır.
`ChunkPos` mesafe yardımcıları, chessboard mesafesinde Java `Math.abs`/`Math.max` ve kareli mesafede wrapping i32 aritmetiğini korur; 6 senaryo [ayrı sözleşme ve parity belgelerinde](chunkpos-distances-contract.md) doğrulanır.
`ChunkPos` validity yardımcıları, `ChunkPyramid` sınırını ve Java `Math.abs` kenar davranışını korur; 6 senaryo [ayrı sözleşme ve parity belgelerinde](chunkpos-validity-contract.md) doğrulanır.
`ChunkPos` section node dönüşümü, packed SectionPos long içinden signed x/z alanlarını çıkarıp y'yi atar; 6 senaryo [ayrı sözleşme ve parity belgelerinde](chunkpos-from-section-node-contract.md) doğrulanır.

`Shapes.equal`, iki shape'in `NOT_SAME` birleşiminde dolu hücre kalıp kalmadığını tersleyerek Java geometrik eşitlik sorgusunu korur; ayrıntılı senaryo ve rapor [eşitlik parity belgesindedir](shape-equal-parity.md).

`AABB.expandTowards(Vec3)`, Vec3 bileşenlerini scalar genişletme yoluna yönlendirir; signed zero ve ardışık genişletme senaryoları [ayrı parity belgesinde](aabb-expand-overload-parity.md) doğrulanır.

`AABB.equals` ve `hashCode`, endpoint karşılaştırması ile canonical NaN/signed-zero hash kurallarını [ayrı parity belgesinde](aabb-equality-parity.md) doğrular.

`Vec3.equals` ve `hashCode`, üç f64 bileşende Java `Double.compare`, canonical NaN ve signed zero kurallarını korur; altı senaryo [ayrı parity belgesinde](vec3-equality-parity.md) doğrulanır.

`Vec3.toString()`, üç bileşeni Java double metin kurallarıyla sabit `(x, y, z)` biçiminde birleştirir; altı senaryo [ayrı parity belgesinde](vec3-to-string-parity.md) doğrulanır.

`Vec3.xRot`, `yRot` ve `zRot`, Java `Mth` sin/cos lookup tablosunu ve üç eksen formülünü korur; 18 senaryo [ayrı parity belgesinde](vec3-rotation-parity.md) doğrulanır.

`Vec3.directionFromRotation`, float ve Vec2 overload'larında pitch/yaw derecelerini Java f32 ara işlemleri ve Mth lookup yoluyla yön Vec3'üne çevirir; 18 senaryo [ayrı parity belgesinde](vec3-direction-from-rotation-parity.md) doğrulanır.

`Vec3.rotation()`, f64 Vec3 bileşenlerinden f32 pitch/yaw Vec2'si üretir; 18 senaryo f64 `atan2`/`asin`/`sqrt`, açı daraltması ve Vec2 snapshot aritmetiğini [ayrı parity belgesinde](vec3-rotation-method-parity.md) doğrular.

`Vec3.applyLocalCoordinatesToRotation` ve `addLocalCoordinates`, f32 sin/cos taban vektörlerini ve f64 yerel bileşen toplamını Java ile aynı sırada üretir; 6 senaryo [ayrı parity belgesinde](vec3-local-coordinates-parity.md) doğrulanır.

`Vec2.rotate(double)`, f64 açıyı Java `Mth` sin/cos lookup'ına verir ve f32 vektör işlemlerini korur; 18 senaryo [ayrı parity belgesinde](vec2-rotate-parity.md) doğrulanır.

`Vec2.add(Vec2)` ve `Vec2.add(float)`, bileşenleri f32 olarak toplar; 5 senaryo [ayrı parity belgesinde](vec2-add-parity.md) doğrulanır.

`Vec2.distanceToSqr(Vec2)`, iki bileşen farkının karelerini kaynak sırasıyla f32 olarak toplar; 5 senaryo [ayrı parity belgesinde](vec2-distance-to-sqr-parity.md) doğrulanır.

`Vec2.negated()`, iki bileşenin f32 sign bit'lerini unary negation ile çevirir; 5 senaryo [ayrı parity belgesinde](vec2-negated-parity.md) doğrulanır.

`Vec2.equals(Object)` ve `hashCode()`, Java `==`, canonical NaN ve 31 tabanlı Float hash zincirini korur; 6 senaryo [ayrı parity belgesinde](vec2-equality-parity.md) doğrulanır.

`Vec2` public sabitleri, Java `ZERO`, `ONE`, eksen birimleri, `MAX` ve en küçük pozitif subnormal `MIN` değerlerini ham f32 bitleriyle taşır; 8 senaryo [ayrı parity belgesinde](vec2-constants-parity.md) doğrulanır.

`Vec3` public sabitleri, Java `ZERO`, `X_AXIS`, `Y_AXIS` ve `Z_AXIS` değerlerini ham f64 bitleriyle taşır; 4 senaryo [ayrı parity belgesinde](vec3-constants-parity.md) doğrulanır.

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

`directionFromRotation` için Java `Vec3.java:267-279`, `Vec2.java:18-27` ve `Mth.java:29-55` gövdeleri ayrıca incelendi. `rotation()` için `Vec3.java:275-279`, `Vec2.java:18-27,64-70` ve `Mth.java:58-59` gövdeleri ayrıca incelendi. `Vec2.rotate(double)` için `Vec2.java:82-86` ve `Mth.java:50-55` gövdeleri ayrıca incelendi. `Vec2.add` için `Vec2.java:37-43`, `Vec2.distanceToSqr` için `Vec2.java:72-76`, `Vec2.negated` için `Vec2.java:78-80`, `Vec2.equals`/`hashCode` için `Vec2.java:45-60` ve public sabitler için `Vec2.java:9-17` gövdeleri ayrıca incelendi.

Kaynak köküne göre `net/minecraft/world/phys/Vec2.java`, `net/minecraft/world/phys/Vec3.java`, `net/minecraft/util/RandomSource.java` ve `net/minecraft/util/Mth.java` yeniden incelendi. Referans Java 26.2 client JAR'ı; SHA-1 `2dc72797acbc1b63fc16a11c4ac393605f453754`. Mevcut `MathOracle.java` doğrudan orijinal sınıfları çağırır. Vec2 public sabitleri, add, distanceToSqr, negated ve equals/hashCode, Vec3 add/subtract, vectorTo, dot, cross, distanceTo, distanceToSqr, reverse, multiply, horizontal, horizontalDistance, horizontalDistanceSqr, closerThan, offsetRandom, lerp ve rotateClockwise90 çağrıları ile isFinite, get, with, relative, align, projectedOn, toVector3f, equals/hashCode, toString, `CODEC` ve koordinat yardımcıları sorgusu senaryoları Java adaptörüne eklendi; beklenen çıktılar Java yakalamasından üretildi.

Vec3 public sabitleri için Java `Vec3.java:37-40` gövdeleri ayrıca incelendi; `MathOracle.java` bu dört sabiti doğrudan sorgular ve Rust adaptörü aynı adları `Vec3::ZERO`, `X_AXIS`, `Y_AXIS` ve `Z_AXIS` üzerinden çağırır.

`directionFromRotation` iki overload'u 18 senaryoda Java f32 ara hassasiyeti, lookup quantization ve özel açı sonuçlarıyla doğrulandı.

`rotation()` 18 senaryoda f64 trigonometrik hesap sırası, f32 açı daraltması, signed zero ve özel değer sonuçlarıyla doğrulandı.

`Vec2.rotate(double)` 18 senaryoda f64 açı, Mth lookup quantization'ı, f32 aritmetik, signed zero ve özel değer sonuçlarıyla doğrulandı.

`Vec2.dot(Vec2)` 5 senaryoda f32 çarpım ve kaynak sıralı toplama, yuvarlama sınırı, signed zero, taşma ve özel değer sonuçlarıyla doğrulandı.

`Vec2.add(Vec2)` ve `Vec2.add(float)` 5 senaryoda bileşen toplama, signed zero, taşma ve özel değer sonuçlarıyla doğrulandı.

`Vec2.negated()` 5 senaryoda unary negation, signed zero, subnormal, Infinity/sınır ve NaN payload sonuçlarıyla doğrulandı.

`Vec2.equals(Object)`/`hashCode()` 6 senaryoda f32 `==`, signed zero/NaN eşitliği, canonical NaN hash'i, Infinity ve wrapping int aritmetiğiyle doğrulandı.

`Vec2` public sabitleri 8 senaryoda bileşen bitleri, `Float.MAX_VALUE` ve `Float.MIN_VALUE` pozitif subnormal sınırıyla doğrulandı.

`Vec3` public sabitleri 4 senaryoda `ZERO` ve üç pozitif birim eksenin f64 bileşen bitleriyle doğrulandı.

`Vec3` local coordinate yardımcıları 6 senaryoda statik dönüşüm, instance forwarding, f32 trigonometrik ara değerleri ve f64 Vec3 snapshot bitleriyle doğrulandı.

`Vec3.CODEC` 12 senaryoda tam üç öğeli decode, kısa/uzun liste hataları ve x/y/z encode sırasını ham f64 bitleriyle doğruladı.

| Rust API | Korunan davranış |
| --- | --- |
| `Vec2::new` | f32 bileşenler |
| `Vec2::ZERO`, `ONE`, `UNIT_X`, `NEG_UNIT_X`, `UNIT_Y`, `NEG_UNIT_Y`, `MAX`, `MIN` | Java public sabit değerlerini taşır; `MAX` f32 en büyük sonlu değeri, `MIN` ise `f32::from_bits(1)` ile en küçük pozitif subnormal değeri kullanır |
| `Vec2::scale` | Her bileşen f32 hassasiyetinde çarpılır |
| `Vec2::length_squared` | Ayrı çarpımlar ve toplam; ara f32 taşması/yuvarlaması ile Java NaN operand seçimi korunur |
| `Vec2::length` | f32 kareler toplamı f64 karekökten sonra f32'ye döner |
| `Vec2::normalized` | Uzunluk 0,0001'den küçükse sıfır; diğer durumda her bileşen uzunluğa ayrı bölünür |
| `Vec2::rotate` | F64 açıyı Java Mth sin/cos lookup'ına verip f32 çarpma ve toplama/çıkarma sırasıyla yeni Vec2 üretir |
| `Vec2::dot` | İki f32 çarpımını ayrı üretip Java kaynak sırasındaki toplamla tek f32 sonuç döndürür; ara yuvarlama, signed zero, taşma ve NaN davranışı korunur |
| `Vec2::add_vec2`, `Vec2::add_scalar` | Java'nın iki add overload'unda bileşenleri f32 olarak toplar; Vec2 ve scalar yollarının bileşen sırası, signed zero, taşma ve özel değer sonuçları korunur |
| `Vec2::distance_to_sqr` | İki bileşen farkını, karelerini ve kaynak sıralı f32 toplamını Java ile aynı sırada üretir; signed zero, taşma ve özel değer sonuçları korunur |
| `Vec2::negated` | İki f32 bileşenin sign bit'ini unary negation ile çevirir; signed zero, subnormal, Infinity ve NaN payload bitleri korunur |
| `Vec2::equals_java`, `Vec2::hash_code_java` | Java f32 `==` karşılaştırmasını, canonical NaN Float hash'ini ve `Objects.hash` 31 tabanlı wrapping int zincirini korur |
| `Vec3::new` | f64 bileşenler |
| `Vec3::codec_decode` | Java `Util.fixedSize` ile aynı tam üç öğe kabulünü, kısa listede partial olmamasını ve uzun listede ilk üç öğeden partial üretimini taşır |
| `Vec3::codec_encode` | x, y ve z bileşenlerini Java `Vec3.CODEC` encode sırasındaki sabit üçlü diziye koyar |
| `Vec3::ZERO`, `X_AXIS`, `Y_AXIS`, `Z_AXIS` | Java public sabit değerlerini taşır; üç eksen sabiti ilgili bileşende f64 `1.0`, diğer bileşenlerde pozitif `0.0` kullanır |
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
| `Vec3::x_rot`, `y_rot`, `z_rot` | F32 radyanları Java `Mth` sin/cos lookup sonuçlarıyla f64 dönüş formüllerine taşır |
| `Vec3::direction_from_rotation`, `direction_from_rotation_components` | Pitch/yaw f32 bileşenlerini Java derece→radyan ara işlemleri, Mth lookup ve f32 ürünleriyle f64 Vec3 yönüne taşır |
| `Vec3::rotation` | F64 `atan2`/`asin`/`sqrt` sırasını, açıların f32'ye daraltılmasını, Java derece sabitini ve Vec2 pitch/yaw sonucunu korur; Vec2 uzunluk karesinde Java NaN operand seçimi sürdürülür |
| `Vec3::apply_local_coordinates_to_rotation`, `add_local_coordinates` | Java pitch/yaw taban vektörlerini f32 trigonometrik ara değerleriyle kurar; cross/scale ve yerel bileşen f64 toplama sırasını korur, instance yolu `rotation()` sonucunu kullanır |
| `Direction`/`Axis`/`AxisDirection` | Ad, veri değerleri, eksen, adım, karşıt yön ve eksenli döndürmeler; hizalı eksende birim dönüş |
| `floor_f32`, `floor_f64` | Aşağı yuvarlama ve i32 dönüşümü; sınır dışı sonuçlar doyar, NaN sıfıra dönüşür |
| `wrap_degrees_f32`, `wrap_degrees_f64` | İşaretli kalan üzerinden [-180, 180) aralığı; negatif sıfır kaybolmaz |
| `clamp_f32`, `clamp_f64` | Referansın alt sınır karşılaştırması ve üst sınır minimumu; NaN/sıfır davranışı Rust'ın genel yardımcılarına bırakılmaz |
| `block_to_section_coord`, `section_to_block_coord`, `section_relative` | Aritmetik kaydırma ve maskeleme; negatiflerde tabana yuvarlama, `0–15` aralığı |
| `ChunkPos::containing`, `ChunkPos::pack_block_pos` | BlockPos x/z değerlerini 4 bit arithmetic shift ile chunk'a çevirir; containing y'yi yok sayar ve packed sonucu düşük x/yüksek z 32 bit alanlarında taşır |
| `ChunkPos::get_chessboard_distance`, `get_chessboard_distance_coordinates` | Chunk koordinatı farklarını Java arithmetic i32 çıkarma, `Math.abs`/`Math.max` ve chessboard sonucu ile taşır |
| `ChunkPos::distance_squared`, `distance_squared_from_packed` | Chunk koordinatı farklarının karelerini ve toplamını Java i32 wrapping sırasıyla üretir; packed overload'u x/z alanlarını açar |
| `ChunkPos::is_valid`, `is_valid_coordinates` | `Mth.absMax` sonucunu Java `ChunkPyramid.MAX_CHUNK_COORDINATE_VALUE` sınırıyla karşılaştırır; `Integer.MIN_VALUE` mutlak değer kenarı korunur |
| `ChunkPos::from_section_node` | SectionPos packed long içinden signed x/z alanlarını çıkarıp y alanını atarak ChunkPos packed long üretir |

`hypot`, çarpma-toplama birleştirmesi veya ortak ters uzunlukla çarpma kullanılmadı; matematiksel olarak benzer yazımlar ara yuvarlama veya taşmayı değiştirebilir. İki hassasiyetteki scalar işlevleri küçük bir özel makroyla üretiliyor; genel matematik trait sistemi veya haricî vektör kütüphanesi eklenmedi.

Bu API, Vec2/Vec3/Direction sınıflarının tamamı değildir: kuaterniyon ve diğer resource codec'leri kapsam dışındaki sonraki ihtiyaçlardır. Mth'nin int/long overload'ları ve diğer yardımcıları bu dilimde uygulanmadı.

## Doğrulama

Mevcut 965 matematik senaryosu; bu corpus içinde Vec2 için sıfır/işaretli sıfır, eksen ve diyagonal vektörler, normalize eşiğinin komşuları ve ölçek-normalize zincirleri; Vec3 için 27 yön, işaretli sıfır, subnormal, en büyük sonlu, karışık bileşen, 1e-5 eşiğinin bit komşuları ve ölçek-normalize zincirleri; Vec3 CODEC için tam/eksik/fazla üçlü listeler, encode sırası ve signed zero/özel f64 bitleri; add/subtract için Vec3, scalar ve üç bileşenli zincirler, signed zero, Infinity ve NaN senaryoları; vectorTo için sonlu fark, zincir, signed zero, double taşması, Infinity ve NaN senaryoları; dot için temel çarpım, evaluation order/cancellation, signed zero, double overflow ve özel değer senaryoları; cross için determinant bileşenleri, zincir, signed zero, double overflow ve özel değer senaryoları; distanceTo için temel uzaklık, evaluation order, signed zero, subnormal underflow ve overflow/özel değer senaryoları; distanceToSqr için Vec3 ve üç bileşenli overload, evaluation order, signed zero ve overflow/NaN senaryoları; reverse için `scale(-1.0)` yönlendirmesi, signed zero, subnormal, büyük sonlu ve Infinity senaryoları; multiply için Vec3 ve üç bileşenli overload, overload zinciri, signed zero ve overflow/özel değer senaryoları; horizontal için x/z koruması, pozitif y sıfırı, signed zero, subnormal, büyük sonlu ve Infinity senaryoları; horizontalDistance için x/z kareleri, kaynak toplama sırası, signed zero, subnormal ve Infinity senaryoları; closerThan için strict sınır, y kapısı, negatif sıfır ve özel değer senaryoları; offsetRandom için üç çekiş, XZ iki çekiş, negatif/signed-zero/subnormal offset ve özel Vec3 değer senaryoları; lerp için temel, iki yönde extrapolation, signed zero/subnormal ve özel değer senaryoları; rotateClockwise90 için eksen permütasyonu, signed zero, sınır ve özel değer senaryoları; isFinite için sonlu sınırlar, signed zero, Infinity ve NaN senaryoları; get için x/y/z seçimi, signed zero, subnormal, sınır ve özel f64 değerleri; with için eksen bileşeni değişimi ve korunan ham bitler; relative için altı yön, işlem sırası ve özel f64 sonuçları; align için eksen üyeliği, negatif floor/cast sınırları, signed zero, subnormal, Infinity ve NaN senaryoları; projectedOn için sıfır uzunluk karesi kısa yolu, projection işlem sırası ve özel f64 sonuçları; toVector3f için bağımsız f64 → f32 daraltması, signed zero, subnormal, taşma ve özel değer sonuçları; koordinat yardımcıları için BlockPos alt köşesi, offset, merkez, taban merkezi, y offset'i ve özel f64 değerleri; equals/hashCode için temel, farklı bileşen, signed zero, NaN payload, Infinity ve özel değer senaryoları; toString için temel biçim, gösterim eşikleri, signed zero, Infinity ve NaN senaryoları; xRot/yRot/zRot için üç eksen, negatif/küçük quantized/çeyrek dönüş açıları, signed zero, özel vektörler ve özel açı senaryoları; directionFromRotation için iki overload, cardinal ve birleşik pitch/yaw, lookup quantization, signed zero, özel açı ve f32 taşma senaryoları; rotation() için eksen, birleşik/non-unit vektör, sıfır/signed-zero, subnormal, sonlu sınır ve Infinity/NaN sonuçları; Vec2 rotate(double) için f64 açı, lookup quantization, eksen dönüşleri, signed zero, subnormal, özel açı ve f32 taşması; Vec2 dot(Vec2) için temel f32 çarpım, yuvarlama sınırı, signed zero, overflow ve özel değer sonuçları; Vec2 add(Vec2)/add(float) için bileşen toplama, signed zero, taşma ve özel değer sonuçları; Vec2 distanceToSqr için çıkarma, kare alma, kaynak sıralı toplama, signed zero, taşma ve özel değer sonuçları; ayrıca büyük/küçük sayılar, f32/f64 sınır komşuları, sonsuzluk ve NaN girdilerini kapsıyor. Clamp sınırları mevcut corpus'ta [-1, 1] ve [0, 1]; ters/NaN sınırlar, bütün NaN payload'ları ve tüm float bit desenleri test edilmiş sayılmaz. Shape dönüşümleri için identity, dokuz signed 90/180 derece dönüş, farklı grid boyutu ve OR birleşimi senaryoları; merkez overload için özel merkezli dört dönüş senaryosu; `rotateHorizontalAxis` için X/Z map ve özel merkez overload için merkezli X/Z map, `rotateAllAxis` için X/Y/Z map ve özel merkez overload için merkezli X/Y/Z map, `rotateHorizontal` için dört yön map ve `rotateAll` için altı yön map; `rotateAll` merkez overload için özel merkezli üç map senaryosu; `rotateAttachFace` için floor, wall ve ceiling iç map'lerini karşılaştıran üç senaryo; `rotateHorizontal` desteklenen initial overload için altı senaryo; singleEncompassing için boş, temas eden ve ayrık birleşimden tek kapsama box'ı üreten üç senaryo; axis min/max için empty, ayrık ve negatif koordinatlı üç senaryo; axis min/max sütun sorguları için empty, ayrık ve negatif koordinatlı üç senaryo; VoxelShape move overload'ları için Vec3, Vec3i ve empty shape üç senaryosu; VoxelShape optimize/getCoords için empty, temas eden satır ve ayrık shape üç senaryosu; VoxelShape forAllEdges için empty, box ve birleşik 3B shape üç senaryosu; `Shapes.join`/`joinUnoptimized` coordinate plane ayrımı için üç senaryo; VoxelShape forAllBoxes callback'i için üç senaryo; `Shapes.create(AABB)` için üç senaryo; `Shapes.create(double...)` için üç senaryo; `Shapes.INFINITY` için üç senaryo; `Shapes.EPSILON` ve `Shapes.BIG_EPSILON` için üç senaryo; `AABB.expandTowards(Vec3)` için üç senaryo; `AABB.equals`/`hashCode` için beş senaryo; `AABB.toString()` için beş senaryo; `AABB.Builder` için beş senaryo; `AABB.deflate(double, double, double)` için beş senaryo; `VoxelShape.equals(Object)` için üç senaryo; `VoxelShape.toString()` için beş senaryo; grid ve empty senaryoları da corpus'a dahildir.

Java 25.0.4.1 ve Rust 1.98.1 / Linux x86_64 ortamında:

- Kayıtlı 1467 senaryoluk birleşik Java referansı iki ayrı JVM çalıştırmasıyla doğrulandı.
- Debug ve release Rust adaptörleri **1467/1467** eşleşti: 965 math + 22 timer + 144 wire + 141 utf8 + 164 binary + 31 nbt.
- Vec2/Vec3'te başlangıç ve her adımın bileşen, uzunluk ve uzunluk karesi bitleri; scalar işlevlerde sonuç bitleri veya tamsayı değeri karşılaştırıldı. Float toleransı veya NaN normalizasyonu eklenmedi.
- Yanlış f64 bit formatı, bilinmeyen tür/işlem/aile, eksik clamp sınırı ve bilinmeyen Vec2/Vec3 adımı reddedildi.
- Biçim, Clippy ve paket kapsamı kontrolleri geçti; `.data/` ve test adaptörü paket dışında kaldı.

Birleşik referans kimliği: `5cb121b3bc1e12571ad171166a785300fd6e581cf78813e4ddaf2b99d19ab7d1`. Math ailesi kimliği: `bbc2c1dc1c594cc962e6acbf6fcdc98815c896f66dcad6a72c2da4be5b0a0f81`.

Yerel raporlar: `compare-ctepk6ji` (math debug 965), `compare-5wubt_if` (math release 965), `compare-lsdsnr89` (birleşik debug 1467), `compare-b5y_ka7p` (birleşik release 1467) ve `verify-o3reu3wd` dizinlerinde, `.data/runs/behavior/` altında bulunur.

## Yeniden çalıştırma ve sonraki dilim

Depo kökünde, yerel referans ve Cargo önbelleği mevcutken:

```bash
cargo build --locked --offline --example behavior_adapter
python3 tools/behavior/run.py capture --family math --family timer --family wire
python3 tools/behavior/run.py compare --family math --family timer --family wire --adapter target/debug/examples/behavior_adapter
```

`capture` mevcut beklentileri değiştirmez. Kayıtlı Java referansını yeniden ölçmek için `verify` kullanılır. Release için derlemeye `--release` eklenir ve `target/release/examples/behavior_adapter` seçilir.

P01/P02/P03 bölümlerinin tamamı ve karakter hareketi henüz doğrulanmış değildir. Koordinat dönüşümleri 57 senaryoda doğrulandı; [sözleşme](coords-contract.md). P01.d/e random/AABB için 12 Java referans senaryosu hazır ([sözleşme](random-aabb-contract.md)). İlgisiz sayısal yardımcılar peşinen genişletilmeyecek.
