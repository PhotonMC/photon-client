# Java Edition 26.2 istemci yol haritası

Referans Minecraft Java Edition 26.2 / protokol 776 / Java 25'tir. [Referans envanteri](reference-baseline.md), [kaynak incelemesi](source-review.md), [davranış testi yaklaşımı](behavior-testing.md) ve [yerel runner](behavior-runner.md) bu planın dayanağıdır.

2026-09-12 [yön düzeltmesi](course-correction-2026-09-12.md) bu planın önceliklendirme kuralıdır ve yeni dilim seçmeden önce okunur.

Her bölüm küçük, incelenebilir davranış dilimlerine ayrılır. Dilim durumu kendi birleşik contract/parity kaydında tutulur; bu yol haritası bağımlılıkları, sıralamayı ve engelleri açıklar. Teknoloji seçimi, ilgili kaynak ve davranış ihtiyacı ortaya çıkmadan yapılmaz.

## Uygulama ve kabul kuralı

Pxx bir **bölüm**, Pxx.a/b/c ise küçük davranış dilimidir. Bölüm tek commit veya tek büyük taşıma işi değildir. Her alt dilim için önce ilgili Java dalları incelenir, sözleşme ve Java referans testi oluşturulur; sonra gerekiyorsa Rust implementasyonu yapılır. Test çalıştırıcısı/test sözleşmesi ile tamamlanmış Rust dilimi ayrı incelenebilir commitler olabilir. Aşağıdaki sıra, bütün testleri tek seferde yazıp sonra bütün oyunu geliştirmek anlamına gelmez.

Tablolardaki testler başlangıç senaryolarıdır; alanın tüm varyantlarını kapsadığı varsayılmaz. Her alt dilimin kaydı kimliği, kesin kaynak konumlarını, bağımlılıklarını, girdileri, Java yakalama yöntemini, karşılaştırılan çıktıları, sonucu ve commit'i içerecek. Karşılaştırma kuralları [ilk sözleşmelerde](first-behavior-tests.md) somutlaştırıldı.

Kabul için gerekli yerel testler gerçekten çalışmalı, aynı girdide Java ve Rust'ın ilgili gözlenebilir çıktıları eşit olmalı ve önceki ilgili dilimlerde fark oluşmamalı. Kaynak/ortam eksikliği veya sunucu farkı “engelli”; beklenen fark “geçti” değildir. Görsel/ses dilimlerinde yalnızca mantık izi yeterli kabul edilmez.

## A — Çalıştırılabilir referans ve temel davranışlar

