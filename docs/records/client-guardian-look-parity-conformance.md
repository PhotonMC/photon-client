# T35j — Gardiyanın hedefe bakışı ve karadaki çırpınışı — [client-parity]

Bu kayıt, `Guardian.aiStep`'in T35h'de bırakılan dallarını kapsar: hedefe bakış (`LookControl`), kabarcık
parçacıklarının rastgele kaynaktan çekilişleri, karadaki çırpınış ve `Mob.baseTick`'in ortam sesi zarı.
Kanıt sınıfı `client-parity`'dir: baş, pitch, gövde, yaw, konum, hareket, `onGround` ve rastgele kaynağın
48 bitlik durumu her tikte gerçek `Guardian` tikinden okunur.

## Java kaynak incelemesi

- Hedef varken (`getActiveAttackTarget` dolu): saldırı süresi artar, `setLookAt(hedef, 90, 90)` ve `tick()` çağrılır.
  `LookControl.tick` önce pitch'i sıfırlar; başı hedefin yönüne, pitch'i hedefin göz hizasına tik başına en çok
  90° döndürür. Hedefle x ve z farkı `1,0E-5F`'in altındaysa baş, ikisinin de farkı altındaysa pitch dönmez.
  Yol yok sayılır (istemcide gezinme tamamlanmış sayılır), başı gövdeye kırpan dal çalışmaz.
- Bakış dönüşünün açı ölçeği `180F / (float)Math.PI`'nin **derleyicinin float olarak katladığı** değerdir
  (57,2957763671875). Dekompile kaynak bunu `× 180F / (float)Math.PI` diye gösterir ve double ile hesaplanırsa son
  bit sapar; doğrusu `javap -c` ile okundu. Pitch bu yüzden `−90` yerine `−89,99999`'a oturur.
- Kabarcıklar: suda ve hareket ederken iki tur üç `nextDouble` (x, y, z); hedef varken `nextDouble` ile başlayan,
  uzaklığa varana dek `1,8 − k + nextDouble × (1,7 − k)` adımlı döngü (k = saldırı ölçeği). Parçacıklar çizilmez,
  çekilişler tüketilir. Hedef, ilk bulunduğunda önbelleğe alınır ve veri güncellenene dek aynı kalır (dünya
  varlığı bırakırsa son konumunda).
- Suda değilken ve yerdeyken: üç `nextFloat` (x itişi, z itişi, yaw), `+0,5` dikey hız, `onGround` kapanır.
  Hedef varsa son adım `yaw = baş`.
- `Mob.baseTick`: `nextInt(1000) < ortamSesiSüresi++` tuttuğunda ses çalınır; sesin perdesi iki `nextFloat`
  çeker (susturulsa da), süre `−160`'a düşer. Ses çalınmaz, çekilişler tüketilir.

## Kapsam

`living-entity-animations`'a 8 satır (hedefe bakış, hareketli, yukarıdaki hedef, arkadaki hedef, elder,
veriyi yeniden gönderme, karada çırpınış, hedefle çırpınış); eski gardiyan satırları da artık rastgele kaynağı,
yön ve konumu karşılaştırır. Oracle sahnesine ikinci bir varlık (`second`) eklendi. Hedef boyutları için
`ClientEntity::dimensions` ortaklaştırıldı.

## Sapmalar (açık)

- Hedef olarak yerel oyuncu yalnızca birim testle denetlenir (oracle sahnesinde yerel oyuncu dünyada değildir);
  boyu pozdan okunur, ölçek niteliği uygulanmaz.
- Hasar, ölüm ve adım sesleri de perde için çekiliş yapar ve hasarda ortam sesi süresi sıfırlanır; bunlar henüz
  gardiyanın kaynağında yoktur (ses görevleri T60 ve sonrası).
- Parçacıkların kendisi, `setAirSupply` ve sunucu tarafı çırpınış (`needsSync`) yok.
