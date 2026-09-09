# İlk davranış testleri: implementasyondan önceki sözleşmeler

Bu belge P00, P01.a, P02.a ve P03.a için çalıştırılabilir testlere dönüştürülecek ilk sözleşmeleri tanımlar. [Kalıcı çalıştırıcı](behavior-runner.md) ile 180 sayısal/vektör, 22 zamanlayıcı ve 144 wire senaryosu hazır. Bu ilk corpus tam P01/P02/P03 kapsamı değildir; P03.a Rust wire dilimi 144 senaryoda doğrulandı; [sonuç](wire-parity.md). P02.a Rust zamanlayıcısı da 22 senaryoda doğrulandı; [sonuç](timer-parity.md). P01.a Vec2/sayısal yardımcıları da 180 senaryoda doğrulandı; [sonuç](math-parity.md). Sonraki P03.b UTF-8 için 141 senaryo ve ayrı [sözleşme](utf8-contract.md) hazır ve Rust karşılığı doğrulandı ([kanıt](utf8-parity.md)). P03.b binary için 116 senaryo ve ayrı [sözleşme](binary-contract.md) hazır ve Rust karşılığı doğrulandı ([kanıt](binary-parity.md)). P01.b Vec3 için 40 senaryo ve ayrı [sözleşme](vec3-contract.md) hazır ve Rust karşılığı doğrulandı ([kanıt](math-parity.md)); P01.c koordinat dönüşümleri için 57 senaryo ve ayrı [sözleşme](coords-contract.md) hazır; P01.c BlockPos/section-relative için 10 senaryo ve ayrı [sözleşme](blockpos-contract.md) hazır, Rust karşılığı [kanıtlandı](blockpos-parity.md); P01.d AABB contract için 5 senaryo ve ayrı [sözleşme](aabb-contract.md) hazır, Rust karşılığı [kanıtlandı](aabb-parity.md); P01.d AABB inflate için 5 senaryo ve ayrı [sözleşme](aabb-inflate-contract.md) hazır, Rust karşılığı [kanıtlandı](aabb-inflate-parity.md); P01.d AABB intersect için 6 senaryo ve ayrı [sözleşme](aabb-intersect-contract.md) hazır, Rust karşılığı [kanıtlandı](aabb-intersect-parity.md); P01.d AABB predicate overload'ları için 6 senaryo ve ayrı [sözleşme](aabb-predicate-overloads-contract.md) hazır, Rust karşılığı [kanıtlandı](aabb-predicate-overloads-parity.md); P01.d AABB collidedAlongVector için 8 senaryo ve ayrı [sözleşme](aabb-collided-along-vector-contract.md) hazır, Rust karşılığı [kanıtlandı](aabb-collided-along-vector-parity.md); P01.d AABB kutu kurucuları ve üreticileri için 6 senaryo ve ayrı [sözleşme](aabb-constructors-contract.md) hazır, Rust karşılığı [kanıtlandı](aabb-constructors-parity.md); P01.d AABB BoundingBox köprüsü için 6 senaryo ve ayrı [sözleşme](aabb-bounding-box-contract.md) hazır, Rust karşılığı [kanıtlandı](aabb-bounding-box-parity.md); P01.d AABB minmax için 6 senaryo ve ayrı [sözleşme](aabb-minmax-contract.md) hazır, Rust karşılığı [kanıtlandı](aabb-minmax-parity.md); P01.d AABB move için 6 senaryo ve ayrı [sözleşme](aabb-move-contract.md) hazır, Rust karşılığı [kanıtlandı](aabb-move-parity.md); P01.d AABB move overload'ları için 6 senaryo ve ayrı [sözleşme](aabb-move-overloads-contract.md) hazır, Rust karşılığı [kanıtlandı](aabb-move-overloads-parity.md); P01.d AABB sizes için 6 senaryo ve ayrı [sözleşme](aabb-sizes-contract.md) hazır, Rust karşılığı [kanıtlandı](aabb-sizes-parity.md); P01.d AABB hasNaN için 6 senaryo ve ayrı [sözleşme](aabb-has-nan-contract.md) hazır, Rust karşılığı [kanıtlandı](aabb-has-nan-parity.md); P01.d AABB positions için 6 senaryo ve ayrı [sözleşme](aabb-positions-contract.md) hazır, Rust karşılığı [kanıtlandı](aabb-positions-parity.md); P01.d AABB axis min/max için 6 senaryo ve ayrı [sözleşme](aabb-axis-limits-contract.md) hazır, Rust karşılığı [kanıtlandı](aabb-axis-limits-parity.md); P01.d AABB distanceToSqr için 6 senaryo ve ayrı [sözleşme](aabb-distance-contract.md) hazır, Rust karşılığı [kanıtlandı](aabb-distance-parity.md); P01.d AABB ofSize için 6 senaryo ve ayrı [sözleşme](aabb-of-size-contract.md) hazır, Rust karşılığı [kanıtlandı](aabb-of-size-parity.md); P01.d AABB endpoint setter'ları için 6 senaryo ve ayrı [sözleşme](aabb-set-endpoint-contract.md) hazır, Rust karşılığı [kanıtlandı](aabb-set-endpoint-parity.md); P01.d AABB clip için 6 senaryo ve ayrı [sözleşme](aabb-clip-contract.md) hazır, Rust karşılığı [kanıtlandı](aabb-clip-parity.md); P01.d AABB koordinat clip overloadu için 6 senaryo ve ayrı [sözleşme](aabb-clip-coordinates-contract.md) hazır, Rust karşılığı [kanıtlandı](aabb-clip-coordinates-parity.md); P01.d AABB iterable clip ve BlockHitResult için 9 senaryo ve ayrı [sözleşme](aabb-clip-many-contract.md) hazır, Rust karşılığı [kanıtlandı](aabb-clip-many-parity.md); P01.d VoxelShape temel/OR için 5 senaryo ve ayrı [sözleşme](shape-contract.md) hazır, Rust karşılığı [kanıtlandı](shape-parity.md); P01.d VoxelShape BooleanOp join için 8 senaryo ve ayrı [sözleşme](shape-boolean-contract.md) hazır, Rust karşılığı [kanıtlandı](shape-boolean-parity.md); P01.d tek VoxelShape collision için 8 senaryo ve ayrı [sözleşme](shape-collision-contract.md), çoklu shape collision için 5 senaryo ve ayrı [sözleşme](shape-collision-many-contract.md) hazır; Rust karşılıkları [tekli](shape-collision-parity.md) ve [çoklu](shape-collision-many-parity.md) kanıtlarda doğrulandı; P01.d VoxelShape clip için 6 senaryo ve ayrı [sözleşme](shape-clip-contract.md) hazır, Rust karşılığı [kanıtlandı](shape-clip-parity.md); P01.d VoxelShape closestPointTo için 6 senaryo ve ayrı [sözleşme](shape-closest-contract.md) hazır, Rust karşılığı [kanıtlandı](shape-closest-parity.md); P01.d VoxelShape getFaceShape için 8 senaryo ve ayrı [sözleşme](shape-face-contract.md) hazır, Rust karşılığı [kanıtlandı](shape-face-parity.md); P01.d Shapes.blockOccludes için 8 senaryo ve ayrı [sözleşme](shape-occlusion-contract.md) hazır, Rust karşılığı [kanıtlandı](shape-occlusion-parity.md); P01.d mergedFaceOccludes ve faceShapeOccludes için 10 senaryo ve ayrı [sözleşme](shape-face-occlusion-contract.md) hazır, Rust karşılığı [kanıtlandı](shape-face-occlusion-parity.md); P01.d Shapes.equal için 6 senaryo ve ayrı [sözleşme](shape-equal-contract.md) hazır, Rust karşılığı [kanıtlandı](shape-equal-parity.md); P01.d Shapes.rotate için 12 senaryo ve ayrı [sözleşme](shape-rotate-contract.md) hazır, Rust karşılığı [kanıtlandı](shape-rotate-parity.md); P01.d Shapes.rotate merkez overload için 4 senaryo ve ayrı [sözleşme](shape-rotate-center-contract.md) hazır, Rust karşılığı [kanıtlandı](shape-rotate-center-parity.md); P01.d Shapes.rotateHorizontalAxis için 3 senaryo ve ayrı [sözleşme](shape-rotate-horizontal-axis-contract.md) hazır, Rust karşılığı [kanıtlandı](shape-rotate-horizontal-axis-parity.md); P01.d Shapes.rotateAllAxis için 3 senaryo ve ayrı [sözleşme](shape-rotate-all-axis-contract.md) hazır, Rust karşılığı [kanıtlandı](shape-rotate-all-axis-parity.md); P01.d Shapes.rotateHorizontal için 3 senaryo ve ayrı [sözleşme](shape-rotate-horizontal-contract.md) hazır, Rust karşılığı [kanıtlandı](shape-rotate-horizontal-parity.md); P01.d Shapes.rotateAll için 3 senaryo ve ayrı [sözleşme](shape-rotate-all-contract.md) hazır, Rust karşılığı [kanıtlandı](shape-rotate-all-parity.md); P01.d Shapes.rotateAll merkez overload için 3 senaryo ve ayrı [sözleşme](shape-rotate-all-center-contract.md) hazır, Rust karşılığı [kanıtlandı](shape-rotate-all-center-parity.md); P01.d/e random/AABB için 12 senaryo ve ayrı [sözleşme](random-aabb-contract.md) hazır, Rust karşılığı [kanıtlandı](random-aabb-parity.md); P01.e nextGaussian için 6 senaryo ve ayrı [sözleşme](random-gaussian-contract.md) hazır, Rust karşılığı [kanıtlandı](random-gaussian-parity.md); P01.e fork/forkPositional için 5 senaryo ve ayrı [sözleşme](random-fork-contract.md) hazır, Rust karşılığı [kanıtlandı](random-fork-parity.md); P01.e setSeed için 3 senaryo ve ayrı [sözleşme](random-set-seed-contract.md) hazır, Rust karşılığı [kanıtlandı](random-set-seed-parity.md); P01.e random yardımcı overload'ları için 3 senaryo ve ayrı [sözleşme](random-overloads-contract.md) hazır, Rust karşılığı [kanıtlandı](random-overloads-parity.md); P03.c NBT için 31 senaryo ve ayrı [sözleşme](nbt-contract.md) hazır, Rust karşılığı [kanıtlandı](nbt-parity.md); toplam 1013 Java senaryosu mevcut. Referans sürümü [envanterde](reference-baseline.md) sabitlenmiştir.