| Bölüm | Ön koşul | Küçük dilimler | Önce yazılacak davranış testleri ve karşılaştırılan çıktı |
| --- | --- | --- | --- |
| **P00 Referans çalıştırıcısı** | Yerel girdiler mevcut | a: kimlik/bütünlük kontrolü; b: orijinal JAR sınıf çağrıları; c: senaryo, yakalama ve karşılaştırma komutları | Eksik/farklı JAR veya asset başarı üretmemeli. Aynı sabit senaryo tekrarında aynı referans; değiştirilmiş bir çıktı fark olarak yakalanmalı. Rust adaptörü yoksa eşitlik sonucu verilmemeli. |
| **P01 Sayısal ve geometrik temel** | P00 | a: kullanılan `Mth` ve `Vec2` işlemleri; b: `Vec3`, koordinat/angle dönüşümleri; c: `BlockPos`, `SectionPos`, `Direction`; d: AABB/ray ve shape temel işlemleri; e: istemcinin gerçekten kullandığı random yolları | Negatif koordinatlar, sıfır/işaretli sıfır, sınır komşuları, diyagonal normalizasyon, açı sarmalama, kutu yüzüne temas ve sabit seed dizileri. Sayısal çıktılar, float bitleri, yön/koordinat ve hit sonucu. Bütün matematik kütüphanesini peşinen taşıma. |
| **P02 Zaman ve olay sırası** | P00; gereken P01 işlemleri | a: `DeltaTracker.Timer`; b: frame/tick sırası ve en çok 10 tick sınırı; c: pause/freeze ve server tick-rate olayları | 0/1/49/50/51 ms çevresi, artık zaman, uzun frame, pause/unpause ve freeze kombinasyonları. Üretilen/yürütülen tick, kısmi tick bitleri, görev/paket/input/render sırası. P06 sonrası gerçek paket kuyruğuyla yeniden bağlanır. |
| **P03 Veri ve wire kodlamaları** | P00; P01.c gereken yerlerde | a: VarInt/VarLong; b: UTF-8, UUID, konum ve bitset; c: NBT okuma/sınırları; d: registry bağımlı alanlar için adaptör sınırı | Başarılı, kesilmiş, fazla uzun, sınır dışı ve geçerli fakat alışılmadık kodlamalar. Yalnızca roundtrip değil Java'dan alınan bayt dizisi, tüketilen bayt sayısı, değer ve hata davranışı. Registry adaptörünün tamamlanması P07'ye bağlı. |
| **P04 Yerel kaynaklar ve pack'ler** | P00, gerekli P03 biçimleri | a: index/object ve JAR kaynaklarını çözme; b: pack sırası/metadata/known pack verisi; c: reload, eksik kaynak ve uyumsuz sürüm; d: kullanıcının seçtiği asset kökünü bağlama | Aynı mantıksal yola birden fazla pack, olmayan nesne, geçersiz index, reload sırasında kaynak değişimi. Çözülen kaynak kimliği/hash'i, öncelik, hata/fallback ve reload olayları. Test verisindeki orijinal içerik yerelde kalır. |
| **P05 Ayarlar, pencere olayları ve input** | P00–P02; kaynak isteyen alt dilimde P04 | a: option varsayılanları ve kaydet/yükle; b: key mapping, basma/bırakma/tekrar; c: `KeyboardInput`; d: mouse birikimi, scroll, odak ve GUI koordinatı | Karşıt yönler ve tüm yön kombinasyonları, aynı frame'de olaylar, tuş tutma/tekrar, odak kaybı, GUI ölçeği ve yeniden boyutlandırma. Input snapshot'ı ve hareket vektörü, click tüketimi, koordinatlar. GUI'ye yönlendirme P22'de tamamlanır. |

P00–P03'ün ilk dilimleri grafik, ses veya Pumpkin kurulumu beklemez. P01/P03'te yalnızca sonraki davranış diliminin ihtiyaç duyduğu yardımcılar geliştirilir.

## B — Bağlantı ve istemci dünyası

