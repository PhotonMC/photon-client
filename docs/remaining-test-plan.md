# Kalan bölümler test planı (çalıştırılamayanlar)

Bu belge, henüz çalıştırılamayan istemci ve entegrasyon davranışlarının ön koşullarını kaydeder. Eşitlik kanıtı için [davranış testi sözleşmesindeki](behavior-testing.md) sıra, Java oracle ve Rust karşılaştırması gerekir.

Kapsam veya test durumu bu belgede hareketli toplamlarla tutulmaz. Tamamlanan küçük dilimler kendi parity belgelerinde, bekleyen kaynak kararları [source review](source-review.md) ve bağımlılık sırası [roadmap](roadmap.md) içinde bulunur.

## Kural

- Engelli test başarı sayılmaz; varsayılan koşu engeli gizlemez.
- Ortam hazır olunca ilgili bölüm için önce Java referansı ve testler hazırlanır (davranış testi belgesindeki sıra değişmez).
- Bu dosya kapsam defteridir; hiçbir satırı eşitlik kanıtı değildir.
- Hücrelerde senaryo adı biriktirilmez. Geçen senaryoların listesi kendi birleşik conformance kaydındadır; burada yalnızca engel, bağımlılık ve kanıt sınıfı yazılır.
- `server-conformance` kaydı bölümü ilerletir, kapatmaz. Bölüm ancak `client-parity` kaydıyla kapanır; tanımlar [davranış testi sözleşmesindedir](behavior-testing.md).
- Aynı kod yolunun sabiti değişen kopyası yeni satır veya yeni dilim üretmez; gerekçe [yön düzeltmesindedir](course-correction-2026-09-12.md).

## B — Bağlantı ve istemci dünyası

| Bölüm | Engel | İlk senaryolar (engel kalkınca) |
| --- | --- | --- |
| P02.b döngü (10 tick sınırı) | `Minecraft.runTick` tam istemci gerektirir | Büyük zaman artışında üretilen tick'e karşı yürütülen tick (en çok 10), kuyruk/paket/render sırası |
| P03.d registry adaptör sınırı | Registry verisi P07'ye bağlı | Bilinmeyen registry girdisi, eksik tag, phase sınırı; adaptörün reddetme davranışı |
| P04 kaynaklar/pack'ler | Asset index yerelde var ama pack reload akışı oyun dizini ister | Aynı yola iki pack, olmayan nesne, geçersiz index, reload sırasında değişim; çözülen kimlik/hash ve öncelik |
| P05 ayarlar/input | Pencere/odak/GUI koordinatı gerçek istemci ister | Karşıt yön sıfırlama, tuş tutma/tekrar, odak kaybı; input snapshot ve hareket vektörü |
| P06 bağlantı/login | Vanilla 26.2 status ve ilk offline play probe'ları geçti; ilk oyuncu konumu kabulü ve `player_loaded` bildirimi de vanilla koşusunda tamamlandı; encryption, parçalı ağ, disconnect ve reconnect harness'i açık | Parçalı paket, sıkıştırma eşiği, kopma; paket baytları ve durum sırası |
| P07 configuration/registry | Vanilla baseline'da ilk registry/tag ve phase geçiş probe'u geçti; eksik/tekrarlı veri ve tam client state bağlantısı açık | Registry/tag biriktirme, eksik girdide ack'siz bekleme, play'e geçiş anı |
| P08 dünya/chunk | İlk vanilla chunk/world delivery ve wire tüketimi, sorgulanabilir `ClientWorld` chunk/section/entity/block entity/light ve local player state'ine kadar ilerledi. Remote Player için üç tick'lik position interpolation, `4096.0` üstü snap, motion interpolation, teleport delta/relative motion, araç/passenger ilişki state'i ve passenger tick/rideTick dalları `server-conformance` olarak ilerletildi; passenger seat `positionRider` dalı [P08.ax kaydında](client-world-entity-passenger-position-conformance.md), Player vehicle attachment dalı [P08.ay kaydında](client-world-entity-passenger-player-conformance.md), chest boat/raft geometry dalı [P08.az kaydında](client-world-entity-passenger-boat-family-conformance.md), boat passenger rotation clamp dalı [P08.ba kaydında](client-world-entity-passenger-rotation-conformance.md), Breeze `CAN_TURN_IN_BOATS` istisnası [P08.bb kaydında](client-world-entity-passenger-boat-turn-exemption-conformance.md), Animal passenger rotation offset'i [P08.bc kaydında](client-world-entity-passenger-animal-rotation-conformance.md), Animal passenger vehicle attachment/seat offset'i [P08.bd kaydında](client-world-entity-passenger-animal-position-conformance.md), `Entity.turn` → boat `onPassengerTurned` callback'i [P08.be kaydında](client-world-entity-passenger-turn-conformance.md), pinned living body/head dispatch'i [P08.bf kaydında](client-world-entity-passenger-living-conformance.md), default vehicle attachment [P08.bg kaydında](client-world-entity-passenger-generic-position-conformance.md), boat input/nonzero `deltaRotation` [P08.bh kaydında](client-world-entity-boat-control-conformance.md), Zombie baby `VEHICLE` attachment [P08.bi kaydında](client-world-entity-passenger-baby-attachment-conformance.md), local boat `IN_AIR` tick'i [P08.bj kaydında](client-world-entity-boat-air-conformance.md) ve remote boat authority dalı [P08.bk kaydında](client-world-entity-boat-remote-authority-conformance.md) tutulur; kesin kayıtlar [doküman dizinindeki P08 kayıtlarında](index.md) tutulur. P08 `client-parity` ile kapanmadı. | Palette genişlik ve boş section edge case'leri; attribute gameplay etkileri; ışık propagation/queue; kalan semantic metadata/serializer alanları; pose/scale/dimensions'a bağlı entity attachment'ları, nested controlling passenger çözümü, su/kara boat friction/buoyancy, collision/movement packet'leri, genel motion/entity tick/lifecycle ve `client-parity` |

