# T32g — Ravager — [client-parity]

Bu kayıt, ravagerin modelinin Java 26.2 ile Rust'ta aynı köşe konum, doku koordinatı ve normali
üretmesini; saldırı sallanmasının istemci sayacının aynı ilerlemesini kapsar. Kanıt sınıfı
`client-parity`'dir (model ve tik) ve canlı gözlemdir: gerçek `RavagerModel` katman tablosundan pişirilip
gerçek `setupAnim` ile pozlanır; sayaç gerçek `Ravager` varlığıyla ve gerçek varlık olayıyla
karşılaştırılır. Sersemleme ve kükreme sayaçları oracle'da koşulamaz (aşağıda); onlar için kanıt
birim testidir.

## Java kaynak incelemesi

- `RavagerModel.setupAnim`: saldırı varsa boyun öne atılır (`triangleWave(kalan, 10)` ile `((1+dalga)/2)³·12`),
  ağız beşten büyük kalanda `sin((kalan-4)/4)·π·0,4`, değilse `π/20·sin(π·kalan/10)` açılır; yoksa boyun
  dinlenir (sersemleyince `0,2199` eğik, ağız `0,05π`, boyun yana `sin(kalan/40·10)·3` salınır — çift
  duyarlıklı `Math.sin`), kükremede ağız `π/2·sin(kükreme·π/4)` açılır; baş bakışı izler, bacaklar `0,4`
  genlikle çaprazlama sallanır.
- `RavagerRenderer`: kalan süreler çizim kesriyle (`sayaç - kısmi`) küçülür; kükreme `(20 - sayaç + kısmi)/20`.
- `Ravager` (istemci): olay 4 saldırıyı 10, olay 39 sersemlemeyi 40 tike kurar (olay 69 kükremenin
  parçacık ve itme patlamasıdır, sayaç kurmaz); yaşarken `aiStep` kükreme, saldırı ve sersemlemeyi bir
  azaltır; sersemleme her tikte altıda bir bulut için çekim yapar ve bitince kükremeyi 20 tike kurar.

## Kapsam

`entity/ravager-model` 21 satır (duruş, bakış, yürüyüş, saldırının sekiz noktası, sersemlemenin dört
noktası, kükremenin beş noktası, saldırı + sersemleme); `living-entity-animations`'a üç satır (boşta,
bir ve iki kez sallanan).

## Sapmalar (açık)

- Sersemleme ve ardından kükreme Java oracle'da koşulamaz: sersemlemenin bulutu başsız istemcide
  eklenemez ve çekimleri tikten tike aynı sonucu vermez (tohumlu bile, çekim sayısı koşuya göre
  değişiyor). Sayaçların geri sayımı, kükremenin sersemlemeden sonra 20 tike kurulması ve ölü ravagerin
  durması `photon-world` birim testindedir; çekimler (altıda bir bulut, kükremede 120 gauss) Java kaynağına
  göre yapılır ama karşılaştırılamaz.
- Kükremenin çevreyi itmesi (`applyRoarKnockbackClient`, yerel oyuncuya fizik), parçacıklar ve sesler yok.