| Bölüm | Ön koşul | Küçük dilimler | Önce yazılacak davranış testleri ve karşılaştırılan çıktı |
| --- | --- | --- | --- |
| **P06 Bağlantı ve login** | P02–P03, P05 ayarları | a: durum/ping ve frame sınırları; b: handshake/login; c: compression/encryption geçişleri; d: keepalive, ilk oyuncu konumu, player-loaded, disconnect ve yeniden bağlanma | Parçalı/birleştirilmiş ağ verisi, geçiş sırasında gelen paket, sıkıştırma eşiği ve kopma. Paket baytları, protokol durumu, yanıt sırası ve kullanıcıya iletilen sonuç. Güncel canlı baseline resmi vanilla 26.2 server'dır; Pumpkin adayındaki farklar ayrı karşılaştırma kaydıdır. |
| **P07 Configuration ve registry'ler** | P03–P04, P06 | a: known pack/feature seçimi; b: registry ve tag biriktirme; c: configuration→play ve yeniden configuration; d: item/block/data component bağlantıları | Paket sırasını, eksik/tekrarlı registry girdisini, tag güncellemesini ve phase sınırını kontrol et. Registry kimliği/çözümü, ack ve play'e geçiş anı Java ile aynı olmalı. Pumpkin verisinin tamamlığı ayrı raporlanır. |
| **P08 Dünya kopyası ve chunk yaşam döngüsü** | P01–P03, P07 | a: dimension ve client level; b: palette/chunk section/biome çözümü; c: chunk/light/block entity paketleri; d: cache merkezi/radius/unload; e: entity ve local player state/tick; f: login/respawn; g: dimension registry bounds/environment; h: semantic metadata ve client world lifecycle | İlk vanilla chunk teslimi, sorgulanabilir `ClientWorld` section/entity/block entity/light state'i ve local player world state'i `server-conformance` sınıfında ilerledi. P08 kayıtlarının kesin kapsamı ve canlı sonuçları [doküman dizinindeki P08 kayıtlarında](index.md) tutulur. P08 bölümü için `client-parity`, palette/empty-section edge case'leri, attribute gameplay etkileri, ışık propagation/queue, kalan semantic metadata/serializer alanları, kalan generic body/head turn semantiği, genel motion integration, gerçek entity tick/lifecycle ve tam oynanabilir istemci hâlâ açıktır; remote Player position, snap ve motion interpolation kayıtları bu ilerlemenin sınırlı alt dallarıdır. | Palette genişlik ve boş section edge case'leri; attribute gameplay etkileri; ışık propagation/queue; kalan semantic metadata/serializer alanları; kalan generic body/head turn, genel motion/entity tick/lifecycle ve `client-parity` |

## C — Oyuncunun hissettiği davranış

| Bölüm | Ön koşul | Küçük dilimler | Önce yazılacak davranış testleri ve karşılaştırılan çıktı |
| --- | --- | --- | --- |
| **P09 Çarpışma ve çevre sorguları** | P01, P08 | a: blok/entity/world-border shape toplama; b: eksen sırasıyla çözüm; c: step-up adayları; d: pose ve sığma/kenar kontrolleri | Tam blok, slab, merdiven, dar köşe, düşük tavan, blok sınırı ve birden fazla collider. Çözülen delta, AABB, on-ground, collision flag'leri ve seçilen step yüksekliği. Genel fizik engine'i eşitlik kanıtı sayılmaz. |
| **P10 Yerel hareket** | P02, P05, P08–P09 | a: input çarpanları ve kare hareket düzeltmesi; b: yürüme/sprint/zıplama/crouch; c: su/lav/merdiven ve farklı yüzeyler; d: uçuş/elytra/auto-jump; e: araç kontrolü ve attribute/effect etkileşimi | Tick başına konum, hız, pose, sprint/flight durumu ve tetik zamanları. Diyagonal hareket, eşya kullanırken yürüme, sprint çift basışı, tavan altında zıplama, su geçişi, buz/özel yüzeyler. Her hareket modu ayrı dilim; örnek liste tüm block/entity varyantları yerine geçmez. |
| **P11 Hareket senkronizasyonu** | P06–P08, P10 | a: input/sprint değişim paketleri; b: konum/rotasyon/status seçimi; c: teleport ve göreli düzeltme; d: remote entity interpolasyonu ve araç paketleri | Kontrollü üç paketlik vanilla movement probe'u ile ilk packet kabulü; hareket eşiğinin altı/eşiti/üstü, 20 tick hatırlatması, yalnızca rotasyon veya flag değişimi; teleport sırasında binme ve prediction. Son durumun yanında paket türü, alanları, sırası ve gönderim tick'i. |
| **P12 Hedefleme, eylem ve prediction** | P09–P11 | a: blok/entity pick ve reach; b: saldırı/elle vurma; c: kırma başlat/devam/iptal; d: yerleştirme/eşya kullanımı; e: sequence ack, rollback ve teleport etkileşimi | Blok eylemi ack'i, kırma başlat/iptal/durdur, hardness ve tool progress, menzil reddi, boş-el ve consumable `use_item`/`use_item_on` akışları `server-conformance` sınıfında geçti; kayıtlar kendi birleşik conformance dosyalarındadır. Açık: hit sonucu, animasyon/ses tetikleri, istemci tarafı blok/eşya tahmini, iki el önceliği, gecikmiş ack ve P09–P11'e bağlı genel hedefleme. Otoriter sunucu sonucunu istemcide yeniden üretme; bölüm `client-parity` kaydı olmadan kapanmaz. |
| **P13 Envanter ve container davranışı** | P03, P07, P12 | a: item stack/component ve cursor; b: pickup/quick-move/swap/throw; c: drag dağıtımı ve çift tıklama; d: server state-id/slot düzeltmeleri; e: creative ve özel menü işlemleri | Pickup/quick-move/swap/throw, drag ve double-click varyantları, state-id (doğru, stale, ahead) ve `HashedStack` prediction (doğru, yanlış slot, yanlış carried, partial) akışları `server-conformance` sınıfında geçti. Item component prediction'ı tek kayıt altında toplanır; component başına ayrı dilim açılmaz ([yön düzeltmesi](course-correction-2026-09-12.md)). Açık: istemci tarafı container menu/slot modeli, özel menüler, değişen slot haritası varyantları ve `client-parity` karşılaştırması. UI görüntüleri P22'dedir. |
| **P14 Kamera ve bakış hissi** | P02, P05, P10–P11 | a: mouse hassasiyet/smooth/scope; b: yaw/pitch ve kamera geçişleri; c: eye-height/FOV interpolasyonu; d: bob/hurt/üçüncü şahıs kamera çarpışması | Aynı mouse izi farklı frame bölünmelerinde, invert eksenler, sprint FOV, pose değişimi ve duvar yakını üçüncü şahıs. Kamera konumu, açı, FOV, transform ve zamanlama izi. Nihai görüntü P15 sonrası karşılaştırılır. |

