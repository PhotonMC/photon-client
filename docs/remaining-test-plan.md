# Kalan bölümler test planı (çalıştırılamayanlar)

1515 Java senaryosu kayıtlıdır (beş+bir aile).

P03.b Vec3.STREAM_CODEC 13 senaryoda hazırdır ([sözleşme](vec3-stream-codec-contract.md), [kanıt](vec3-stream-codec-parity.md)). Aşağıdaki bölümler yerel ortamda çalıştırılamaz; her biri engelli kaydedilir, sahte golden üretilmez. Her bölümün önkoşulu, engel kalkınca yazılacak ilk senaryolar ve kabul ölçütü tanımlıdır.
P03.b LP Vec3 stream codec 15 senaryoda hazırdır ([sözleşme](lp-vec3-contract.md), [kanıt](lp-vec3-parity.md)).
P03.b ChunkPos.STREAM_CODEC 20 senaryoda hazırdır ([sözleşme](chunkpos-contract.md), [kanıt](chunkpos-parity.md)).
P01.c ChunkPos.CODEC 12 senaryoda hazırdır ([sözleşme](chunkpos-codec-contract.md), [kanıt](chunkpos-codec-parity.md)).
P01.c ChunkPos blok koordinatı yardımcıları 6 senaryoda hazırdır ([sözleşme](chunkpos-block-coordinates-contract.md), [kanıt](chunkpos-block-coordinates-parity.md)).
P01.c ChunkPos region koordinatı yardımcıları 6 senaryoda hazırdır ([sözleşme](chunkpos-region-coordinates-contract.md), [kanıt](chunkpos-region-coordinates-parity.md)).
P01.c ChunkPos BlockPos yardımcıları 6 senaryoda hazırdır ([sözleşme](chunkpos-block-positions-contract.md), [kanıt](chunkpos-block-positions-parity.md)).
P01.c ChunkPos temsil ve hash 6 senaryoda hazırdır ([sözleşme](chunkpos-representation-contract.md), [kanıt](chunkpos-representation-parity.md)).
P01.c ChunkPos BlockPos'tan containing/pack 6 senaryoda hazırdır ([sözleşme](chunkpos-from-blockpos-contract.md), [kanıt](chunkpos-from-blockpos-parity.md)).
P01.c ChunkPos mesafe yardımcıları 6 senaryoda hazırdır ([sözleşme](chunkpos-distances-contract.md), [kanıt](chunkpos-distances-parity.md)).
P01.c ChunkPos validity yardımcıları 6 senaryoda hazırdır ([sözleşme](chunkpos-validity-contract.md), [kanıt](chunkpos-validity-parity.md)).
P01.c ChunkPos section node dönüşümü 6 senaryoda hazırdır ([sözleşme](chunkpos-from-section-node-contract.md), [kanıt](chunkpos-from-section-node-parity.md)).
P01.c ChunkPos rangeClosed 8 senaryoda hazırdır ([sözleşme](chunkpos-range-closed-contract.md), [kanıt](chunkpos-range-closed-parity.md)).
P01.c ChunkPos public sabitleri 4 senaryoda hazırdır ([sözleşme](chunkpos-constants-contract.md), [kanıt](chunkpos-constants-parity.md)).
P01.c ChunkPos packed koordinat erişimcileri 6 senaryoda hazırdır ([sözleşme](chunkpos-packed-accessors-contract.md), [kanıt](chunkpos-packed-accessors-parity.md)).
P01.c SectionPos packed koordinat erişimcileri 6 senaryoda hazırdır ([sözleşme](sectionpos-packed-accessors-contract.md), [kanıt](sectionpos-packed-accessors-parity.md)).
P01.c SectionPos blok koordinatı yardımcıları 6 senaryoda hazırdır ([sözleşme](sectionpos-block-coordinates-contract.md), [kanıt](sectionpos-block-coordinates-parity.md)).
P01.c SectionPos relative-to-block yardımcıları 6 senaryoda hazırdır ([sözleşme](sectionpos-relative-to-block-contract.md), [kanıt](sectionpos-relative-to-block-parity.md)).
P01.c SectionPos spatial yardımcıları 6 senaryoda hazırdır ([sözleşme](sectionpos-spatial-contract.md), [kanıt](sectionpos-spatial-parity.md)).
P01.c SectionPos packed dönüşümleri 6 senaryoda hazırdır ([sözleşme](sectionpos-packed-transformations-contract.md), [kanıt](sectionpos-packed-transformations-parity.md)).

