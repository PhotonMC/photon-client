# T32e — Tilki — [client-parity]

Bu kayıt, tilkinin (yetişkin ve bebek) modelinin Java 26.2 ile Rust'ta aynı köşe konum, doku koordinatı
ve normali üretmesini; ağzında tuttuğu eşyanın (`FoxHeldItemLayer`) matrisinin aynı olmasını; baş
yuvarlanması ve çömelme miktarının aynı istemci ilerlemesini; doku tablosunun Java'nın tablosuyla aynı
olmasını kapsar. Kanıt sınıfı `client-parity`'dir (model, katman matrisi, tik ve tablo) ve canlı
gözlemdir: gerçek `AdultFoxModel` ve `BabyFoxModel` katman tablosundan pişirilip gerçek `setupAnim` ile
pozlanır; tilkinin tiki gerçek `Fox` varlığıyla ve gerçek veri paketleriyle karşılaştırılır.

## Java kaynak incelemesi

- `FoxModel.setupAnim`: yürüme pozu (baş yuvarlanması, bacaklar görünür; yetişkinde bacaklar çaprazlama,
  bebekte `FOX_BABY_WALK` anahtar kare animasyonu `applyWalk(konum, hız, 1, 2,5)`), sonra sırayla çömelme,
  uyku, oturma (biri varsa, bu öncelikle), zıplama (yalnız yetişkinde `crouchAmount/2` kadar iner), baş
  bakışı (uyurken, yüzüstüyken ve çömelirken değil), uyurken baş yatışı ve `cos(yaş·0,027)/22`
  sallanması, yüz üstüyken bacakların `legMotionPos` ile sallanması. `legMotionPos` modelin kendi
  sayacıdır, yüz üstü her pozda `0,67` artar (Java'da bütün tilkiler tek modeli paylaşır; burada da
  katman başına tek model).
- `AdultFoxModel` / `BabyFoxModel`: oturma, uyku ve çömelmenin yetişkin ve bebek kaymaları farklıdır
  (bebekte `ageScale` ile ölçeklenir, uykuda kuyruk açısı ikinci atamayla `-2,1816616` olur).
- `Fox`: `DATA_TYPE_ID` 18 (int; kırmızı 0, kar 1, bilinmeyen kırmızı), `DATA_FLAGS_ID` 19 (1 oturma, 4
  çömelme, 8 ilgili, 16 atlıyor, 32 uyuyor, 64 yüz üstü). `isSleeping` ve `isCrouching` bayrağı okur
  (poz değil). İstemci `tick`: ilgi açısı hedefe `0,4` oranında yaklaşır, çömelme miktarı çömelirken
  tikte `0,2` artar (en çok beş) ve çömelmeyince hemen sıfırlanır; baş yuvarlanması
  `lerp(ilgi)·0,11·π`.
- `FoxRenderer`: doku çeşide, uykuya ve yaşa göre; atlarken ya da yüz üstüyken gövde `-pitch` kadar X
  ekseninde döner.
- `FoxHeldItemLayer`: eşya başın konumuna (başın dönüşü olmadan) taşınır; bebekte `0,75` ölçek; baş
  yuvarlanması, baş dönüşü ve bakış pitch'i; sonra ağız konumu (yetişkin/bebek, uyku/uyanık dört
  ayrı kayma); yatık `90°` (uykuda ek `90°`).

## Kapsam

`entity/fox-model` 26 satır (iki katman × duruş, bakış, yürüme, ilgi, çömelme, uyku, oturma, zıplama,
yüz üstü; her satırda kuş bakışı eşya matrisi); `entity/fox-textures` 8 satır (çeşit × uyku × yaş);
`living-entity-animations`'a beş tilki satırı; `entity/variant-fields`'a iki satır.

## Sapmalar (açık)

- `fox-model` yüz üstü satırları modelin ilk pozunu karşılaştırır (`legMotionPos` sıfırdan başlar); canlı
  çizimde sayaç kare başına artar ve Java'daki gibi kareye bağlıdır.
- Tilkinin yüz üstü yere toprak parçacıkları, yeme olayı (45) parçacıkları, sesler yok.
- `Fox.isSleeping` bayrağı ve genel `ClientEntity::is_sleeping` (yatak) ayrıdır; yatakta uyuma yalnız
  oyuncular içindir, tilki için yalnız bayrak kullanılır.
- Güvenilen varlıklar (`DATA_TRUSTED_ID`) çizimi etkilemez; okunmaz.