## D — Görüntü, animasyon ve ses

| Bölüm | Ön koşul | Küçük dilimler | Önce yazılacak davranış testleri ve karşılaştırılan çıktı |
| --- | --- | --- | --- |
| **P15 Görsel referans ve temel çizim** | P04–P05, P14 | a: kontrollü Java frame yakalama/ortam kaydı; b: texture/sampler/atlas temeli; c: shader/pipeline, hedef ve renk davranışı; d: reload/resize/frame sunumu | Aynı kaynak ve ayarlarla kontrollü frame'ler; UV, filtreleme, alpha/depth, renk dönüşümü ve sıralama. Önce Java tekrarlarıyla ortam değişkenliği ölçülür. Backend seçimi bu ihtiyaçlar ortaya çıktıktan sonra yapılır; orijinal shaderlar yerel girdidir. |
| **P16 Bloklar, ışık ve dünya çizimi** | P08, P15 | a: model/blockstate ve varyant seçimi; b: section geometrisi/face culling; c: ışık/AO/tint; d: transparan/fluid katman ve sıralama; e: görünürlük/section invalidation | Sabit küçük sahneler, sınır komşusu blok, saydam yüzey, biome sınırı, gece/gündüz ışığı ve chunk güncellemesi. Geometri/ışık verisi ve görüntü. Varyant seçimi random akışıyla eşleşmeli. |
| **P17 Entity, model ve animasyon** | P08, P11, P15–P16 | a: entity render state; b: pose/yaş/interpolasyon; c: model ve animation kanalları; d: skin/equipment/layer; e: block entity ve özel renderer | Sabit tick ve partial tick'te yürüme, hasar, ölüm, yüzme, binme, ekipman ve farklı entity türleri. Eklem/transform, layer sırası ve görüntü. İçerik defteri her özel model yolunu ilgili senaryoya bağlar. |
| **P18 Eller, eşya ve ekran etkileri** | P12–P15, P17 | a: el/equip/swing; b: yemek/içmek/yay/arbalet/dürbün ve diğer kullanım animasyonları; c: harita/iki el; d: portal/su/ateş ve diğer ekran efektleri | Kullanım başlangıcı/bitişi/iptali, el değişimi ve saldırıyla çakışma. Transform ve animasyon evresi yanında frame görüntüsü. Referanstaki yeni eşya animasyonları içerik defterine ayrıca eklenir. |
| **P19 Partiküller ve çevre sunumu** | P01–P02, P08, P15–P16 | a: partikül doğumu/random/yaşam; b: hareket/çarpışma; c: hava/sky/cloud/fog/environment attribute; d: özel efekt ve geçişler | Seed, konum, tick ve kamera sabitken parçacık listesi/yaşı, hava/dimension geçişleri ve görüntü. Random tüketim sırası da gözlenir. |
| **P20 Ses** | P02, P04; oyun tetikleri için P10, P12, P17 | a: ses tanımı/ağırlıklı seçim; b: oyun olayı, gecikme/loop/stop; c: listener, pitch/volume/attenuasyon; d: decode/stream/mix ve pause; e: altyazı bağlantısı | Ayak sesi, kazma, eşya, ortam/müzik; kaynak seçimi ve zaman damgaları; sabit listener ve kanal ayarlarında ses yakalama. Aynı ses dosyasını seçmek tek başına duyulan eşitlik değildir. Java'da pause sırasında devam eden kategoriler ayrıca ölçülür. |

