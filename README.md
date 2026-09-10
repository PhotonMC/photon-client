# voxel-client

Minecraft Java Edition istemcisinin Rust ile sıfırdan yeniden uygulanması için hazırlık deposu. `voxel-client` geçici proje adıdır; Minecraft ile resmî bağlantısı yoktur.

Hedef, oyuncunun fark hissedemeyeceği davranış ve sunum eşitliğidir: mekanikler, hareket, animasyon, texture, UI ve sesler dahil. Sunucu geliştirilmeyecek; [Pumpkin](https://github.com/Pumpkin-Mc/Pumpkin) kullanılacak. Referans Java Edition **26.2**, protokol **776**, Java **25** olarak sabitlendi. İncelenen Pumpkin adayında 776 sürüm eşlemesi mevcut; gerçek bağlantı ve mekanik uyumluluğu henüz doğrulanmadı.

Orijinal kaynak ve assetler yalnızca yerelde referans ve test girdisi olarak kullanılacak. Git'e, GitHub'a, derleme çıktısına veya dağıtım paketine eklenmeyecek. İstemci ileride kullanıcının seçtiği yerel asset klasörünü okuyacak; kurulum orijinal asset içermeyecek veya otomatik indirmeyecek.

## Mevcut aşama

P01.b Vec3 `directionFromRotation` overload'ları 18 senaryoda Java/Rust bit eşitliğiyle doğrulandı ([kanıt](docs/vec3-direction-from-rotation-parity.md)).

P01.b Vec3 `rotation()` 18 senaryoda Java/Rust bit eşitliğiyle doğrulandı ([kanıt](docs/vec3-rotation-method-parity.md)).

P01.b Vec2 `rotate(double)` 18 senaryoda Java/Rust bit eşitliğiyle doğrulandı ([kanıt](docs/vec2-rotate-parity.md)).

P01.b Vec2 `dot(Vec2)` 5 senaryoda Java/Rust bit eşitliğiyle doğrulandı ([kanıt](docs/vec2-dot-parity.md)).

P01.b Vec2 `add(Vec2)` ve `add(float)` overload'ları 5 senaryoda Java/Rust bit eşitliğiyle doğrulandı ([kanıt](docs/vec2-add-parity.md)).

P01.b Vec2 `distanceToSqr(Vec2)` 5 senaryoda Java/Rust bit eşitliğiyle doğrulandı ([kanıt](docs/vec2-distance-to-sqr-parity.md)).

P01.b Vec2 `negated()` 5 senaryoda Java/Rust bit eşitliğiyle doğrulandı ([kanıt](docs/vec2-negated-parity.md)).

P01.b Vec2 `equals(Object)`/`hashCode()` 6 senaryoda Java/Rust bit eşitliğiyle doğrulandı ([kanıt](docs/vec2-equality-parity.md)).

P01.b Vec2 public sabitleri 8 senaryoda Java/Rust bit eşitliğiyle doğrulandı ([kanıt](docs/vec2-constants-parity.md)).

P01.b Vec3 public sabitleri 4 senaryoda Java/Rust bit eşitliğiyle doğrulandı ([kanıt](docs/vec3-constants-parity.md)).

P01.b Vec3 local coordinate yardımcıları 6 senaryoda Java/Rust bit eşitliğiyle doğrulandı ([kanıt](docs/vec3-local-coordinates-parity.md)).

P01.b Vec3 `CODEC` 12 senaryoda Java/Rust bit eşitliğiyle doğrulandı ([sözleşme](docs/vec3-codec-contract.md), [kanıt](docs/vec3-codec-parity.md)).

P03.b Vec3.STREAM_CODEC 13 senaryoda Java/Rust byte eşitliğiyle doğrulandı ([kanıt](docs/vec3-stream-codec-parity.md)).
P03.b LP Vec3 stream codec 15 senaryoda Java/Rust byte eşitliğiyle doğrulandı ([sözleşme](docs/lp-vec3-contract.md), [kanıt](docs/lp-vec3-parity.md)).
P03.b ChunkPos.STREAM_CODEC 20 senaryoda Java/Rust byte eşitliğiyle doğrulandı ([sözleşme](docs/chunkpos-contract.md), [kanıt](docs/chunkpos-parity.md)).
P01.c ChunkPos.CODEC 12 senaryoda Java/Rust liste eşitliğiyle doğrulandı ([sözleşme](docs/chunkpos-codec-contract.md), [kanıt](docs/chunkpos-codec-parity.md)).
P01.c ChunkPos blok koordinatı yardımcıları 6 senaryoda Java/Rust liste eşitliğiyle doğrulandı ([sözleşme](docs/chunkpos-block-coordinates-contract.md), [kanıt](docs/chunkpos-block-coordinates-parity.md)).
P01.c ChunkPos region koordinatı yardımcıları 6 senaryoda Java/Rust liste eşitliğiyle doğrulandı ([sözleşme](docs/chunkpos-region-coordinates-contract.md), [kanıt](docs/chunkpos-region-coordinates-parity.md)).
P01.c ChunkPos BlockPos yardımcıları 6 senaryoda Java/Rust liste eşitliğiyle doğrulandı ([sözleşme](docs/chunkpos-block-positions-contract.md), [kanıt](docs/chunkpos-block-positions-parity.md)).
P01.c ChunkPos temsil ve hash 6 senaryoda Java/Rust liste eşitliğiyle doğrulandı ([sözleşme](docs/chunkpos-representation-contract.md), [kanıt](docs/chunkpos-representation-parity.md)).
P01.c ChunkPos BlockPos'tan containing/pack 6 senaryoda Java/Rust liste eşitliğiyle doğrulandı ([sözleşme](docs/chunkpos-from-blockpos-contract.md), [kanıt](docs/chunkpos-from-blockpos-parity.md)).
P01.c ChunkPos mesafe yardımcıları 6 senaryoda Java/Rust liste eşitliğiyle doğrulandı ([sözleşme](docs/chunkpos-distances-contract.md), [kanıt](docs/chunkpos-distances-parity.md)).
P01.c ChunkPos validity yardımcıları 6 senaryoda Java/Rust liste eşitliğiyle doğrulandı ([sözleşme](docs/chunkpos-validity-contract.md), [kanıt](docs/chunkpos-validity-parity.md)).
P01.c ChunkPos section node dönüşümü 6 senaryoda Java/Rust liste eşitliğiyle doğrulandı ([sözleşme](docs/chunkpos-from-section-node-contract.md), [kanıt](docs/chunkpos-from-section-node-parity.md)).
P01.c ChunkPos rangeClosed 8 senaryoda Java/Rust liste eşitliğiyle doğrulandı ([sözleşme](docs/chunkpos-range-closed-contract.md), [kanıt](docs/chunkpos-range-closed-parity.md)).
P01.c ChunkPos public sabitleri 4 senaryoda Java/Rust liste eşitliğiyle doğrulandı ([sözleşme](docs/chunkpos-constants-contract.md), [kanıt](docs/chunkpos-constants-parity.md)).
P01.c ChunkPos packed koordinat erişimcileri 6 senaryoda Java/Rust liste eşitliğiyle doğrulandı ([sözleşme](docs/chunkpos-packed-accessors-contract.md), [kanıt](docs/chunkpos-packed-accessors-parity.md)).
P01.c SectionPos packed koordinat erişimcileri 6 senaryoda Java/Rust liste eşitliğiyle doğrulandı ([sözleşme](docs/sectionpos-packed-accessors-contract.md), [kanıt](docs/sectionpos-packed-accessors-parity.md)).
P01.c SectionPos blok koordinatı yardımcıları 6 senaryoda Java/Rust liste eşitliğiyle doğrulandı ([sözleşme](docs/sectionpos-block-coordinates-contract.md), [kanıt](docs/sectionpos-block-coordinates-parity.md)).
P01.c SectionPos relative-to-block yardımcıları 6 senaryoda Java/Rust liste eşitliğiyle doğrulandı ([sözleşme](docs/sectionpos-relative-to-block-contract.md), [kanıt](docs/sectionpos-relative-to-block-parity.md)).
P01.c SectionPos spatial yardımcıları 6 senaryoda Java/Rust liste eşitliğiyle doğrulandı ([sözleşme](docs/sectionpos-spatial-contract.md), [kanıt](docs/sectionpos-spatial-parity.md)).
P01.c SectionPos packed dönüşümleri 6 senaryoda Java/Rust liste eşitliğiyle doğrulandı ([sözleşme](docs/sectionpos-packed-transformations-contract.md), [kanıt](docs/sectionpos-packed-transformations-parity.md)).
P01.c SectionPos asLong overload'ları 6 senaryoda Java/Rust liste eşitliğiyle doğrulandı ([sözleşme](docs/sectionpos-as-long-contract.md), [kanıt](docs/sectionpos-as-long-parity.md)).
P01.c SectionPos offset overload'ları 6 senaryoda Java/Rust liste eşitliğiyle doğrulandı ([sözleşme](docs/sectionpos-offset-contract.md), [kanıt](docs/sectionpos-offset-parity.md)).
P01.c SectionPos range yardımcıları 6 senaryoda Java/Rust liste eşitliğiyle doğrulandı ([sözleşme](docs/sectionpos-ranges-contract.md), [kanıt](docs/sectionpos-ranges-parity.md)).
P01.c SectionPos blocksInside 3 senaryoda Java/Rust liste eşitliğiyle doğrulandı ([sözleşme](docs/sectionpos-blocks-inside-contract.md), [kanıt](docs/sectionpos-blocks-inside-parity.md)).
P01.c SectionPos aroundAndAtBlockPos 6 senaryoda Java/Rust liste eşitliğiyle doğrulandı ([sözleşme](docs/sectionpos-around-and-at-contract.md), [kanıt](docs/sectionpos-around-and-at-parity.md)).
P01.c SectionPos double koordinat yardımcıları 6 senaryoda Java/Rust liste eşitliğiyle doğrulandı ([sözleşme](docs/sectionpos-double-coordinates-contract.md), [kanıt](docs/sectionpos-double-coordinates-parity.md)).
P01.c SectionPos `of(...)` veri overload'ları 6 senaryoda Java/Rust liste eşitliğiyle doğrulandı ([sözleşme](docs/sectionpos-of-overloads-contract.md), [kanıt](docs/sectionpos-of-overloads-parity.md)).
P01.a Mth tamsayı yardımcıları 15 senaryoda Java/Rust sonuç eşitliğiyle doğrulandı ([sözleşme](docs/mth-integer-helpers-contract.md), [kanıt](docs/mth-integer-helpers-parity.md)).
P01.a Mth tamsayı temel yardımcıları 25 senaryoda Java/Rust sonuç eşitliğiyle doğrulandı ([sözleşme](docs/mth-integer-basics-contract.md), [kanıt](docs/mth-integer-basics-parity.md)).
P01.a Mth açı yardımcıları 21 senaryoda Java/Rust sonuç eşitliğiyle doğrulandı ([sözleşme](docs/mth-angle-helpers-contract.md), [kanıt](docs/mth-angle-helpers-parity.md)).
P01.a Mth yuvarlama yardımcıları 36 senaryoda Java/Rust sonuç eşitliğiyle doğrulandı ([sözleşme](docs/mth-rounding-helpers-contract.md), [kanıt](docs/mth-rounding-helpers-parity.md)).
P01.a Mth pozitif modulo 44 senaryoda Java/Rust sonuç eşitliğiyle doğrulandı ([sözleşme](docs/mth-positive-modulo-contract.md), [kanıt](docs/mth-positive-modulo-parity.md)).
P01.a Mth absMax 28 senaryoda Java/Rust sonuç eşitliğiyle doğrulandı ([sözleşme](docs/mth-abs-max-contract.md), [kanıt](docs/mth-abs-max-parity.md)).
P01.a Mth equal 28 senaryoda Java/Rust sonuç eşitliğiyle doğrulandı ([sözleşme](docs/mth-equal-contract.md), [kanıt](docs/mth-equal-parity.md)).
P01.a Mth clampedLerp 32 senaryoda Java/Rust sonuç eşitliğiyle doğrulandı ([sözleşme](docs/mth-clamped-lerp-contract.md), [kanıt](docs/mth-clamped-lerp-parity.md)).
P01.a Mth next random 20 senaryoda Java/Rust sonuç eşitliğiyle doğrulandı ([sözleşme](docs/mth-next-random-contract.md), [kanıt](docs/mth-next-random-parity.md)).
P01.a Mth random helpers 18 senaryoda Java/Rust sonuç eşitliğiyle doğrulandı ([sözleşme](docs/mth-random-helpers-contract.md), [kanıt](docs/mth-random-helpers-parity.md)).
P01.a Mth length 16 senaryoda Java/Rust sonuç eşitliğiyle doğrulandı ([sözleşme](docs/mth-length-contract.md), [kanıt](docs/mth-length-parity.md)).
P01.a Mth quantize 19 senaryoda Java/Rust sonuç eşitliğiyle doğrulandı ([sözleşme](docs/mth-quantize-contract.md), [kanıt](docs/mth-quantize-parity.md)).
P01.a Mth outFromOrigin 24 senaryoda Java/Rust sonuç eşitliğiyle doğrulandı ([sözleşme](docs/mth-out-from-origin-contract.md), [kanıt](docs/mth-out-from-origin-parity.md)).
P01.a Mth mulAndTruncate 20 senaryoda Java/Rust sonuç eşitliğiyle doğrulandı ([sözleşme](docs/mth-mul-and-truncate-contract.md), [kanıt](docs/mth-mul-and-truncate-parity.md)).
P01.a Mth rotationAroundAxis 24 senaryoda Java/Rust sonuç eşitliğiyle doğrulandı ([sözleşme](docs/mth-rotation-around-axis-contract.md), [kanıt](docs/mth-rotation-around-axis-parity.md)).

P01.a Mth inverseLerp 28 senaryoda Java/Rust sonuç eşitliğiyle doğrulandı ([sözleşme](docs/mth-inverse-lerp-contract.md), [kanıt](docs/mth-inverse-lerp-parity.md)).
P01.a Mth lerp 28 senaryoda Java/Rust sonuç eşitliğiyle doğrulandı ([sözleşme](docs/mth-lerp-contract.md), [kanıt](docs/mth-lerp-parity.md)).
P01.a Mth lerpInt/lerpDiscrete 28 senaryoda Java/Rust sonuç eşitliğiyle doğrulandı ([sözleşme](docs/mth-lerp-integer-contract.md), [kanıt](docs/mth-lerp-integer-parity.md)).
P01.a Mth smoothstep/smoothstepDerivative 28 senaryoda Java/Rust sonuç eşitliğiyle doğrulandı ([sözleşme](docs/mth-smoothstep-contract.md), [kanıt](docs/mth-smoothstep-parity.md)).
P01.a Mth sign 14 senaryoda Java/Rust sonuç eşitliğiyle doğrulandı ([sözleşme](docs/mth-sign-contract.md), [kanıt](docs/mth-sign-parity.md)).
P01.a Mth rotLerp float/double overload'ları 30 senaryoda Java/Rust sonuç eşitliğiyle doğrulandı ([sözleşme](docs/mth-rot-lerp-contract.md), [kanıt](docs/mth-rot-lerp-parity.md)).
P01.a Mth rotLerpRad 15 senaryoda Java/Rust sonuç eşitliğiyle doğrulandı ([sözleşme](docs/mth-rot-lerp-rad-contract.md), [kanıt](docs/mth-rot-lerp-rad-parity.md)).
P01.a Mth triangleWave 15 senaryoda Java/Rust sonuç eşitliğiyle doğrulandı ([sözleşme](docs/mth-triangle-wave-contract.md), [kanıt](docs/mth-triangle-wave-parity.md)).
P01.a Mth square(float) 15 senaryoda Java/Rust sonuç eşitliğiyle doğrulandı ([sözleşme](docs/mth-square-f32-contract.md), [kanıt](docs/mth-square-f32-parity.md)).
P01.a Mth cube(float) 15 senaryoda Java/Rust sonuç eşitliğiyle doğrulandı ([sözleşme](docs/mth-cube-f32-contract.md), [kanıt](docs/mth-cube-f32-parity.md)).
P01.a Mth square(double) 15 senaryoda Java/Rust sonuç eşitliğiyle doğrulandı ([sözleşme](docs/mth-square-f64-contract.md), [kanıt](docs/mth-square-f64-parity.md)).

P01.a Mth square(int) 15 senaryoda Java/Rust sonuç eşitliğiyle doğrulandı ([sözleşme](docs/mth-square-i32-contract.md), [kanıt](docs/mth-square-i32-parity.md)).

P01.a Mth square(long) 15 senaryoda Java/Rust sonuç eşitliğiyle doğrulandı ([sözleşme](docs/mth-square-i64-contract.md), [kanıt](docs/mth-square-i64-parity.md)).

P01.a Mth clampedMap(double) 20 senaryoda Java/Rust sonuç eşitliğiyle doğrulandı ([sözleşme](docs/mth-clamped-map-f64-contract.md), [kanıt](docs/mth-clamped-map-f64-parity.md)).

P01.a Mth clampedMap(float) 20 senaryoda Java/Rust sonuç eşitliğiyle doğrulandı ([sözleşme](docs/mth-clamped-map-f32-contract.md), [kanıt](docs/mth-clamped-map-f32-parity.md)).

P01.a Mth map(double) 20 senaryoda Java/Rust sonuç eşitliğiyle doğrulandı ([sözleşme](docs/mth-map-f64-contract.md), [kanıt](docs/mth-map-f64-parity.md)).

P01.a Mth map(float) 20 senaryoda Java/Rust sonuç eşitliğiyle doğrulandı ([sözleşme](docs/mth-map-f32-contract.md), [kanıt](docs/mth-map-f32-parity.md)).

P01.a Mth wobble(double) 20 senaryoda Java/Rust sonuç eşitliğiyle doğrulandı ([sözleşme](docs/mth-wobble-contract.md), [kanıt](docs/mth-wobble-parity.md)).
P01.a Mth lerp2/lerp3/catmullrom 60 senaryoda Java/Rust sonuç eşitliğiyle doğrulandı ([sözleşme](docs/mth-interpolation-contract.md), [kanıt](docs/mth-interpolation-parity.md)).
P01.a Mth kayan nokta temel yardımcıları 60 senaryoda Java/Rust sonuç eşitliğiyle doğrulandı ([sözleşme](docs/mth-floating-primitives-contract.md), [kanıt](docs/mth-floating-primitives-parity.md)).

Kaynaklar sağlandı; bütünlük envanteri ve ilk kaynak incelemesi tamamlandı. Bölüm bazlı roadmap ve ilk davranış testi sözleşmeleri hazır. P00 yerel test çalıştırıcısı ve 2405 Java referans senaryosu mevcut. Mth tamsayı yardımcıları, Mth tamsayı temel yardımcıları, Mth açı yardımcıları, Mth yuvarlama yardımcıları, Mth pozitif modulo, Mth absMax, Mth equal, Mth clampedLerp, Mth next random, Mth random helpers, Mth length, Mth quantize, Mth outFromOrigin, Mth mulAndTruncate, Mth rotationAroundAxis, Mth inverseLerp, Mth lerp, Mth lerpInt/lerpDiscrete, Mth smoothstep/smoothstepDerivative, Mth sign, Mth rotLerp, Mth rotLerpRad, Mth triangleWave, Mth square(float), Mth cube(float), Mth square(double), Mth square(int), Mth square(long), Mth clampedMap(double), Mth clampedMap(float), Mth map(double), Mth map(float), Mth wobble(double), Mth lerp2/lerp3/catmullrom, Mth kayan nokta temel yardımcıları, Vec3 add/subtract overload'ları, vectorTo, dot, cross, distanceTo, distanceToSqr, reverse(), multiply overload'ları, horizontal(), horizontalDistance() ve horizontalDistanceSqr(), closerThan() overload'ları, offsetRandom()/offsetRandomXZ(), lerp(), rotateClockwise90(), isFinite(), get(), with(), relative(), align(), projectedOn(), toVector3f(), Vec3 koordinat yardımcıları, Vec3 equals/hashCode, Vec3 public constants, Vec3 local coordinate helpers, Vec3 CODEC, ChunkPos.CODEC, ChunkPos blok koordinatı yardımcıları, ChunkPos region koordinatı yardımcıları, ChunkPos BlockPos yardımcıları, ChunkPos temsil ve hash, ChunkPos BlockPos'tan containing/pack, ChunkPos mesafe yardımcıları, ChunkPos validity yardımcıları, ChunkPos section node dönüşümü, ChunkPos rangeClosed, ChunkPos public sabitleri, ChunkPos packed koordinat erişimcileri, SectionPos packed koordinat erişimcileri, SectionPos blok koordinatı yardımcıları, SectionPos double koordinat yardımcıları, SectionPos of(...) veri overload'ları, SectionPos relative-to-block yardımcıları, SectionPos spatial yardımcıları, SectionPos packed dönüşümleri, SectionPos asLong overload'ları, SectionPos offset overload'ları, P01.c SectionPos blocksInside, P01.c SectionPos aroundAndAtBlockPos, P03.b Vec3.STREAM_CODEC, P03.b LP Vec3 stream codec, P03.b ChunkPos.STREAM_CODEC, Vec3 toString(), Vec3 xRot/yRot/zRot, Vec3 rotation(), Vec3 directionFromRotation overload'ları, Vec2 rotate(double), Vec2 dot(Vec2), Vec2 add overload'ları, Vec2 distanceToSqr, Vec2 negated, Vec2 equals/hashCode, Vec2 public constants, Random/AABB, AABB contract, AABB expandTowards(Vec3), AABB equals/hashCode, AABB toString(), AABB.Builder, AABB deflate üç eksenli overload'u, AABB inflate, AABB intersect, AABB predicate overload'ları, AABB collidedAlongVector, AABB kutu kurucuları ve üreticileri, AABB BoundingBox köprüsü, AABB minmax, AABB move, AABB move overload'ları, AABB sizes, AABB hasNaN, AABB positions, AABB axis min/max, AABB distanceToSqr, AABB ofSize, AABB endpoint setter'ları, AABB clip, koordinat ve iterable clip/BlockHitResult overload'ları, VoxelShape temel/OR, varargs OR, singleEncompassing, axis min/max, min/max sütun sorguları, move overload'ları, optimize, getCoords, join/joinUnoptimized, forAllBoxes, Shapes.create(AABB), Shapes.create(double...), Shapes.INFINITY, Shapes.EPSILON/BIG_EPSILON ve forAllEdges, BooleanOp join, tekli/çoklu shape collision, VoxelShape clip, VoxelShape closestPointTo, VoxelShape getFaceShape, Shapes.blockOccludes, mergedFaceOccludes/faceShapeOccludes, Shapes.equal, VoxelShape equals(Object), VoxelShape toString(), Shapes.rotate, Shapes.rotate merkez overload'ı, Shapes.rotateHorizontalAxis ve merkez overload'ı, Shapes.rotateAllAxis ve merkez overload'ı, Shapes.rotateHorizontal, Shapes.rotateHorizontal initial, Shapes.rotateAll initial, Shapes.rotateAll, Shapes.rotateAll merkez overload'ı, Shapes.rotateAttachFace, Shapes.rotateAttachFace initial, nextGaussian, fork/positional, setSeed, random yardımcı overload'ları, NBT ağaç/liste/dizi, BlockPos/section-relative, SectionPos blocksInside ve [SectionPos of(...) veri overload'ları](docs/sectionpos-of-overloads-parity.md) dilimleri dahil Rust uygulanan kapsam debug/release profillerinde 2405 senaryonun tamamında Java ile eşleşiyor. Oynanabilir istemci henüz yok. Boş `main`, yalnızca iskelettir.

Mevcut referanslar [yerel veri rehberindeki](docs/reference-data.md) `.data/` alanında kalır. Test yol haritası kalan P01 yardımcıları ve bağlantı önkoşullarıyla sürüyor. Her Rust diliminden önce ilgili Java referansı ve testler hazırlanacak. Grafik, ses, pencere ve ağ teknolojileri sonraya bırakıldı. Test otomasyonu Python standart kütüphanesi ve özgün Java adaptörlerini kullanıyor; üretim kütüphanesine ek Rust bağımlılığı eklenmedi. Rust JSONL test adaptörü `serde_json` geliştirme bağımlılığını kullanıyor.

## Belgeler

- [Tüm ajanlar için çalışma kuralları](AGENTS.md)
- [Yerel kaynak ve asset yerleşimi](docs/reference-data.md)
- [Davranış doğrulama yaklaşımı](docs/behavior-testing.md)
- [Doğrulanan referans envanteri ve Pumpkin adayı](docs/reference-baseline.md)
- [Kaynak incelemesi ve istemci sınırları](docs/source-review.md)
- [Bölüm bazlı roadmap](docs/roadmap.md)
- [İlk davranış testi sözleşmeleri](docs/first-behavior-tests.md)
- [Yerel test komutları ve sonuçların anlamı](docs/behavior-runner.md)
- [VarInt/VarLong doğrulaması](docs/wire-parity.md)
- [Zamanlayıcı doğrulaması](docs/timer-parity.md)
- [Vec2 ve sayısal yardımcıların doğrulaması](docs/math-parity.md)
- [Vec3 davranış sözleşmesi ve referans testleri](docs/vec3-contract.md)
- [Vec3 add/subtract overload sözleşmesi](docs/vec3-add-subtract-contract.md)
- [Vec3 add/subtract overload doğrulaması](docs/vec3-add-subtract-parity.md)
- [Vec3 vectorTo sözleşmesi](docs/vec3-vector-to-contract.md)
- [Vec3 vectorTo doğrulaması](docs/vec3-vector-to-parity.md)
- [Vec3 dot sözleşmesi](docs/vec3-dot-contract.md)
- [Vec3 dot doğrulaması](docs/vec3-dot-parity.md)
- [Vec3 cross sözleşmesi](docs/vec3-cross-contract.md)
- [Vec3 cross doğrulaması](docs/vec3-cross-parity.md)
- [Vec3 distanceTo sözleşmesi](docs/vec3-distance-to-contract.md)
- [Vec3 distanceTo doğrulaması](docs/vec3-distance-to-parity.md)
- [Vec3 distanceToSqr sözleşmesi](docs/vec3-distance-to-sqr-contract.md)
- [Vec3 distanceToSqr doğrulaması](docs/vec3-distance-to-sqr-parity.md)
- [Vec3 reverse sözleşmesi](docs/vec3-reverse-contract.md)
- [Vec3 reverse doğrulaması](docs/vec3-reverse-parity.md)
- [Vec3 multiply overload sözleşmesi](docs/vec3-multiply-contract.md)
- [Vec3 multiply overload doğrulaması](docs/vec3-multiply-parity.md)
- [Vec3 horizontal sözleşmesi](docs/vec3-horizontal-contract.md)
- [Vec3 horizontal doğrulaması](docs/vec3-horizontal-parity.md)
- [Vec3 horizontalDistance sözleşmesi](docs/vec3-horizontal-distances-contract.md)
- [Vec3 horizontalDistance doğrulaması](docs/vec3-horizontal-distances-parity.md)
- [Vec3 CODEC sözleşmesi](docs/vec3-codec-contract.md)
- [Vec3 CODEC doğrulaması](docs/vec3-codec-parity.md)
- [ChunkPos.CODEC sözleşmesi](docs/chunkpos-codec-contract.md)
- [ChunkPos.CODEC doğrulaması](docs/chunkpos-codec-parity.md)
- [ChunkPos blok koordinatı yardımcıları sözleşmesi](docs/chunkpos-block-coordinates-contract.md)
- [ChunkPos blok koordinatı yardımcıları doğrulaması](docs/chunkpos-block-coordinates-parity.md)
- [ChunkPos region koordinatı yardımcıları sözleşmesi](docs/chunkpos-region-coordinates-contract.md)
- [ChunkPos region koordinatı yardımcıları doğrulaması](docs/chunkpos-region-coordinates-parity.md)
- [ChunkPos BlockPos yardımcıları sözleşmesi](docs/chunkpos-block-positions-contract.md)
- [ChunkPos BlockPos yardımcıları doğrulaması](docs/chunkpos-block-positions-parity.md)
- [ChunkPos temsil ve hash sözleşmesi](docs/chunkpos-representation-contract.md)
- [ChunkPos temsil ve hash doğrulaması](docs/chunkpos-representation-parity.md)
- [ChunkPos BlockPos'tan containing/pack sözleşmesi](docs/chunkpos-from-blockpos-contract.md)
- [ChunkPos BlockPos'tan containing/pack doğrulaması](docs/chunkpos-from-blockpos-parity.md)
- [ChunkPos mesafe yardımcıları sözleşmesi](docs/chunkpos-distances-contract.md)
- [ChunkPos mesafe yardımcıları doğrulaması](docs/chunkpos-distances-parity.md)
- [Vec3 closerThan sözleşmesi](docs/vec3-closer-than-contract.md)
- [Vec3 closerThan doğrulaması](docs/vec3-closer-than-parity.md)
- [Vec3 offsetRandom sözleşmesi](docs/vec3-offset-random-contract.md)
- [Vec3 offsetRandom doğrulaması](docs/vec3-offset-random-parity.md)
- [Vec3 lerp sözleşmesi](docs/vec3-lerp-contract.md)
- [Vec3 lerp doğrulaması](docs/vec3-lerp-parity.md)
- [Vec3 rotateClockwise90 sözleşmesi](docs/vec3-rotate-clockwise-90-contract.md)
- [Vec3 rotateClockwise90 doğrulaması](docs/vec3-rotate-clockwise-90-parity.md)
- [Vec3 isFinite sözleşmesi](docs/vec3-is-finite-contract.md)
- [Vec3 isFinite doğrulaması](docs/vec3-is-finite-parity.md)
- [Vec3 get sözleşmesi](docs/vec3-get-contract.md)
- [Vec3 get doğrulaması](docs/vec3-get-parity.md)
- [Vec3 with sözleşmesi](docs/vec3-with-contract.md)
- [Vec3 with doğrulaması](docs/vec3-with-parity.md)
- [Vec3 relative sözleşmesi](docs/vec3-relative-contract.md)
- [Vec3 relative doğrulaması](docs/vec3-relative-parity.md)
- [Vec3 align sözleşmesi](docs/vec3-align-contract.md)
- [Vec3 align doğrulaması](docs/vec3-align-parity.md)
- [Vec3 projectedOn sözleşmesi](docs/vec3-projected-on-contract.md)
- [Vec3 projectedOn doğrulaması](docs/vec3-projected-on-parity.md)
- [Vec3 toVector3f sözleşmesi](docs/vec3-to-vector3f-contract.md)
- [Vec3 toVector3f doğrulaması](docs/vec3-to-vector3f-parity.md)
- [Vec3 koordinat yardımcıları sözleşmesi](docs/vec3-coordinates-contract.md)
- [Vec3 koordinat yardımcıları doğrulaması](docs/vec3-coordinates-parity.md)
- [Vec3 equals/hashCode sözleşmesi](docs/vec3-equality-contract.md)
- [Vec3 equals/hashCode doğrulaması](docs/vec3-equality-parity.md)
- [Vec3 toString sözleşmesi](docs/vec3-to-string-contract.md)
- [Vec3 toString doğrulaması](docs/vec3-to-string-parity.md)
- [Vec3 xRot/yRot/zRot sözleşmesi](docs/vec3-rotation-contract.md)
- [Vec3 xRot/yRot/zRot doğrulaması](docs/vec3-rotation-parity.md)
- [Vec3 directionFromRotation sözleşmesi](docs/vec3-direction-from-rotation-contract.md)
- [Vec3 directionFromRotation doğrulaması](docs/vec3-direction-from-rotation-parity.md)
- [Vec3 rotation() sözleşmesi](docs/vec3-rotation-method-contract.md)
- [Vec3 rotation() doğrulaması](docs/vec3-rotation-method-parity.md)
- [Vec2 rotate(double) sözleşmesi](docs/vec2-rotate-contract.md)
- [Vec2 rotate(double) doğrulaması](docs/vec2-rotate-parity.md)
- [Vec2 dot(Vec2) sözleşmesi](docs/vec2-dot-contract.md)
- [Vec2 dot(Vec2) doğrulaması](docs/vec2-dot-parity.md)
- [Vec2 add overload'ları sözleşmesi](docs/vec2-add-contract.md)
- [Vec2 add overload'ları doğrulaması](docs/vec2-add-parity.md)
- [Vec2 distanceToSqr sözleşmesi](docs/vec2-distance-to-sqr-contract.md)
- [Vec2 distanceToSqr doğrulaması](docs/vec2-distance-to-sqr-parity.md)
- [Vec2 negated sözleşmesi](docs/vec2-negated-contract.md)
- [Vec2 negated doğrulaması](docs/vec2-negated-parity.md)
- [Vec2 equals/hashCode sözleşmesi](docs/vec2-equality-contract.md)
- [Vec2 equals/hashCode doğrulaması](docs/vec2-equality-parity.md)
- [Vec2 sabitleri sözleşmesi](docs/vec2-constants-contract.md)
- [Vec2 sabitleri doğrulaması](docs/vec2-constants-parity.md)
- [Direction davranış sözleşmesi ve referans testleri](docs/direction-contract.md)
- [Koordinat dönüşümleri sözleşmesi ve referans testleri](docs/coords-contract.md)
- [Random/AABB sözleşmesi ve referans testleri](docs/random-aabb-contract.md)
- [BlockPos/section-relative sözleşmesi ve referans testleri](docs/blockpos-contract.md)
- [BlockPos/section-relative doğrulaması](docs/blockpos-parity.md)
- [AABB contract sözleşmesi ve referans testleri](docs/aabb-contract.md)
- [AABB contract doğrulaması](docs/aabb-parity.md)
- [AABB expandTowards(Vec3) sözleşmesi](docs/aabb-expand-overload-contract.md)
- [AABB expandTowards(Vec3) doğrulaması](docs/aabb-expand-overload-parity.md)
- [AABB equals/hashCode sözleşmesi](docs/aabb-equality-contract.md)
- [AABB equals/hashCode doğrulaması](docs/aabb-equality-parity.md)
- [AABB toString sözleşmesi](docs/aabb-to-string-contract.md)
- [AABB toString doğrulaması](docs/aabb-to-string-parity.md)
- [AABB.Builder sözleşmesi](docs/aabb-builder-contract.md)
- [AABB.Builder doğrulaması](docs/aabb-builder-parity.md)
- [AABB deflate üç eksenli overload sözleşmesi](docs/aabb-deflate-overload-contract.md)
- [AABB deflate üç eksenli overload doğrulaması](docs/aabb-deflate-overload-parity.md)
- [AABB inflate sözleşmesi ve referans testleri](docs/aabb-inflate-contract.md)
- [AABB inflate doğrulaması](docs/aabb-inflate-parity.md)
- [AABB intersect sözleşmesi ve referans testleri](docs/aabb-intersect-contract.md)
- [AABB intersect doğrulaması](docs/aabb-intersect-parity.md)
- [AABB predicate overload sözleşmesi ve referans testleri](docs/aabb-predicate-overloads-contract.md)
- [AABB predicate overload doğrulaması](docs/aabb-predicate-overloads-parity.md)
- [AABB collidedAlongVector sözleşmesi ve referans testleri](docs/aabb-collided-along-vector-contract.md)
- [AABB collidedAlongVector doğrulaması](docs/aabb-collided-along-vector-parity.md)
- [AABB kutu kurucuları ve üreticileri sözleşmesi](docs/aabb-constructors-contract.md)
- [AABB kutu kurucuları ve üreticileri doğrulaması](docs/aabb-constructors-parity.md)
- [AABB BoundingBox köprüsü sözleşmesi](docs/aabb-bounding-box-contract.md)
- [AABB BoundingBox köprüsü doğrulaması](docs/aabb-bounding-box-parity.md)
- [AABB minmax sözleşmesi ve referans testleri](docs/aabb-minmax-contract.md)
- [AABB minmax doğrulaması](docs/aabb-minmax-parity.md)
- [AABB move sözleşmesi ve referans testleri](docs/aabb-move-contract.md)
- [AABB move doğrulaması](docs/aabb-move-parity.md)
- [AABB move overload sözleşmesi ve referans testleri](docs/aabb-move-overloads-contract.md)
- [AABB move overload doğrulaması](docs/aabb-move-overloads-parity.md)
- [AABB sizes sözleşmesi ve referans testleri](docs/aabb-sizes-contract.md)
- [AABB sizes doğrulaması](docs/aabb-sizes-parity.md)
- [AABB hasNaN sözleşmesi ve referans testleri](docs/aabb-has-nan-contract.md)
- [AABB hasNaN doğrulaması](docs/aabb-has-nan-parity.md)
- [AABB positions sözleşmesi ve referans testleri](docs/aabb-positions-contract.md)
- [AABB positions doğrulaması](docs/aabb-positions-parity.md)
- [AABB axis min/max sözleşmesi ve referans testleri](docs/aabb-axis-limits-contract.md)
- [AABB axis min/max doğrulaması](docs/aabb-axis-limits-parity.md)
- [AABB distanceToSqr sözleşmesi ve referans testleri](docs/aabb-distance-contract.md)
- [AABB distanceToSqr doğrulaması](docs/aabb-distance-parity.md)
- [AABB ofSize sözleşmesi ve referans testleri](docs/aabb-of-size-contract.md)
- [AABB ofSize doğrulaması](docs/aabb-of-size-parity.md)
- [AABB endpoint setter sözleşmesi ve referans testleri](docs/aabb-set-endpoint-contract.md)
- [AABB endpoint setter doğrulaması](docs/aabb-set-endpoint-parity.md)
- [AABB clip sözleşmesi ve referans testleri](docs/aabb-clip-contract.md)
- [AABB clip doğrulaması](docs/aabb-clip-parity.md)
- [AABB koordinat clip overloadu sözleşmesi](docs/aabb-clip-coordinates-contract.md)
- [AABB koordinat clip overloadu doğrulaması](docs/aabb-clip-coordinates-parity.md)
- [AABB iterable clip ve BlockHitResult sözleşmesi](docs/aabb-clip-many-contract.md)
- [AABB iterable clip ve BlockHitResult doğrulaması](docs/aabb-clip-many-parity.md)
- [VoxelShape temel/OR doğrulaması](docs/shape-parity.md)
- [Shapes.or varargs sözleşmesi](docs/shape-or-many-contract.md)
- [Shapes.or varargs doğrulaması](docs/shape-or-many-parity.md)
- [VoxelShape.singleEncompassing sözleşmesi](docs/shape-single-encompassing-contract.md)
- [VoxelShape.singleEncompassing doğrulaması](docs/shape-single-encompassing-parity.md)
- [VoxelShape min/max sözleşmesi](docs/shape-axis-limits-contract.md)
- [VoxelShape min/max doğrulaması](docs/shape-axis-limits-parity.md)
- [VoxelShape min/max sütun sözleşmesi](docs/shape-axis-limits-at-contract.md)
- [VoxelShape min/max sütun doğrulaması](docs/shape-axis-limits-at-parity.md)
- [VoxelShape move overload sözleşmesi](docs/shape-move-overloads-contract.md)
- [VoxelShape move overload doğrulaması](docs/shape-move-overloads-parity.md)
- [VoxelShape optimize sözleşmesi](docs/shape-optimize-contract.md)
- [VoxelShape optimize doğrulaması](docs/shape-optimize-parity.md)
- [VoxelShape joinUnoptimized sözleşmesi](docs/shape-join-unoptimized-contract.md)
- [VoxelShape join ve joinUnoptimized doğrulaması](docs/shape-join-unoptimized-parity.md)
- [VoxelShape forAllBoxes sözleşmesi](docs/shape-for-all-boxes-contract.md)
- [VoxelShape forAllBoxes doğrulaması](docs/shape-for-all-boxes-parity.md)
- [Shapes.create(AABB) sözleşmesi](docs/shape-create-aabb-contract.md)
- [Shapes.create(AABB) doğrulaması](docs/shape-create-aabb-parity.md)
- [Shapes.create(double...) sözleşmesi](docs/shape-create-coords-contract.md)
- [Shapes.create(double...) doğrulaması](docs/shape-create-coords-parity.md)
- [Shapes.INFINITY sözleşmesi](docs/shape-infinity-contract.md)
- [Shapes.INFINITY doğrulaması](docs/shape-infinity-parity.md)
- [Shapes.EPSILON/BIG_EPSILON sözleşmesi](docs/shape-epsilon-contract.md)
- [Shapes.EPSILON/BIG_EPSILON doğrulaması](docs/shape-epsilon-parity.md)
- [VoxelShape forAllEdges sözleşmesi](docs/shape-edges-contract.md)
- [VoxelShape forAllEdges doğrulaması](docs/shape-edges-parity.md)
- [VoxelShape BooleanOp doğrulaması](docs/shape-boolean-parity.md)
- [VoxelShape collision doğrulaması](docs/shape-collision-parity.md)
- [VoxelShape çoklu collision doğrulaması](docs/shape-collision-many-parity.md)
- [VoxelShape clip sözleşmesi](docs/shape-clip-contract.md)
- [VoxelShape clip doğrulaması](docs/shape-clip-parity.md)
- [VoxelShape closestPointTo sözleşmesi](docs/shape-closest-contract.md)
- [VoxelShape closestPointTo doğrulaması](docs/shape-closest-parity.md)
- [VoxelShape getFaceShape sözleşmesi](docs/shape-face-contract.md)
- [VoxelShape getFaceShape doğrulaması](docs/shape-face-parity.md)
- [Shapes.blockOccludes sözleşmesi](docs/shape-occlusion-contract.md)
- [Shapes.blockOccludes doğrulaması](docs/shape-occlusion-parity.md)
- [mergedFaceOccludes ve faceShapeOccludes sözleşmesi](docs/shape-face-occlusion-contract.md)
- [mergedFaceOccludes ve faceShapeOccludes doğrulaması](docs/shape-face-occlusion-parity.md)
- [Shapes.equal sözleşmesi](docs/shape-equal-contract.md)
- [Shapes.equal doğrulaması](docs/shape-equal-parity.md)
- [VoxelShape equals(Object) sözleşmesi](docs/shape-object-equals-contract.md)
- [VoxelShape equals(Object) doğrulaması](docs/shape-object-equals-parity.md)
- [VoxelShape toString() sözleşmesi](docs/shape-to-string-contract.md)
- [VoxelShape toString() doğrulaması](docs/shape-to-string-parity.md)
- [Shapes.rotate doğrulaması](docs/shape-rotate-parity.md)
- [Shapes.rotate merkez overload sözleşmesi](docs/shape-rotate-center-contract.md)
- [Shapes.rotate merkez overload doğrulaması](docs/shape-rotate-center-parity.md)
- [Shapes.rotateHorizontalAxis sözleşmesi](docs/shape-rotate-horizontal-axis-contract.md)
- [Shapes.rotateHorizontalAxis doğrulaması](docs/shape-rotate-horizontal-axis-parity.md)
- [Shapes.rotateHorizontalAxis merkez overload sözleşmesi](docs/shape-rotate-horizontal-axis-center-contract.md)
- [Shapes.rotateHorizontalAxis merkez overload doğrulaması](docs/shape-rotate-horizontal-axis-center-parity.md)
- [Shapes.rotateAllAxis sözleşmesi](docs/shape-rotate-all-axis-contract.md)
- [Shapes.rotateAllAxis doğrulaması](docs/shape-rotate-all-axis-parity.md)
- [Shapes.rotateAllAxis merkez overload sözleşmesi](docs/shape-rotate-all-axis-center-contract.md)
- [Shapes.rotateAllAxis merkez overload doğrulaması](docs/shape-rotate-all-axis-center-parity.md)
- [Shapes.rotateHorizontal doğrulaması](docs/shape-rotate-horizontal-parity.md)
- [Shapes.rotateAll doğrulaması](docs/shape-rotate-all-parity.md)
- [Shapes.rotateAll merkez overload sözleşmesi](docs/shape-rotate-all-center-contract.md)
- [Shapes.rotateAll merkez overload doğrulaması](docs/shape-rotate-all-center-parity.md)
- [Shapes.rotateAttachFace sözleşmesi](docs/shape-rotate-attach-face-contract.md)
- [Shapes.rotateAttachFace doğrulaması](docs/shape-rotate-attach-face-parity.md)
- [Shapes.rotateAttachFace initial sözleşmesi](docs/shape-rotate-attach-face-initial-contract.md)
- [Shapes.rotateAttachFace initial doğrulaması](docs/shape-rotate-attach-face-initial-parity.md)
- [Shapes.rotateHorizontal initial sözleşmesi](docs/shape-rotate-horizontal-initial-contract.md)
- [Shapes.rotateHorizontal initial doğrulaması](docs/shape-rotate-horizontal-initial-parity.md)
- [Shapes.rotateAll initial sözleşmesi](docs/shape-rotate-all-initial-contract.md)
- [Shapes.rotateAll initial doğrulaması](docs/shape-rotate-all-initial-parity.md)
- [`nextGaussian` sözleşmesi ve referans testleri](docs/random-gaussian-contract.md)
- [`nextGaussian` doğrulaması](docs/random-gaussian-parity.md)
- [`fork` ve positional random sözleşmesi](docs/random-fork-contract.md)
- [`fork` ve positional random doğrulaması](docs/random-fork-parity.md)
- [`setSeed` sözleşmesi](docs/random-set-seed-contract.md)
- [`setSeed` doğrulaması](docs/random-set-seed-parity.md)
- [Random yardımcı overload sözleşmesi](docs/random-overloads-contract.md)
- [Random yardımcı overload doğrulaması](docs/random-overloads-parity.md)
- [Kalan bölümler test planı](docs/remaining-test-plan.md)
- [NBT davranış sözleşmesi ve referans testleri](docs/nbt-contract.md)
- [NBT doğrulaması](docs/nbt-parity.md)
- [UTF-8 doğrulaması](docs/utf8-parity.md)
- [UTF-8 davranış sözleşmesi ve referans testleri](docs/utf8-contract.md)
- [UUID/BlockPos doğrulaması](docs/binary-parity.md)
- [UUID/BlockPos davranış sözleşmesi ve referans testleri](docs/binary-contract.md)
- [ChunkPos stream codec sözleşmesi](docs/chunkpos-contract.md)
- [ChunkPos stream codec doğrulaması](docs/chunkpos-parity.md)

Her AI aracı işe başlamadan `AGENTS.md` dosyasını okumalıdır. Claude, Gemini ve Copilot giriş dosyaları aynı kurallara yönlendirir. Talimat dosyalarını otomatik okumayan araçlarda `AGENTS.md` görev bağlamına açıkça verilmelidir; bu dosyalar modellerin davranışını teknik olarak zorlayan bir mekanizma değildir.