## P00 — Java referansı ve karşılaştırıcı

Java tarafında yeni oyun algoritmaları yazmak yerine `.data/minecraft/runtime/client-26.2.jar` içindeki sınıflar çağrılacak. Küçük, özgün Java adaptörü yalnızca girdi kurma, çağrı ve gözlem yapacak. Decompile edilmiş algoritmayı test koduna kopyalamak bağımsız referans oluşturmaz.

Yerel yoklama, `DeltaTracker.Timer`, `Vec2` ve `VarInt` çağrılarının mevcut Java 25 ve `runtime/libs/*` ile grafik arayüzsüz çalışabildiğini gösterdi. İlk kalıcı çalıştırıcı bu yolu kullanabilir. Java wrapper'ı depoda tutulabilir; orijinal JAR, bağımlılıklar, derlenen wrapper çıktısı ve Java sonuçları `.data/` içinde kalır.

Çalıştırıcıda ayrı işlemler bulunmalı:

1. **Ön kontrol:** JDK, referans kimliği ve gereken dosyaları doğrula. Her test ailesi yalnızca ihtiyaç duyduğu girdileri zorunlu tutsun; VarInt testi için GPU/asset gerektirme.
2. **Referans yakalama:** Özgün senaryo girdisini Java'ya ver, sürüm/ortamla birlikte çıktıyı yerelde kaydet. Aynı senaryoyu tekrar çalıştırıp kararlılığı kontrol et. Bu işlemin başarılı olması Rust eşitliğinin geçtiği anlamına gelmez.
3. **Karşılaştırma:** Aynı girdiyi Rust adaptörüne ver; çıktıları Java referansıyla karşılaştır. Eksik adaptör, eksik golden veya yanlış sürüm başarı üretemez. Eşitlik komutu çalıştırılmamış zorunlu senaryoları başarılı çıkışla gizlememeli.
4. **Fark raporu:** İlk ayrışan olay/alanı, beklenen ve gerçekleşen değeri, referans kimliğini ve aynı senaryoyu yeniden çalıştırma bilgisini `.data/runs/` altına yaz.

