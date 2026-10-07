# T37h2 — Ender ejderhasının ölümü — [client-parity]

Bu kayıt, ölürken ender ejderhasının çizimini kapsar: ışınların geometrisi Java 26.2'nin kendi `submitRays`'ine karşı,
`dragonDeathTime` sayacı gerçek `EnderDragon` tikine karşı karşılaştırılır (T37h kaydı). Kanıt sınıfı
`client-parity`'dir; çözünme ve ışınların boru hattı ayarları oracle ile karşılaştırılmaz. Canlı gözlem `server-conformance`.

## Java kaynak incelemesi

- `EnderDragonRenderer.submit`: `deathTime = dragonDeathTime > 0 ? dragonDeathTime + kısmiTik : 0`. Sıfırdan büyükse gövde
  `entityCutoutDissolve` ile çizilir: renk beyaz, alfa `1 − deathTime/200` (255'te birin tamı), bindirme yok; her texel,
  köşenin alfası maske dokusunun (`dragon_exploding.png`) aynı texelinin alfasından küçükse atılır (gölgelendirici `DISSOLVE`,
  ardından köşe alfası 1 sayılır, kesme eşiği 0.1 doku alfasına uygulanır). Gözler her durumda çizilir.
- Işınlar: gövdenin `translate(0, −1, −2)`'sinden sonra, `deathTime/200` ile. Işın sayısı `floor((t + t²)/2 · 60)`; tohumu
  432 olan rastgele kaynaktan her ışına altı açı (iki `rotationXYZ`/`rotateXYZ`, ikincinin z'sine `t·π/2` eklenir),
  uzunluk `5 + 20r + aşırıHız·10`, genişlik `1 + 2r + aşırıHız·2`; `aşırıHız` son beşte birde 0'dan 1'e çıkar. Her ışın üç
  üçgendir (merkez, iki dış köşe), merkez rengi beyaz ve alfası `1 − aşırıHız`, dış köşeler alfasız mor (0x00FF00FF); poz her
  ışında birikir (`pose.rotate`), köşeler o anki pozla yerleşir. Boru hattı (`DRAGON_RAYS`): şimşeğin gölgelendiricileri,
  `SRC_ALPHA, ONE` karışımı, derinliğe yazar, sis rengi yerine rengi sis oranınca azaltır.
- `JOML Quaternionf.rotateXYZ` Rust'a eklendi (kaynağı javap/Vineflower ile okundu).

## Kapsam

`entity/dragon-rays` 12 satır (ışınsız, çok az, az, orta, yarım, dörtte üç, son beşin kıyısı, aşırı hız, bitmek üzere,
bitti, bittikten sonra, kaydırılmış pozdan; her satır köşelerin konum, renk ve sırasını Java ile karşılaştırır).

## Sapmalar (açık)

- Çözünme gölgelendiricisi, ışın boru hattı ve karışım ayarları yalnız derlenme ve oyunun başlaması düzeyinde denetlendi;
  ölen ejderha canlı gözlemlenemedi (sunucuda `kill` ejderhayı öldürmüyor, `Health` ile öldürülen ejderha ekrana alınamadı).
  Maske, aynı atlasta, doku farkı olarak itme sabiti (push constant) ile örneklenir; maske ve kaplama aynı boyda olmalıdır.
- Patlama parçacıkları ve sesleri yok.