P20.a/b için renderer tamamlanmasını beklemek gerekmez. P15–P20'de tüm orijinal görüntü, PCM ve golden çıktılar `.data/` altında kalır. Yalnızca test çalıştırıcısı ve özgün senaryo tanımları izlenir.

## E — Arayüz ve kapsamın tamamlanması

| Bölüm | Ön koşul | Küçük dilimler | Önce yazılacak davranış testleri ve karşılaştırılan çıktı |
| --- | --- | --- | --- |
| **P21 Font, dil ve metin** | P04–P05, P15 | a: font sağlayıcı/glyph/fallback; b: genişlik/wrap/bidi/stil; c: dil değişimi ve reload; d: metin input, seçim ve IME | Türkçe ve diğer referans diller, eksik glyph, birleşen karakterler, sağdan sola metin, genişlik sınırları, caret/selection. Ölçüler, satırlar, event sonucu ve glyph görüntüsü. |
| **P22 GUI, HUD ve menüler** | P05, P13, P18, P20–P21 | a: focus/hover/click/scroll ve katmanlar; b: title/options/server list/connect/pause; c: HUD/hotbar/chat paneli; d: container/creative/recipe book; e: tooltip/toast/loading/death/credits ve kalan ekranlar | Input izi ve ekran boyutuna göre widget sınırları, focus/olay tüketimi, geçiş sırası, ekran görüntüsü ve UI sesi. Ekran defteri her somut screen'i kapsar; birkaç ana ekran tamamlanması GUI eşitliği değildir. |
| **P23 Chat, oturum ve ortak servis davranışları** | P06–P07, P21–P22 | a: component/chat geçmişi ve komut önerileri; b: signed chat/ack/session akışı; c: server resource pack/cookie/transfer/dialog; d: sosyal görünürlük ve servis hata ekranları | Kontrollü mesaj dizisi, sıralama, ack, disconnect/reconnect, iptal edilen pack ve diyalog. Metin/stil, protokol çıktısı ve ekran sonucu. Hesap gerektiren gerçek servis kontrolleri erişim yoksa engelli kaydedilir. |
| **P24 İçerik ve özel davranış defteri** | P07–P23'te ilgili yollar | a: block/item/entity özel davranış eşlemesi; b: özel container, harita/kitap/banner gibi veriyle çizilen içerik; c: advancement/stat/scoreboard/bossbar/waypoint/tutorial; d: tüm packet handler ve resource türlerinin kapsam kontrolü | Her özel davranış yoluna senaryo ata; aynı genel yolu kullanan veri varyantlarını eşdeğerlik gruplarıyla kapsa. Bir milyon kopya test yerine kaynak dalına ve gözlenen farka göre genişlet. Atanmamış davranışlar görünür listede kalmalı. |
| **P25 Sunucuya/servise bağlı istemci akışları** | P06–P08, P22–P24 | a: local world oluştur/aç/kapat akışını Pumpkin yetenekleriyle eşle; b: singleplayer pause/LAN/dünya yönetimi; c: Realms ve diğer haricî servis ekran/işlemleri; d: erişilebilirlik/narrator ve platform özellikleri | Akış durumu ve UI'nin yanında sürecin gerçek sonucu ölçülür. Sunucu yazılmaz; Pumpkin veya servis karşılamıyorsa fark kapanmış sayılmaz. Bu bölüm kapsamdan çıkarılmış değil, entegrasyon/ürün kararı gereken bölümdür. |
| **P26 Birleşik eşitlik ve dağıtım doğrulaması** | İlgili tüm önceki dilimler | a: Java ve Rust'ta aynı kontrollü oturum; b: sabit Pumpkin'de iki istemci senaryosu; c: platform/ayar matrisi ve kapsam açıkları; d: paket içeriği ve dış asset yükleme | Bağlan→chunk→hareket→eylem→envanter→ses/UI→respawn/dimension→yeniden bağlan. Tick/packet/state ve görüntü/ses birlikte karşılaştırılır. Orijinal içerik Git, paket veya executable'a girmemeli. Geçen sınırlı senaryolar tüm oyun için mutlak kanıt olarak sunulmaz. |