Dil bağımsız senaryo/sonuç kaydı şu bilgileri taşımalı; mevcut uygulama JSONL kullanır; operasyon alanları [çalıştırıcı belgesinde](behavior-runner.md) tanımlıdır:

| Alan | Anlam |
| --- | --- |
| `schema_version`, `case_id`, `operation` | Kayıt şeması ve sabit davranış kimliği |
| `reference_id` | Client JAR, ilgili kaynak/asset seti, JDK kimliği |
| `initial_state`, `inputs` | Başlangıç durumu ve sıralı girdiler |
| `clock`, `seed`, `environment` | Gereken senaryolarda kontrollü koşullar |
| `events`, `outputs` | Sırası korunmuş gözlemler ve son durum |
| `outcome` | Değer/normal dönüş veya gözlenen hata |

64 bit tamsayılar ve float değerler taşıma formatında hassasiyet kaybetmemeli. `float`/`double` için ham IEEE bit gösterimi kullanılmalı; `NaN`, sonsuzluk ve işaretli sıfır JSON sayısına zorlanmamalı. İnsan tarafından okunabilir ondalık gösterim yardımcı olabilir, karşılaştırmanın yerini alamaz. Koleksiyon sırası davranışsa normalize edilmemeli.

P00'ın gerekli doğrulamaları: sabit senaryonun tekrarı; kasıtlı değiştirilmiş tek alanın yakalanması; eksik/yanlış referansın reddi; Rust yokken eşitlik iddiası üretilmemesi. Bu birkaç kontrol dışında genel test framework'ü için kapsamlı test yazımı yapılmaz. Golden yenileme ayrı ve açık işlem olmalı; normal test koşusu beklenen sonuçların üstüne yazmamalı.

