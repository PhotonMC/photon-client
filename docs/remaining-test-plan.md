# Kalan bölümler test planı (çalıştırılamayanlar)

783 Java senaryosu kayıtlıdır (beş+bir aile). Aşağıdaki bölümler yerel ortamda çalıştırılamaz; her biri engelli kaydedilir, sahte golden üretilmez. Her bölümün önkoşulu, engel kalkınca yazılacak ilk senaryolar ve kabul ölçütü tanımlıdır.

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

- P01.d: AABB çoklu shape birleşimleri; bu turdaki contains/intersects/expand/deflate ve tek kutulu `clip` hazır.
- P01.e: `fork`/`forkPositional` türev akışları; bu turdaki sıralı çekimler ve `nextGaussian` hazır.
