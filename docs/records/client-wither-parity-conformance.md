# T37g — Wither — [client-parity]

Bu kayıt, wither'ın iki modelinin (gövde, zırh) ve istemci durumunun (yan başların hedefe dönüşü, hareketle yüzünü
çevirmesi, eşzamanlı veriler) Java 26.2 ile aynı sonucu vermesini kapsar. Kanıt sınıfı `client-parity`'dir:
modeller gerçek `WitherBossModel`'e, durum gerçek `WitherBoss` varlığının tikine karşı karşılaştırılır. Çizim
ayarları (ölçek, doku seçimi, zırhın kaydırması, aydınlatma) oracle ile karşılaştırılmaz. Canlı gözlem
`server-conformance`.

## Java kaynak incelemesi

- `WitherBossModel.setupAnim`: yan başlar (`yHeadRots − bodyRot`, `xHeadRots`), kaburga (`(0.065 + 0.05·cos(yaş·0.1))·π`),
  kuyruk (yeri kaburganın açısından, açısı `(0.265 + 0.1·cos)·π`) ve orta baş (bakış) döner. Zırh katmanı aynı parçaları taşır.
- `WitherBoss` verisi (16 = gövdenin kendi hedefi, sunucuya ait; 17, 18 yan başların hedefleri; 19 dokunulmazlık tikleri).
- `WitherBoss.aiStep` (istemci): `super.aiStep`'ten önce hareketin y bileşeni 0.6 ile çarpılır ve yatay hız²
  0.05'i aşarsa `yRot` hareket yönüne çevrilir; sonra her yan baş için, hedef dünyada varsa hedefin gözüne dönülür
  (tikte en çok 40° dikey, 10° yatay; baş konumu gövde açısından), yoksa gövdeye. Açılar `atan2·(180/π)`'dir ve javac
  `180.0F / (float) Math.PI`'yi tek bir float sabitine katlar (`javap -c` ile doğrulandı).
- `WitherBossRenderer`: ölçek 2 (dokunulmazken `invulnerableTicks/220·0.5` kadar az), blok ışığı 15, doku dokunulmazken
  (tik > 80, ya da `tik/5` tek değilken) soluk doku; `WitherArmorLayer`: can yarıya inince `EnergySwirlLayer`, doku
  kayması `[cos(t·0.02)·3 % 1, t·0.01 % 1]`, renk 0xFF808080. Yan baş açıları aradeğerlenmez (ham değer çizilir).

## Kapsam

`entity/wither-model` 10 satır (iki modelde: durumsuz, ileri yaş, bakış, yan başlar, hepsi); `living-entity-animations`'a 6
satır (boşta, iki hedefe bakış ve dokunulmazlık verisi, hedefi yitirme, hedef tam yukarıda, hareketle dönüş, yavaş hareket).

## Sapmalar (açık)

- Parçacıklar (duman, etki), sesler, davranış, can çubuğu, kafatası mermileri yok. Hedef olarak yalnız canlı varlıklar
  aranır (`Level.getEntity` her varlığı bulur). Doku yolları renderer sabitinden elle yazıldı. Canlı gözlem yapılmadı.
- `WitherBoss.isPowered` için en çok can özniteliğinin varsayılanı (300) kullanılır, sunucu değiştirmediyse.