## P01.a — Kullanılan sayısal işlemler ve Vec2

Kaynak: `net/minecraft/world/phys/Vec2.java` ve `net/minecraft/util/Mth.java`. Başlangıçta `Vec2.normalized/scale/length/lengthSquared` ile sonraki dilimlerde gereken `floor/clamp/wrapDegrees` overload'ları ele alınacak. İlgisiz tüm yardımcılar topluca taşınmayacak.

| Davranış ailesi | Girdiler | Gözlem ve kabul |
| --- | --- | --- |
| V2-NORMALIZE | Sıfır, her eksendeki iki yön, dört diyagonal, küçük/çok küçük ve büyük uzunluklar | İki bileşenin ham float bitleri, gerekirse uzunluk; Java ile kesin eşitlik |
| V2-OPERATIONS | Sıfır/negatif/pozitif çarpan, farklı büyüklükler; normalizasyonla ardışık uygulama | Her işlem sonrası bileşen/uzunluk bitleri; işlem sırası korunur |
| MTH-FLOOR | Pozitif/negatif tamsayılar ve hemen alt/üst komşuları | Aynı overload'dan tamsayı sonuç; özel değerlerin kapsamı Java çağrısıyla belirlenir |
| MTH-ANGLE | Sarma sınırları, hemen komşuları, çok turlu açılar ve işaretli sıfır | Java overload'ıyla aynı değer/bit sonucu |
| MTH-CLAMP | Aralık içi/dışı ve sınır değerleri; çağrı yerlerinde geçerli aralıklar | Aynı değer ve float bitleri; ters aralık davranışı ancak çağrı kapsamı gerektiriyorsa ayrı |

Test tablosu, Rust algoritmasını tekrar yazan hesaplamalarla beklenen değer üretmez. Özgün girdiler depoya girebilir; Java çıktı dosyaları yerelde kalır. Float→double yükseltme veya farklı işlem gruplama nedeniyle oluşan küçük fark otomatik toleransla kabul edilmez.

## P02.a — DeltaTracker.Timer

Kaynak: `net/minecraft/client/DeltaTracker.java`, `Timer.advanceGameTime/advanceRealTime/updatePauseState/updateFrozenState` ve üç getter. Adaptör, başlangıç zamanı ile `targetMsptProvider` girdisini kontrol edebilir; gerçek sistem saatini beklemesi gerekmez.

| Davranış ailesi | Sıralı senaryo | Gözlem ve kabul |
| --- | --- | --- |
| TIME-BOUNDARY | Aynı başlangıçtan 0, 1, 49, 50, 51 ms ve ardışık küçük artışlar | Dönen tick sayısı, delta ve artık tick bitleri |
| TIME-RESIDUAL | Birikmiş kesirli zamanı birkaç çağrıda tam tick'e taşıma | Her çağrı sonrası çıktı; yalnızca toplam tick karşılaştırması yeterli değil |
| TIME-RATE | Kontrollü sağlayıcıyla farklı hedef tick süreleri ve süre değişimi | Oyun zamanı ile gerçek zaman çıktılarının ayrı eşitliği |
| TIME-PAUSE | Artık zaman varken pause, tekrarlı pause, ilerletme, unpause | Pause öncesi saklanan artık ve getter sonuçları; çağrı sırası korunur |
| TIME-FREEZE | Freeze aç/kapat; getter'a iki farklı `ignoreFrozenGame` değeri | Aynı frozen/paused kombinasyonunda Java çıktıları |
| TIME-REAL | Gerçek zaman delta sınırının altı/eşiti/üstü ve uzun frame | Gerçek zaman getter'ının özel dalı dahil bit eşitliği |

