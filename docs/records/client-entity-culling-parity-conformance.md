# T47 — Entity uzaklık ve bölüm görünürlüğü ayıklaması — [client-parity]

Bu kayıt, bir karede hangi entity'nin çizilmeyeceğini (`LevelExtractor.isEntityVisible`)
kapsar: türün uzaklık sınırı (`Entity.shouldRender`/`shouldRenderAtSqrDistance` ve ezmeleri),
çizicinin görüş konisine sorduğu kutu (`EntityRenderer.getBoundingBoxForCulling`,
`affectedByCulling`) ve entity'nin bulunduğu bölümün gösterilmiş olması. Kanıt sınıfı
`client-parity`'dir: gerçek Java entity'si ve gerçek çizici sınıfı (Unsafe ile kurulur)
`movement/living-entity-animations` içindeki `cull-*` satırlarında koşturulur; uzaklık
kararları iki görüş ölçeğinde yirmi uzaklıkta, kutu bit bit karşılaştırılır.

## Java kaynak incelemesi

- `client/renderer/extract/LevelExtractor.java:218-258`: görüş ölçeği
  `clamp(etkin görüş mesafesi / 8, 1, 2.5) × entityDistanceScaling`; entity çiziciye göre
  görünür ya da yerel oyuncu onun (dolaylı) yolcusuysa, bloğu dünya yüksekliği dışında veya
  bölümü derlenmiş ve solması 0.3'ü geçmişse (`LevelRenderer.isSectionCompiledAndVisible`)
  çizilir.
- `client/renderer/entity/EntityRenderer.java:61-96`: uzaklık, sonra `affectedByCulling`,
  sonra yarım blok büyütülmüş kutu (NaN ya da sıfır boyda 4 bloklu küp), sonra tasma sahibinin
  kutusu ve ikisinin birleşimi.
- Kutu ezmeleri: illusioner (yanlara 3), sniffer (0.6), trident (1.5), mutlu ghast (alt, boyun
  yarısı kadar), minecart (gösterdiği blok varsa ofsetin 0.75/16'sı yukarı), ejderha kafası
  takan canlı (0.5), display (kendi genişlik/yükseklik kutusu; biri sıfırsa ayıklanmaz).
  Ayıklanmayanlar: yıldırım, ejderha, olta şamandırası.
- Uzaklık ezmeleri: uzak oyuncu ve oklar 10 kutu; fırlatılanlar, ender gözü, ateş topları ve
  rüzgâr yükü ilk tikte 3.5 bloktan yakında çizilmez, sonra ölçeksiz 4 kutu; öteki zarar veren
  mermiler ölçeksiz 4 kutu; zırh askısı (boş kutu bir blok sayılır); bağ düğümü 1024, shulker
  mermisi 16384, şamandıra ve havai fişek 4096 (entity'ye bağlı fişek hiç); yıldırım 64, çerçeve
  1024 kez ölçek; display görüş aralığı × 64 × ölçek; phantom ve trident her yerden; ışınlı end
  kristali her yerden.
- Görünürlüğü genişleten çiziciler: gardiyanın ışını (gözden hedefin ortasına kutu), end
  kristalinin ışını, shulker'ın kayışı (Java kayış ofsetini konum gibi kullanır; aynısı yapıldı).

## Kapsam ve dal kararı

Her ezme için bir satır (34 satır, 3 tik: ilk tik kuralı da sınanır). Çizici sınıfı Java'da
türden seçilir: ezmesi olan çiziciler kendi sınıflarıyla, öteki canlılar `PigRenderer`, öteki
entity'ler `NoopRenderer` ile (taban davranış). Görüş konisinin kendisi (`Frustum`) ve bölüm
solması bu kaydın dışında, kendi kayıtlarındadır.

## T47b güncellemesi

Eşya çerçevesi ve tablonun asılı kutusu (`HangingEntity.calculateBoundingBox`) `pick.rs`'e
eklendi: `ClientboundAddEntityPacket.getData()`'nın 3B yön değeri (`entity.data`, zaten eşya
çerçevesi çiziminde kullanılıyordu) ve konumun tabanı bloğa `0,46875` duvara çekilmiş merkezi,
yöne göre eksen inceltmesi; çerçevenin haritalı olması (`DataComponents.MAP_ID` eşya yamasında
var mı) 1×1'e, tablonun kayıt varyantının genişlik/yüksekliği kendi kutusuna büyütüyor (çift
boyda blok ızgarasına ortalamak için sola/yukarı kaydırmayla). Oracle'ın `cull` ekstrası artık
bir satırın isteğe bağlı `data` alanını (yön) gerçek `HangingEntity.setPos`/`setDirection`'a
geçiriyor; `cull-item-frame` (batı) ve `cull-glow-item-frame` (kuzey) satırları eklendi.

Mermi tiki: `ThrowableProjectile`'ın varsayılan yerçekimiyle (0,03) paylaşılan adımı artık
yumurta, inci, püskürtme/sürekli iksir için de çalışıyor; `ThrownExperienceBottle`'ın kendi
yerçekimi (0,07) ayrı. `AbstractArrow`'un adımı artık spectral ok ve trident için de çalışıyor
(mızrağın sadakat dönüşü hariç — sahibin konumunu izleyen aktif ivme, ayrı ve sunucu ağırlıklı
bir davranış, burada yok). `cull-trident` artık üç tik (önceden bir tikle sınırlıydı).

## Sapmalar (açık)

- Uzak oyuncu (`RemotePlayer`) oracle'da kurulamadığı için satırı yok; kural kaynaktan.
- Havai fişeğin bağlı olduğu entity verisi okunuyor ama oracle'da sınanmıyor (veri olayı yok).
- Tablonun kayıt dışı ("direkt") varyantı (ağda tam yazılan, kayıt kimliği olmayan) okunmuyor;
  yalnızca kayıt kimliğiyle gelen varyantlar (çizimde de aynı sınır var). Trident'in sadakat
  dönüşü (sahibine aktif ivme) istemci tikinde yok.
- Gölge, ad etiketi ve tasma çizgisi geçişleri kapsam dışı (T47).
