# T37h4 — Ender ejderhasının parçaları — [client-parity]

Bu kayıt, ejderhanın sekiz parçasının (baş, boyun, gövde, üç kuyruk, iki kanat) konumlarının ve ölürken ejderhanın
yükselişinin Java 26.2 ile aynı sonucu vermesini kapsar; hedeflemenin parçalara inmesi bir birim testle denetlenir. Kanıt
sınıfı `client-parity`'dir (konumlar gerçek `EnderDragon` tikine karşı); hedefleme kısmı oracle'sızdır (aşağıda). Canlı gözlem
`server-conformance`.

## Java kaynak incelemesi

- `EnderDragonPart`: `isPickable` doğru, boyutu kendi `(genişlik, yükseklik)`'idir (baş 1×1, boyun 3×3, gövde 5×3, kuyruklar
  2×2, kanatlar 4×2); ejderhanın kendisi `isPickable` değildir. Parça kimlikleri `recreateFromPacket`'te `ejderhaKimliği + sıra + 1`.
- `EnderDragon.aiStep` (istemci) sonunda `tickPart`: konum = ejderhanın konumu + kayma; kayma gövde ve kanatlarda yönünden
  (`yaw`), baş ve boyunda yön ile tırmanıştan (`sin/cos(eğim)`, uçuş geçmişinin 5. ve 10. örneğinin yükseklik farkı), kuyruklarda geçmişin
  12, 14, 16. örneklerinden gelir (oturuyorsa başın yüksekliği −1). `yRotA` yalnız sunucuda değişir (istemcide sıfır). Parçalar ancak
  ejderha ölmüyorsa ve yapay zekâsız değilse yerleştirilir (ilk tikten önce orijindedirler).
- `EnderDragon.tickDeath`: ölürken her tik `move(0, 0.1F, 0)` ile ejderha ve parçaları 0.1 yükselir; konum yaklaşımı durur.
- `Level.getEntities(except, kutu, seçici)`: önce varlıklar, sonra ejderha parçaları (`parentMob != except`, `seçici` ve kutu kesişimi);
  `ProjectileUtil.getEntityHitResult` hepsini aynı dalla tarar. Parçanın tıklama sonucu parçanın kimliğiyle, parçanın konumu ve
  kutusuyla döner (saldırı ve etkileşim paketlerinin hedefi).

## Kapsam

`living-entity-animations`'taki ejderha satırlarının her tikinde sekiz parçanın konumu (ölen satırlarda yükseliş dahil). Birim test:
tikleyen bir ejderhanın başına önden nişan başı (kimlik 101) bulur, ejderhanın kendisini ve uzaktaki boşluğu bulmaz.

## Sapmalar (açık)

- Parça kimlikleri oracle sahnesinde `Entity.setId` ile kurulduğu için Java'da `recreateFromPacket` sırası görülmez; kimlik kuralı
  kaynaktan okundu. Nişan, `GameRenderer.pick`'in gerçek bir ejderhayla koşusuyla karşılaştırılmadı (varlık sahnesi ejderhayı
  tikletmiyor). Aynı uzaklıkta eşit mesafedeki iki parçadan hangisinin seçileceği Java'da liste sırasına bağlıdır, burada eşleme sırasına.