Timer testinin büyük zaman artışında ürettiği tick sayısına, `Minecraft.runTick` içindeki 10 tick yürütme sınırı uygulanmaz. Bu sınır ayrı P02.b döngü testidir. Ayrıca zamanlayıcının pause API'si, multiplayer dünyasının gerçekten durduğu anlamına gelmez.

## P03.a — VarInt ve VarLong

Kaynak: `net/minecraft/network/VarInt.java` ve `VarLong.java`. Java adaptörü gerçek Netty buffer üzerinden `getByteSize`, `write`, `read` çağırır. Codec algoritmasını adaptörde yeniden yazmaz.

| Davranış ailesi | Girdiler | Gözlem ve kabul |
| --- | --- | --- |
| VAR-WRITE | 0, 1, -1, işaretli tür min/max ve her 7 bit uzunluk sınırının komşuları | Java'nın yazdığı baytlar ve `getByteSize` sonucu |
| VAR-READ | Java'nın yazdığı geçerli diziler ve bağımsız sentetik bayt girdileri | Okunan değer, tüketilen bayt sayısı ve kalan veri |
| VAR-TRUNCATED | Boş buffer, tamamlanmayan devam bitleri ve her kesme konumu | Başarı/hata, Java exception kategorisi ve hataya kadar tüketim |
| VAR-OVERLONG | VarInt için 5, VarLong için 10 bayt sınırının çevresi ve daha uzun devam dizileri | Java'nın kabul/red davranışı ve cursor; daha katı decoder varsayma |
| VAR-NONCANONICAL | Gereksiz uzun temsil, son bayttaki sınır bitleri ve arkasına eklenen veri | Java'nın gerçek sonucu; “geçersiz olmalı” varsayımıyla oracle'ı değiştirme |

Java exception sınıfı referans kaydında tutulur; Rust'ın aynı dil sınıfını üretmesi beklenmez. Rust adaptörü hata kategorisini eşleyecek; protokolü/kullanıcı sonucunu etkileyen hata davranışı ve tüketim korunacak. Java stack trace metni eşitlik ölçütü değildir.

## Sonraki testlerin otomasyon yolu

Bu ilk saf işlemler grafik arayüzsüz doğrudan çağrılabilir. `LocalPlayer`, dünya, GUI ve renderer testleri aynı derecede bağımsız değildir. İlgili bölümde şu sınırlar kullanılacak:

- Dünya/input/packet senaryolarında Java client'ın gerçek mantığı çalıştırılır; başlangıç durumu ve olay girdileri kontrollü kurulur. Test için olay besleyen adaptör üretim sunucusu değildir. Referans mantığını basitleştiren sahte `LocalPlayer` beklenen sonuç kaynağı olamaz.
- Gerekirse özgün bir Java instrumentasyon adaptörü kullanılır; gözlem ve kontrollü girdi sınırları belgelenir. Decompile algoritmayı yeniden yazmak veya oyun sonucunu değiştiren hook kullanmak referansı geçersizleştirir. Uygulama yöntemi henüz seçilmedi.
- Grafik/ses referansı için ayrı oyun dizini, sabit ayarlar ve otomatik input replay hazırlanır. Kullanıcının Prism ayarları/save'leri test alanı yapılmaz. Frame/ses yakalama ve fark raporu kullanıcıdan elle oyunu yönetmesini beklememeli.
- Java ve Rust'ın farklı frame bölünmelerinde kendi aralarında aynı sonucu vermesi peşinen şart koşulmaz; **her kontrollü zaman çizelgesinde Rust, o çizelgedeki Java sonucuyla** karşılaştırılır.
- Donanım/servis erişimi eksikse senaryo engelli kalır. GPU veya ses yakalama henüz kurulmadı diye saf referans testleri bekletilmez. Bu sonraki adaptörlerin çalışabilirliği bu turda doğrulanmış değildir.

İlk çalışma tamamlandığında rapor Java referansının hazır olup olmadığını, hangi Rust davranışlarının mevcut olduğunu ve gerçekten karşılaştırılan senaryoları ayrı göstermeli. “Test altyapısı çalışıyor” ile “oyun Java ile aynı” birbirinin yerine kullanılmaz.
