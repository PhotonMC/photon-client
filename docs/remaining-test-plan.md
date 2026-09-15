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
| P05 ayarlar/input | Window GUI ölçeği ve ölçekli boyut [P05.d.1 client-parity kaydıyla](window-gui-scale-conformance.md) geçti; pencere oluşturma, odak, resize ve GUI koordinatı gerçek yüzey ister | Karşıt yön sıfırlama, tuş tutma/tekrar, odak kaybı; input snapshot ve hareket vektörü |
| P06 bağlantı/login | Vanilla 26.2 status ve ilk offline play probe'ları geçti; ilk oyuncu konumu kabulü ve `player_loaded` bildirimi de vanilla koşusunda tamamlandı; encryption, parçalı ağ, disconnect ve reconnect harness'i açık | Parçalı paket, sıkıştırma eşiği, kopma; paket baytları ve durum sırası |
| P07 configuration/registry | Vanilla baseline'da ilk registry/tag ve phase geçiş probe'u geçti; eksik/tekrarlı veri ve tam client state bağlantısı açık | Registry/tag biriktirme, eksik girdide ack'siz bekleme, play'e geçiş anı |
| P08 dünya/chunk | Chunk/section/light teslimi, `ClientWorld` chunk cache, block/entity/light state ve local player binding ilerledi; entity tick, boat ve passenger dalları da bu bölüm altında yürüdü. Kayıtların tamamı [belge dizinindeki P08 girdilerindedir](index.md). Bölüm `client-parity` ile kapanmadı; entity/boat/passenger kapsamı [P17'ye taşınacaktır](course-correction-2026-09-12.md) | Palette genişlik değişimi, boş section, load-update-unload sırası |

## C — His, görüntü, ses

| Bölüm | Engel | İlk senaryolar (engel kalkınca) |
| --- | --- | --- |
| P09 çarpışma | Shape toplama, eksen çözümü, step-up aday filtresi, entity fit/pose ölçüleri, world-border shape'leri ve pinned block-state registry çözümü `client-parity` olarak doğrulandı; kayıtlar [belge dizinindeki P09 girdilerindedir](index.md). Gerçek dünya collider'ları ile tam hareket çözümü açık | Slab/merdiven/dar köşe/düşük tavan; çözülen delta, on-ground ve step yüksekliği |
| P10 yerel hareket | LocalPlayer input dönüşümü, `applyInput`, sprint gate/trigger, fluid falling, climbable clamp, friction yolları, travel dispatch ve attribute→input binding'leri `client-parity` olarak doğrulandı; kayıtlar [belge dizinindeki P10 girdilerindedir](index.md). LocalPlayer tick'i, collision-aware hareket ve kalan attribute binding'leri açık | Tick başına konum/hız/pose; diyagonal, sprint çift basışı, su/buz/merdiven |
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
