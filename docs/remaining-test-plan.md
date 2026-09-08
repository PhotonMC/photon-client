# Kalan bölümler test planı (çalıştırılamayanlar)

896 Java senaryosu kayıtlıdır (beş+bir aile). Aşağıdaki bölümler yerel ortamda çalıştırılamaz; her biri engelli kaydedilir, sahte golden üretilmez. Her bölümün önkoşulu, engel kalkınca yazılacak ilk senaryolar ve kabul ölçütü tanımlıdır.

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

- P01.d: VoxelShape temel/OR ve `Shapes.join` tarafından kabul edilen `FALSE`, `AND`, `ONLY_FIRST`, `ONLY_SECOND`, `NOT_SAME`, `FIRST` ve `SECOND` yolları hazır; tekli ve çoklu shape collision de hazır ([tekli sözleşme](shape-collision-contract.md), [çoklu sözleşme](shape-collision-many-contract.md)); AABB contract, inflate, intersect, minmax, move, move overload'ları, sizes, hasNaN, positions, axis min/max, distanceToSqr, ofSize ve endpoint setter'ları da hazır ([contract sözleşmesi](aabb-contract.md), [inflate sözleşmesi](aabb-inflate-contract.md), [intersect sözleşmesi](aabb-intersect-contract.md), [minmax sözleşmesi](aabb-minmax-contract.md), [move sözleşmesi](aabb-move-contract.md), [move overload sözleşmesi](aabb-move-overloads-contract.md), [sizes sözleşmesi](aabb-sizes-contract.md), [hasNaN sözleşmesi](aabb-has-nan-contract.md), [positions sözleşmesi](aabb-positions-contract.md), [axis min/max sözleşmesi](aabb-axis-limits-contract.md), [distanceToSqr sözleşmesi](aabb-distance-contract.md), [ofSize sözleşmesi](aabb-of-size-contract.md), [endpoint setter sözleşmesi](aabb-set-endpoint-contract.md)); kalan AABB çoklu shape birleşimleri ve occlusion sonraki dilimlerdir. `false,false` için true üreten BooleanOp sabitleri Java `Shapes.join` tarafından reddedilir. Bu turdaki contains/intersects/intersect/minmax/move/move overload'ları/sizes/has_nan/positions/min/max/distance_point/distance_box/of_size/set_min_x/set_min_y/set_min_z/set_max_x/set_max_y/set_max_z/expand/deflate ve tek kutulu `clip` de hazır.
- P01.e: `fork`/`forkPositional`, `nextGaussian`, `setSeed`, random yardımcı overload'ları ve bu turdaki sıralı çekimler hazır; farklı kaynak türleri ve overflow hata yolları ayrı kapsamdır.
