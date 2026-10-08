# T32f — Panda — [client-parity]

Bu kayıt, pandanın (yetişkin ve bebek) modelinin Java 26.2 ile Rust'ta aynı köşe konum, doku koordinatı
ve normali üretmesini; `PandaRenderer.setupRotations` dönüşlerinin (takla, oturma, sırtüstü) ve
`PandaHoldsItemLayer` matrisinin aynı olmasını; pandanın kendi istemci durumunun (sayaçlar, oturma,
sırtüstü, yuvarlanma miktarları, gen) aynı ilerlemesini; doku tablosunun Java'nın tablosuyla aynı olmasını
kapsar. Kanıt sınıfı `client-parity`'dir (model, dönüş ve katman matrisleri, tik ve tablo) ve canlı
gözlemdir: gerçek `PandaModel` ve `BabyPandaModel` katman tablosundan pişirilip gerçek `setupAnim` ile
pozlanır; dönüşler gerçek render sınıflarının `setupRotations` yöntemiyle; tik gerçek `Panda` varlığıyla
ve gerçek veri/ekipman/hava paketleriyle karşılaştırılır.

## Java kaynak incelemesi

- `PandaModel.setupAnim`: dört ayaklı poz sonra: mutsuzken baş ve ön bacaklar sallanır; hapşırırken baş
  `sneezeTime` ile (15'ten önce doğrusal, sonra tam sayı bölmeyle 0 ya da 1 adımı) iner; oturunca
  `animateSitting` (yetişkinde gövde ve baş `rotLerpRad`, bebekte gövde/baş/bacak konumları `lerp`),
  yeme ve korku başı/ön bacakları ezer; sırtüstü ve yuvarlanma baş ve bacak sallar.
- `PandaRenderer.setupRotations`: takla `rollTime`'a göre dört çeyrek dönüşe bölünür (tam tikler
  arasında doğrusal geçiş, bebek ve yetişkin için farklı yükseklik); oturma yukarı çıkarır, pitch'ten
  `+90°` döndürür, korkunca `Math.cos`'la (çift duyarlıklı) titretir; sırtüstü yükseltip `+180°`
  döndürür. Yuvarlanma miktarı bebekte hep sıfırdır.
- `PandaHoldsItemLayer`: oturan ve korkmayan pandanın tuttuğu eşya önüne `(0,1; 1,4; -0,6)` konur, yerken
  `sin(yaş·0,6)` ile sallanır ve geri çekilir.
- `Panda.tick` (istemci): çalışan sayaçları yerel veriye yazar: mutsuzluk her tikte azalır, hapşırma
  sayacı artar ve 20'yi geçince bayrak ve sayaç sıfırlanır, takla sayacı artar ve 32'yi geçince biter
  (yoksa sıfırlanır), oturan panda eğimini sıfırlar; `Mth.lerp` ile oturma (`+0,15`/`-0,19`), sırtüstü ve
  yuvarlanma miktarları; endişeli gen (`worried`) fırtınada oturur ve yemeyi bırakır, fırtına dinince
  (yemiyorsa) kalkar; oturan ve eşyası olan panda her tikte `nextInt(80) == 1` çekimiyle yemeye başlar,
  yerken sayaç artar. Gen: ana ve gizli gen; çekinik (kahverengi, zayıf) ana gen yalnız gizli aynıysa
  görünür, bilinmeyen kimlik normaldir.
- Veri: `UNHAPPY_COUNTER` 18, `SNEEZE_COUNTER` 19, `EAT_COUNTER` 20 (int), `MAIN_GENE_ID` 21,
  `HIDDEN_GENE_ID` 22, `DATA_ID_FLAGS` 23 (byte; 2 hapşırık, 4 yuvarlanma, 8 oturma, 16 sırtüstü).
- `Level.isThundering`: hava alabilen boyutta (gökyüzü ışığı var, tavan yok, End değil) gök gürültüsü ×
  yağmur seviyesi 0,9'u aşar. Bu seviyeler artık oturumdan dünyaya `set_weather` ile verilir.

## Kapsam

`entity/panda-model` 40 satır (iki katman × duruş, bakış, yürüme, mutsuz, hapşırığın altı noktası,
oturma, yeme, korku, sırtüstü, yuvarlanma ve karışımları); `entity/panda-textures` 14 satır;
`entity/setup-rotations` 84 satır (panda: takla on altı nokta, oturma, yeme sallanması, korku, sırtüstü,
karışım; eşya matrisi; ayrıca kedinin yatışı ve tilkinin eğimi: önceki dilimlerin dönüşleri de burada
Java ile karşılaştırılır); `living-entity-animations`'a dokuz panda satırı (boşta, mutsuz, hapşıran,
yuvarlanan, oturup sırtüstü yatan, kendiliğinden yemeye başlayan, eşyası gidince duran, fırtınada oturan
endişeli, çekinik gen); `entity/variant-fields`'a altı satır.

## Sapmalar (açık)

- Başsız istemci parçacık ekleyemez: hapşırığın bitişi (`afterSneeze` parçacığı) ve yemenin beşinci
  tikinden sonrası (yeme parçacıkları ve sesi) Java oracle'da koşulamaz. Bu kısımlar `photon-world`
  birim testlerindedir (bitiş, takla süresi, yemenin başlayıp bitmesi). Parçacıkların harcadığı rastgele
  sayılar (yemede beş tikte bir 39 çekim) Java koduna göre kaynağın yerinde tutulması için çekilir ama
  karşılaştırılamaz.
- Pandanın suda olması (`isInWater`) izlenmiyor: endişeli panda fırtınada suda da oturur.
- `afterSneeze`ın yakın pandaları zıplatması (sunucu fiziği) ve düşürdüğü eşya yok; ses ve parçacık yok.
- Pandanın çarpışma kutusu bebek/yetişkin dışında poz taşımaz (oturma, sırtüstü kutuyu değiştirmez).