P24 kapsam defteri ilgili bölümler geliştikçe doldurulur; en sona bırakılan bir keşif işi değildir. Engelli bölümlerin listesi ve ilk senaryoları [kalan test planındadır](remaining-test-plan.md). P25'e ait uyumluluk riskleri de ilk bağlantı aşamasından itibaren takip edilir. Hiçbir açık fark kullanıcı adına “önemsiz” sayılarak kapatılmaz.

## Kanıt sınıfı

Roadmap satırındaki "geçti" ifadesi kanıt sınıfını taşımak zorundadır.
`server-conformance` canlı vanilla sunucusunun Rust paketini kabul etmesidir ve
bölümü ilerletir; `client-parity` aynı girdide Java istemcisiyle karşılaştırmadır
ve bölümü kapatabilen tek sınıftır. Tanımlar [davranış testi
sözleşmesindedir](behavior-testing.md). P06–P08 ve P12–P13 altındaki mevcut
kayıtların tamamı `server-conformance` sınıfındadır.

## Kritik yol

Bloke ettiği bölüm sayısına göre kritik yol hâlâ **P08 dünya kopyasıdır**.
`ClientWorld` chunk storage/application, ışık ve block entity state'i,
entity/local-player state'i, dimension/respawn, tick ve temel interpolation
adımlarıyla sorgulanabilir hâle geldi. Bu kanıtlar vanilla 26.2 sunucusuna bağlı
`server-conformance` kayıtlarıdır; kesin kapsam ve her dilimin açık alanı
[doküman dizinindeki P08 kayıtlarında](index.md) tutulur.