P01.b Vec3 `CODEC` 12 senaryoda hazırdır ([sözleşme](vec3-codec-contract.md), [kanıt](vec3-codec-parity.md)).

## Kural

- Engelli test başarı sayılmaz; varsayılan koşu engeli gizlemez.
- Ortam hazır olunca ilgili bölüm için önce Java referansı ve testler hazırlanır (davranış testi belgesindeki sıra değişmez).
- Bu dosya kapsam defteridir; hiçbir satırı eşitlik kanıtı değildir.

## B — Bağlantı ve istemci dünyası

| Bölüm | Engel | İlk senaryolar (engel kalkınca) |
| --- | --- | --- |
| P02.b döngü (10 tick sınırı) | `Minecraft.runTick` tam istemci gerektirir | Büyük zaman artışında üretilen tick'e karşı yürütülen tick (en çok 10), kuyruk/paket/render sırası |
| P03.d registry adaptör sınırı | Registry verisi P07'ye bağlı | Bilinmeyen registry girdisi, eksik tag, phase sınırı; adaptörün reddetme davranışı |
| P04 kaynaklar/pack'ler | Asset index yerelde var ama pack reload akışı oyun dizini ister | Aynı yola iki pack, olmayan nesne, geçersiz index, reload sırasında değişim; çözülen kimlik/hash ve öncelik |
| P05 ayarlar/input | Pencere/odak/GUI koordinatı gerçek istemci ister | Karşıt yön sıfırlama, tuş tutma/tekrar, odak kaybı; input snapshot ve hareket vektörü |
| P06 bağlantı/login | Çalışan Pumpkin sunucusu yok | Parçalı paket, sıkıştırma eşiği, kopma; paket baytları ve durum sırası |
| P07 configuration/registry | P06 + P04'e bağlı | Registry/tag biriktirme, eksik girdide ack'siz bekleme, play'e geçiş anı |
| P08 dünya/chunk | P07'ye bağlı | Palette genişlik değişimi, boş section, load-update-unload sırası |

## C — His, görüntü, ses

| Bölüm | Engel | İlk senaryolar (engel kalkınca) |
| --- | --- | --- |
| P09 çarpışma | P08 dünyası yok | Slab/merdiven/dar köşe/düşük tavan; çözülen delta, on-ground ve step yüksekliği |
| P10 yerel hareket | P05 + P08 + P09'a bağlı | Tick başına konum/hız/pose; diyagonal, sprint çift basışı, su/buz/merdiven |
| P11 senkronizasyon | P06 + P10'a bağlı | Eşik altı/üstü hareket paketleri, 20 tick hatırlatması, teleport düzeltmesi |
| P12 hedefleme/eylem | P09–P11'e bağlı | Hit sonucu, kazma iptali, reddedilen yerleştirme, ack/rollback sırası |
| P13 envanter | P07 + P12'ye bağlı | Shift-click, drag iptali, yanlış state, sunucu düzeltmesi |
| P14 kamera | P05 + P10'a bağlı | Aynı mouse izi farklı frame bölünmesinde; FOV/pose ve zamanlama izi |
| P15–P19 görüntü/animasyon/partikül | GPU + kontrollü frame yakalama yok | Sabit sahnede geometri/ışık verisi ve görüntü; seed sabitken parçacık listesi |
| P20 ses | Ses yakalama + oyun tetikleri yok | Kaynak seçimi, zaman damgaları, pause kategorileri; aynı dosya seçimi eşitlik değildir |
| P21–P23 yazı/GUI/chat | P15 + P04 + P06'ya bağlı | Glyph/wrap/bidi ölçüler, focus/olay tüketimi, ack/disconnect sırası |
| P24–P26 kapsam/dağıtım | Yukarıdakilerin tamamına bağlı | Atanmamış davranış listesi, kontrollü oturum, platform matrisi |

## P01 kalanları (çalıştırılabilir, sonraki test dilimleri)

