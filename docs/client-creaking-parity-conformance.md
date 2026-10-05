# T37e — Creaking — [client-parity]

Bu kayıt, creaking'in iki modelinin (gövde, gözler) ve istemci durumunun (olaylarla ve tikle başlayan animasyon
durumları, yürüyüş hızı, bayraklar) Java 26.2 ile aynı sonucu vermesini kapsar. Kanıt sınıfı `client-parity`'dir:
modeller gerçek `CreakingModel`'e, durum gerçek `Creaking` varlığının tikine karşı karşılaştırılır. Gözlerin
yanıp sönmesi karşılaştırılmaz (aşağıda). Canlı gözlem `server-conformance`.

## Java kaynak incelemesi

- `CreakingModel.setupAnim`: bütün parçalar sıfırlanır; baş bakışı alır; hareket edebiliyorsa (`canMove`) yürüyüş
  (`applyWalk(konum, hız, 1, 1)`) uygulanır, edemiyorsa hiç uygulanmaz; sonra saldırı, "dokunulmaz" ve ölüm
  animasyonları (başlamışlarsa). Göz katmanı yalnız başı tutar, parça adları aynıdır.
- `Creaking.handleEntityEvent`: 66 dokunulmazlık sayacını 8, 4 saldırı sayacını 15 yapar; başkası `LivingEntity`'ye gider.
- `Creaking.aiStep`: iki sayaç tik başına birer azalır; `Creaking.tick` (istemci, tikin sonunda): saldırı ve
  dokunulmazlık durumları sayaç pozitifken çalışır, ölüm durumu `IS_TEARING_DOWN` iken çalışır (`animateWhen`).
- `updateWalkAnimation`: hedef hız `min(mesafe·25, 3)`, çarpan 0.4, konum ölçeği 1 (bebek ayrımı yok).
- Veri: `CAN_MOVE` (16, varsayılan açık), `IS_ACTIVE` (17), `IS_TEARING_DOWN` (18), `HOME_POS` (19, isteğe bağlı
  blok konumu; `isHeartBound` = konum var). `tickDeath` istemcide iki dalda da ölüm süresini birer artırır.
- `CreakingRenderer`: yıkılırken (`isTearingDown`) ölüm süresi sıfır, kırmızı bindirme yok, gözler
  `hasGlowingEyes`; değilse gözler `isActive`. Gözler `RenderTypes.eyes`, her zaman görünür.
- `checkEyeBlink` her istemci tikinde ölüm süresine ve varlığın tohumsuz rastgele kaynağına bakar; bu yüzden
  yanıp sönme Java'da da tekrarlanabilir değildir. Rust'ta varlık numarasından tohumlanan Java uyumlu bir kaynak
  kullanılır (aralıklar Java'nınki).

## Kapsam

`entity/creaking-model` 22 satır (gövde: bakış, yürüyüş dört hızda ve dondurulmuşken, üç animasyon erken ve geç,
birlikte; gözler: durumsuz, bakış-yürüyüş, saldırı); `living-entity-animations`'a 9 satır (boşta, saldırı, ikinci
saldırı, sallanma, yürüyüş ve koşu, donma, yıkılma, bağsız ölüm). Oracle'daki oyuncu uzakta durur: Java'daki
oyuncunun bağlantısı yoktur, itilebilir varlık araması onu çözemez.

## Sapmalar (açık)

- Gözlerin yanıp sönme zamanlaması karşılaştırılmaz (Java'da tohumsuz); `has_glowing_eyes` oracle'a yazılmaz.
- Kırılma sesleri, parçacıklar (`tearDown`), davranış ve kalp bloğu bağlantısı yok. Doku yolları renderer
  sabitinden elle yazıldı. Canlı gözlem yapılmadı.