P09 çarpışma, P10 hareket, P14 kamera ve P16 dünya çizimi için palette/empty
section edge case'leri, attribute gameplay etkileri, ışık propagation/queue,
kalan semantic metadata ve serializer alanları, kalan body/head turn semantiği,
genel motion integration, gerçek entity tick yürütmesi ve tam entity lifecycle
hâlâ ön koşuldur. Snowball free-flight tick alt dalı [P08.aq kaydında](client-world-entity-snowball-tick-conformance.md)
sınırlı olarak, remote Player position interpolation [P08.ar kaydında](client-world-entity-player-interpolation-conformance.md),
uzaklık üstü snap [P08.as kaydında](client-world-entity-player-snap-conformance.md),
motion interpolation [P08.at kaydında](client-world-entity-player-motion-conformance.md)
ve teleport delta/relative motion [P08.au kaydında](client-world-entity-teleport-delta-conformance.md), araç/passenger ilişki state'i
[P08.av kaydında](client-world-entity-passengers-conformance.md) ve passenger tick/rideTick
[P08.aw kaydında](client-world-entity-passenger-tick-conformance.md), passenger seat `positionRider`
[P08.ax kaydında](client-world-entity-passenger-position-conformance.md) ve Player vehicle attachment
[P08.ay kaydında](client-world-entity-passenger-player-conformance.md) ve chest boat/raft geometry
[P08.az kaydında](client-world-entity-passenger-boat-family-conformance.md) ve boat passenger rotation clamp
[P08.ba kaydında](client-world-entity-passenger-rotation-conformance.md) ve Breeze `CAN_TURN_IN_BOATS` istisnası
[P08.bb kaydında](client-world-entity-passenger-boat-turn-exemption-conformance.md), Animal passenger rotation offset'i
[P08.bc kaydında](client-world-entity-passenger-animal-rotation-conformance.md) ve Animal passenger vehicle attachment/seat offset'i
[P08.bd kaydında](client-world-entity-passenger-animal-position-conformance.md), ayrıca `Entity.turn` → boat `onPassengerTurned` callback'i
[P08.be kaydında](client-world-entity-passenger-turn-conformance.md) `server-conformance` olarak ilerletilmiştir; P08 `client-parity` ile kapanmadan
bu bağımlılıklar açılmaz.

P12/P13 altında yeni item component varyantı açılmaz; gerekçe ve bağlayıcı
eylemler [yön düzeltmesi belgesindedir](course-correction-2026-09-12.md).
Kolay ölçülebilir olduğu için yatay varyant üretmek ilerleme sayılmaz.

## İlk çalışma sırası ve mevcut engeller

Headless Java oracle, JSONL runner ve Rust adapter ilk davranış dilimleri için kullanılabilir durumdadır. Uygulama girişi hâlâ oynanabilir bir istemci değildir; geçtiği bildirilen adapter senaryoları tam oyun eşitliği iddiası taşımaz.

P01'deki yeni yardımcılar Java kaynak incelemesi ve kendi parity belgeleriyle sırayla ele alınır. P02/P03'ün istemciye bağlanan frame, registry ve packet akışları ilgili bağımlılıklar hazır olduğunda ilerler. Kesin açık helper listesi [source review](source-review.md) içinde tutulur.

İnsan tarafından elle oynanması gereken ayrı bir kabul süreci tasarlanmıyor. P06–P08 için ilk gerçek vanilla 26.2 bağlantı/world-delivery probe'u ve Pumpkin karşılaştırması [birleşik oyun oturumu kaydında](game-connection-conformance.md) çalışır durumdadır; chunk ve ışık packet gövdesinin, heightmap/light sınırlarının ve section palette storage layout'unun yapısal tüketimi [ayrı P08 kaydında](chunk-packet-conformance.md), sorgulanabilir ilk `ClientWorld` state'i ise [P08.a/b kaydında](client-world-state-conformance.md) doğrulanmıştır. Bu probe oynanabilir istemci değildir; P05 pencere/input, dinamik dünya yaşam döngüsü, disconnect/reconnect, hareket, eylem, inventory, kamera, render, ses ve GUI katmanlarının kontrollü harness'leri hâlâ kurulacaktır.

Kalan ilk gerçek istemci engelleri pencere ve input, dinamik chunk/client world state, disconnect/reconnect ve kontrollü frame/audio/network capture ortamıdır. [Kalan test planı](remaining-test-plan.md) her engelin ön koşulunu ve beklenen gözlemi taşır.