- P01.b Vec3 add/subtract overload'ları 5 senaryoda hazırdır ([sözleşme](vec3-add-subtract-contract.md), [kanıt](vec3-add-subtract-parity.md)).
- P01.b Vec3 vectorTo 5 senaryoda hazırdır ([sözleşme](vec3-vector-to-contract.md), [kanıt](vec3-vector-to-parity.md)).
- P01.b Vec3 dot 5 senaryoda hazırdır ([sözleşme](vec3-dot-contract.md), [kanıt](vec3-dot-parity.md)).
- P01.b Vec3 cross 5 senaryoda hazırdır ([sözleşme](vec3-cross-contract.md), [kanıt](vec3-cross-parity.md)).
- P01.b Vec3 distanceTo 5 senaryoda hazırdır ([sözleşme](vec3-distance-to-contract.md), [kanıt](vec3-distance-to-parity.md)).
- P01.b Vec3 distanceToSqr overload'ları 5 senaryoda hazırdır ([sözleşme](vec3-distance-to-sqr-contract.md), [kanıt](vec3-distance-to-sqr-parity.md)).
- P01.b Vec3 multiply overload'ları 5 senaryoda hazırdır ([sözleşme](vec3-multiply-contract.md), [kanıt](vec3-multiply-parity.md)).
- P01.b Vec3 horizontal 5 senaryoda hazırdır ([sözleşme](vec3-horizontal-contract.md), [kanıt](vec3-horizontal-parity.md)).
- P01.b Vec3 horizontalDistance ve horizontalDistanceSqr 5 senaryoda hazırdır ([sözleşme](vec3-horizontal-distances-contract.md), [kanıt](vec3-horizontal-distances-parity.md)).
- P01.b Vec3 closerThan overload'ları 5 senaryoda hazırdır ([sözleşme](vec3-closer-than-contract.md), [kanıt](vec3-closer-than-parity.md)).
- P01.b Vec3 reverse 5 senaryoda hazırdır ([sözleşme](vec3-reverse-contract.md), [kanıt](vec3-reverse-parity.md)).
- P01.b Vec3 offsetRandom ve offsetRandomXZ 5 senaryoda hazırdır ([sözleşme](vec3-offset-random-contract.md), [kanıt](vec3-offset-random-parity.md)).
- P01.b Vec3 lerp 5 senaryoda hazırdır ([sözleşme](vec3-lerp-contract.md), [kanıt](vec3-lerp-parity.md)).
- P01.b Vec3 rotateClockwise90 5 senaryoda hazırdır ([sözleşme](vec3-rotate-clockwise-90-contract.md), [kanıt](vec3-rotate-clockwise-90-parity.md)).
- P01.b Vec3 isFinite 5 senaryoda hazırdır ([sözleşme](vec3-is-finite-contract.md), [kanıt](vec3-is-finite-parity.md)).
- P01.b Vec3 get 5 senaryoda hazırdır ([sözleşme](vec3-get-contract.md), [kanıt](vec3-get-parity.md)).
- P01.b Vec3 with 5 senaryoda hazırdır ([sözleşme](vec3-with-contract.md), [kanıt](vec3-with-parity.md)).
- P01.b Vec3 relative 6 senaryoda hazırdır ([sözleşme](vec3-relative-contract.md), [kanıt](vec3-relative-parity.md)).
- P01.b Vec3 align 6 senaryoda hazırdır ([sözleşme](vec3-align-contract.md), [kanıt](vec3-align-parity.md)).
- P01.b Vec3 projectedOn 6 senaryoda hazırdır ([sözleşme](vec3-projected-on-contract.md), [kanıt](vec3-projected-on-parity.md)).
- P01.b Vec3 toVector3f 6 senaryoda hazırdır ([sözleşme](vec3-to-vector3f-contract.md), [kanıt](vec3-to-vector3f-parity.md)).
- P01.b Vec3 koordinat yardımcıları 6 senaryoda hazırdır ([sözleşme](vec3-coordinates-contract.md), [kanıt](vec3-coordinates-parity.md)).
- P01.b Vec3 equals/hashCode 6 senaryoda hazırdır ([sözleşme](vec3-equality-contract.md), [kanıt](vec3-equality-parity.md)).
- P01.b Vec3 toString 6 senaryoda hazırdır ([sözleşme](vec3-to-string-contract.md), [kanıt](vec3-to-string-parity.md)).
- P01.b Vec3 xRot/yRot/zRot 18 senaryoda hazırdır ([sözleşme](vec3-rotation-contract.md), [kanıt](vec3-rotation-parity.md)).
- P01.b Vec3 directionFromRotation overload'ları 18 senaryoda hazırdır ([sözleşme](vec3-direction-from-rotation-contract.md), [kanıt](vec3-direction-from-rotation-parity.md)).
- P01.b Vec3 rotation() 18 senaryoda hazırdır ([sözleşme](vec3-rotation-method-contract.md), [kanıt](vec3-rotation-method-parity.md)).
- P01.b Vec2 rotate(double) 18 senaryoda hazırdır ([sözleşme](vec2-rotate-contract.md), [kanıt](vec2-rotate-parity.md)).
- P01.b Vec2 dot(Vec2) 5 senaryoda hazırdır ([sözleşme](vec2-dot-contract.md), [kanıt](vec2-dot-parity.md)).
- P01.b Vec2 add(Vec2) ve add(float) overload'ları 5 senaryoda hazırdır ([sözleşme](vec2-add-contract.md), [kanıt](vec2-add-parity.md)).
- P01.b Vec2 distanceToSqr(Vec2) 5 senaryoda hazırdır ([sözleşme](vec2-distance-to-sqr-contract.md), [kanıt](vec2-distance-to-sqr-parity.md)).
- P01.b Vec2 negated() 5 senaryoda hazırdır ([sözleşme](vec2-negated-contract.md), [kanıt](vec2-negated-parity.md)).
- P01.b Vec2 equals(Object)/hashCode() 6 senaryoda hazırdır ([sözleşme](vec2-equality-contract.md), [kanıt](vec2-equality-parity.md)).
- P01.b Vec2 public sabitleri 8 senaryoda hazırdır ([sözleşme](vec2-constants-contract.md), [kanıt](vec2-constants-parity.md)).
- P01.b Vec3 public sabitleri 4 senaryoda hazırdır ([sözleşme](vec3-constants-contract.md), [kanıt](vec3-constants-parity.md)).
- P01.b Vec3 local coordinate yardımcıları 6 senaryoda hazırdır ([sözleşme](vec3-local-coordinates-contract.md), [kanıt](vec3-local-coordinates-parity.md)).
- P01.b Vec3 `CODEC` 12 senaryoda hazırdır ([sözleşme](vec3-codec-contract.md), [kanıt](vec3-codec-parity.md)).
- P01.d: VoxelShape temel/OR, varargs OR, singleEncompassing, min/max, min/max sütun sorguları, move overload'ları, optimize, getCoords, forAllBoxes, Shapes.create(AABB), Shapes.create(double...), Shapes.INFINITY, Shapes.EPSILON/BIG_EPSILON ve forAllEdges ve `Shapes.join` tarafından kabul edilen `FALSE`, `AND`, `ONLY_FIRST`, `ONLY_SECOND`, `NOT_SAME`, `FIRST` ve `SECOND` yolları hazır; tekli ve çoklu shape collision de hazır ([tekli sözleşme](shape-collision-contract.md), [çoklu sözleşme](shape-collision-many-contract.md)); VoxelShape clip de hazır ([sözleşme](shape-clip-contract.md)); VoxelShape closestPointTo da hazır ([sözleşme](shape-closest-contract.md)); VoxelShape getFaceShape da hazır ([sözleşme](shape-face-contract.md)); Shapes.blockOccludes da hazır ([sözleşme](shape-occlusion-contract.md)); mergedFaceOccludes ve faceShapeOccludes da hazır ([sözleşme](shape-face-occlusion-contract.md)); `Shapes.equal` de hazır ([sözleşme](shape-equal-contract.md)); VoxelShape equals(Object) de hazır ([sözleşme](shape-object-equals-contract.md), [kanıt](shape-object-equals-parity.md)); VoxelShape toString() de hazır ([sözleşme](shape-to-string-contract.md), [kanıt](shape-to-string-parity.md)); `Shapes.rotate` de hazır ([sözleşme](shape-rotate-contract.md)); `Shapes.rotate` merkez overload'ı da hazır ([sözleşme](shape-rotate-center-contract.md), [kanıt](shape-rotate-center-parity.md)); `Shapes.rotateHorizontalAxis` de hazır ([sözleşme](shape-rotate-horizontal-axis-contract.md)); merkez overload'ı da hazır ([sözleşme](shape-rotate-horizontal-axis-center-contract.md), [kanıt](shape-rotate-horizontal-axis-center-parity.md)); `Shapes.rotateAllAxis` de hazır ([sözleşme](shape-rotate-all-axis-contract.md)); merkez overload'ı da hazır ([sözleşme](shape-rotate-all-axis-center-contract.md), [kanıt](shape-rotate-all-axis-center-parity.md)); `Shapes.rotateHorizontal` de hazır ([sözleşme](shape-rotate-horizontal-contract.md)); `Shapes.rotateAll` de hazır ([sözleşme](shape-rotate-all-contract.md)); `Shapes.rotateAll` merkez overload'ı da hazır ([sözleşme](shape-rotate-all-center-contract.md), [kanıt](shape-rotate-all-center-parity.md)); Shapes.rotateAttachFace varsayılan overload'ı da hazır ([sözleşme](shape-rotate-attach-face-contract.md), [kanıt](shape-rotate-attach-face-parity.md)); Shapes.rotateHorizontal desteklenen initial overload'ı da hazır ([sözleşme](shape-rotate-horizontal-initial-contract.md), [kanıt](shape-rotate-horizontal-initial-parity.md)); Shapes.rotateAll desteklenen initial overload'ı da hazır ([sözleşme](shape-rotate-all-initial-contract.md), [kanıt](shape-rotate-all-initial-parity.md)); Shapes.rotateAttachFace desteklenen initial overload'ı da hazır ([sözleşme](shape-rotate-attach-face-initial-contract.md), [kanıt](shape-rotate-attach-face-initial-parity.md)); AABB contract, expandTowards(Vec3), equals/hashCode, toString(), Builder, deflate üç eksenli overload'u, inflate, intersect, predicate overload'ları, collidedAlongVector, kutu kurucuları ve üreticileri, BoundingBox köprüsü, minmax, move, move overload'ları, sizes, hasNaN, positions, axis min/max, distanceToSqr, ofSize, endpoint setter'ları, koordinat clip ve iterable clip/BlockHitResult overload'ları da hazır ([contract sözleşmesi](aabb-contract.md), [expandTowards(Vec3) sözleşmesi](aabb-expand-overload-contract.md), [expandTowards(Vec3) kanıtı](aabb-expand-overload-parity.md), [equals/hashCode sözleşmesi](aabb-equality-contract.md), [equals/hashCode kanıtı](aabb-equality-parity.md), [toString sözleşmesi](aabb-to-string-contract.md), [toString kanıtı](aabb-to-string-parity.md), [Builder sözleşmesi](aabb-builder-contract.md), [Builder kanıtı](aabb-builder-parity.md), [deflate üç eksenli overload sözleşmesi](aabb-deflate-overload-contract.md), [deflate üç eksenli overload kanıtı](aabb-deflate-overload-parity.md), [inflate sözleşmesi](aabb-inflate-contract.md), [intersect sözleşmesi](aabb-intersect-contract.md), [predicate overload sözleşmesi](aabb-predicate-overloads-contract.md), [collidedAlongVector sözleşmesi](aabb-collided-along-vector-contract.md), [kutu kurucuları sözleşmesi](aabb-constructors-contract.md), [BoundingBox köprüsü sözleşmesi](aabb-bounding-box-contract.md), [minmax sözleşmesi](aabb-minmax-contract.md), [move sözleşmesi](aabb-move-contract.md), [move overload sözleşmesi](aabb-move-overloads-contract.md), [sizes sözleşmesi](aabb-sizes-contract.md), [hasNaN sözleşmesi](aabb-has-nan-contract.md), [positions sözleşmesi](aabb-positions-contract.md), [axis min/max sözleşmesi](aabb-axis-limits-contract.md), [distanceToSqr sözleşmesi](aabb-distance-contract.md), [ofSize sözleşmesi](aabb-of-size-contract.md), [endpoint setter sözleşmesi](aabb-set-endpoint-contract.md), [iterable clip sözleşmesi](aabb-clip-many-contract.md)); kalan AABB çoklu shape birleşimleri ile diğer merger dalları ve world face/collider toplama sonraki dilimlerdir. `false,false` için true üreten BooleanOp sabitleri Java `Shapes.join` tarafından reddedilir. Bu turdaki contains/contains_vec3/intersects/intersects_coords/intersects_vec3/intersects_block_pos/collided_along_vector/from_block_pos/from_vec3/unit_cube_from_lower_corner/encapsulating_full_blocks/of_bounding_box/intersect/minmax/move/move overload'ları/sizes/has_nan/positions/min/max/distance_point/distance_box/of_size/set_min_x/set_min_y/set_min_z/set_max_x/set_max_y/set_max_z/expand/expand_vec3/equals/hash_code/to_string/deflate_xyz/deflate ve tekli/iterable `clip`/`clip_coords` de hazır.
- P01.e: `fork`/`forkPositional`, `nextGaussian`, `setSeed`, random yardımcı overload'ları ve bu turdaki sıralı çekimler hazır; farklı kaynak türleri ve overflow hata yolları ayrı kapsamdır.
