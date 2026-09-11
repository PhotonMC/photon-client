# Kalan bölümler test planı (çalıştırılamayanlar)

Bu belge, henüz çalıştırılamayan istemci ve entegrasyon davranışlarının ön koşullarını kaydeder. Eşitlik kanıtı için [davranış testi sözleşmesindeki](behavior-testing.md) sıra, Java oracle ve Rust karşılaştırması gerekir.

Kapsam veya test durumu bu belgede hareketli toplamlarla tutulmaz. Tamamlanan küçük dilimler kendi parity belgelerinde, bekleyen kaynak kararları [source review](source-review.md) ve bağımlılık sırası [roadmap](roadmap.md) içinde bulunur.

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
| P06 bağlantı/login | Vanilla 26.2 status ve ilk offline play probe'ları geçti; ilk oyuncu konumu kabulü ve `player_loaded` bildirimi de vanilla koşusunda tamamlandı; encryption, parçalı ağ, disconnect ve reconnect harness'i açık | Parçalı paket, sıkıştırma eşiği, kopma; paket baytları ve durum sırası |
| P07 configuration/registry | Vanilla baseline'da ilk registry/tag ve phase geçiş probe'u geçti; eksik/tekrarlı veri ve tam client state bağlantısı açık | Registry/tag biriktirme, eksik girdide ack'siz bekleme, play'e geçiş anı |
| P08 dünya/chunk | İlk chunk teslimi ve vanilla chunk wire gövdesi ile section palette storage layout'unun yapısal tüketimi geçti; heightmap type/word alanları ve ışık update uzunlukları da gözleniyor; state/biome eşleme, ışık/entity decoder'ları ve world cache açık | Palette genişlik değişimi, boş section, load-update-unload sırası |

## C — His, görüntü, ses

| Bölüm | Engel | İlk senaryolar (engel kalkınca) |
| --- | --- | --- |
| P09 çarpışma | P08 dünyası yok | Slab/merdiven/dar köşe/düşük tavan; çözülen delta, on-ground ve step yüksekliği |
| P10 yerel hareket | P05 + P08 + P09'a bağlı | Tick başına konum/hız/pose; diyagonal, sprint çift basışı, su/buz/merdiven |
| P11 senkronizasyon | Initial vanilla teleport correction ack ve üç paketlik kontrollü movement probe geçti; P06 + P10'a bağlı Java input/tick hareket senkronizasyonu açık | Eşik altı/üstü hareket paketleri, 20 tick hatırlatması, teleport düzeltmesi |
| P12 hedefleme/eylem | Wire-level `ABORT_DESTROY_BLOCK` ACK'i, creative spawn-altı `START_DESTROY_BLOCK` → state `0` block update'u, menzil dışı action reddi, boş-el `use_item_on`, boş-el `use_item`, creative snowball projectile use, survival snowball tüketimi, survival water bucket `use_item_on` + `use_item` fallback, survival empty bucket pickup, component-bearing milk bucket use, aktif multi-effect milk bucket temizliği, component-bearing honey bottle use, aktif poison temizleyen honey bottle use, honey bottle erken bırakma, food component override'lı pufferfish tüketimi, default full-hunger pufferfish reddi, golden apple tüketimi, enchanted golden apple tüketimi, kontrollü creative stone placement, survival start→abort, dirt start→stop, stone ve oak planks kısa start→stop hardness, stone wooden pickaxe tool progress, dirt wooden shovel tool progress, oak planks wooden axe tool progress, delayed destroy ve spawn protection probe'ları geçti; P09–P11'e bağlı genel hedefleme açık | Hit sonucu, survival diğer hardness/tool progress'i, daha fazla item component codec'i ve consumable varyantı, diğer effect türleri, inventory, ack/rollback sırası |
| P13 envanter | P07 + P12'ye bağlı; vanilla survival primary/secondary pickup, quick-move, swap, tek item throw, throw-all, boş carried drag cancellation, iki-slot drag dağıtımı, double-click, double-click stack limit, ahead state-id varyantı, doğru HashedStack prediction'ı, yanlış changed-slot prediction'ı, yanlış carried prediction'ı, iki hash'in birlikte yanlış olduğu durum, yanlış component hash prediction'ı, doğru component hash prediction'ı, partial changed-slot prediction'ı ve stale state-id ile doğru prediction'da authoritative slot/cursor davranışı geçti; genel container mantığı açık | Diğer component codec'leri, ahead state-id'nin doğru hash varyantı, stale state ile component'li item, yanlış state'in diğer değişen packet alanı varyantları ve özel menüler |
| P14 kamera | P05 + P10'a bağlı | Aynı mouse izi farklı frame bölünmesinde; FOV/pose ve zamanlama izi |
| P15–P19 görüntü/animasyon/partikül | GPU + kontrollü frame yakalama yok | Sabit sahnede geometri/ışık verisi ve görüntü; seed sabitken parçacık listesi |
| P20 ses | Ses yakalama + oyun tetikleri yok | Kaynak seçimi, zaman damgaları, pause kategorileri; aynı dosya seçimi eşitlik değildir |
| P21–P23 yazı/GUI/chat | P15 + P04 + P06'ya bağlı | Glyph/wrap/bidi ölçüler, focus/olay tüketimi, ack/disconnect sırası |
| P24–P26 kapsam/dağıtım | Yukarıdakilerin tamamına bağlı | Atanmamış davranış listesi, kontrollü oturum, platform matrisi |

## P01 kalanları (çalıştırılabilir, sonraki test dilimleri)

P01 için sıradaki küçük dilim, [source review](source-review.md) içindeki uygulanmamış Java giriş noktalarından seçilir. Her dilim kendi özgün senaryolarını, Java capture'ını, Rust karşılaştırmasını ve contract/parity belgesini ekler. Önceki dilim belgeleri yeni bir dilim nedeniyle yeniden yazılmaz.

P01.d/e geometrik ve random kapsamı, istemci dünyası oluşmadan yalnızca bağımsız veri yolları ölçülebildiği kadar ilerletilir. World collider toplama, movement resolution ve gerçek packet tüketimi P08–P12 ön koşullarına bağlıdır.

Yeni bir davranış dilimi hazır olduğunda bu dosyaya yalnızca yeni engel veya bağımlılık eklenir; senaryo sayıları ve genel corpus özeti runner raporlarında kalır.