## C — His, görüntü, ses

| Bölüm | Engel | İlk senaryolar (engel kalkınca) |
| --- | --- | --- |
| P09 çarpışma | Yüklenmiş `ClientWorld` state ID'lerinden çağıran shape tablosuyla tam blok/slab toplama, açık shape listesiyle eksen çözümü ve shape Y koordinatlarından step-up adaylarının filtrelenmesi [P09.a](client-world-block-collision-conformance.md), [P09.b](client-world-collision-movement-conformance.md) ve [P09.c](client-world-step-up-candidates-conformance.md) kayıtlarında `client-parity` olarak doğrulandı; gerçek block-state registry/context, entity/world-border collider'ları, pose/dimensions, on-ground/flag ve aday hareket karşılaştırması açık | Slab/merdiven/dar köşe/düşük tavan; çözülen delta, on-ground ve step yüksekliği |
| P10 yerel hareket | P05 + P08 + P09'a bağlı | Tick başına konum/hız/pose; diyagonal, sprint çift basışı, su/buz/merdiven |
| P11 senkronizasyon | Initial vanilla teleport correction ack ve üç paketlik kontrollü movement probe geçti; P06 + P10'a bağlı Java input/tick hareket senkronizasyonu açık | Eşik altı/üstü hareket paketleri, 20 tick hatırlatması, teleport düzeltmesi |
| P12 hedefleme/eylem | Blok eylemi ack'i, kırma başlat/iptal/durdur, hardness ve tool progress, menzil reddi, boş-el ve consumable kullanım akışları `server-conformance` sınıfında geçti; kayıtlar kendi parity belgelerindedir. P09–P11'e bağlı genel hedefleme ve `client-parity` karşılaştırması açık | Hit sonucu, istemci tarafı blok/eşya tahmini, iki el önceliği, ack/rollback sırası |
| P13 envanter | P07 + P12'ye bağlı; pickup/quick-move/swap/throw, drag ve double-click varyantları, state-id (doğru/stale/ahead) ve `HashedStack` prediction akışları `server-conformance` sınıfında geçti. Item component prediction'ı tek kayıtta toplanır; component başına yeni dilim açılmaz | İstemci tarafı container menu/slot modeli, özel menüler, değişen slot haritası varyantları, `client-parity` karşılaştırması |
| P14 kamera | P05 + P10'a bağlı | Aynı mouse izi farklı frame bölünmesinde; FOV/pose ve zamanlama izi |
| P15–P19 görüntü/animasyon/partikül | GPU + kontrollü frame yakalama yok | Sabit sahnede geometri/ışık verisi ve görüntü; seed sabitken parçacık listesi |
| P20 ses | Ses yakalama + oyun tetikleri yok | Kaynak seçimi, zaman damgaları, pause kategorileri; aynı dosya seçimi eşitlik değildir |
| P21–P23 yazı/GUI/chat | P15 + P04 + P06'ya bağlı | Glyph/wrap/bidi ölçüler, focus/olay tüketimi, ack/disconnect sırası |
| P24–P26 kapsam/dağıtım | Yukarıdakilerin tamamına bağlı | Atanmamış davranış listesi, kontrollü oturum, platform matrisi |

## P01 kalanları (çalıştırılabilir, sonraki test dilimleri)

P01 için sıradaki küçük dilim, [source review](source-review.md) içindeki uygulanmamış Java giriş noktalarından seçilir. Her dilim kendi özgün senaryolarını, Java capture'ını, Rust karşılaştırmasını ve birleşik conformance kaydını ekler. Önceki dilim belgeleri yeni bir dilim nedeniyle yeniden yazılmaz.

P01.d/e geometrik ve random kapsamı, istemci dünyası oluşmadan yalnızca bağımsız veri yolları ölçülebildiği kadar ilerletilir. World collider toplama, movement resolution ve gerçek packet tüketimi P08–P12 ön koşullarına bağlıdır.

Yeni bir davranış dilimi hazır olduğunda bu dosyaya yalnızca yeni engel veya bağımlılık eklenir; senaryo sayıları ve genel corpus özeti runner raporlarında kalır.
