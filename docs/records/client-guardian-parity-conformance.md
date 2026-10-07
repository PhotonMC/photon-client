# T35h — Gardiyan ve elder gardiyan — [client-parity]

Bu kayıt, gardiyanın ve elder gardiyanın modelinin ve istemci tikinin (kuyruk ve diken animasyonu,
ışının sayacı) Java 26.2 ile aynı sonucu vermesini kapsar. Kanıt sınıfı `client-parity`'dir: model
gerçek `GuardianModel`'e, tik gerçek `Guardian` varlığına karşı karşılaştırılır. Canlı gözlem
`server-conformance`.

## Java kaynak incelemesi

- `GuardianModel.setupAnim`: baş bakışı alır; on iki diken `1 + cos(yaş × 1,5 + sıra) × 0,01 − geri çekilme`
  ile (geri çekilme `(1 − diken animasyonu) × 0,55`) kendi yerinin katına çekilir; göz, bakılanın yukarısında mı
  aşağısında mı olduğuna göre dikeyde, bakış yönüne göre yatayda kayar (`√|nokta| × 2 × işaret`); kuyruğun üç
  parçası `sin(kuyruk animasyonu)` ile `0,05π`, `0,1π`, `0,15π` döner. Elder katmanı 2,35 ile ölçeklenmiş bir ağdır;
  dikenlerin ve gözün yeri ölçeklenmez (referansın kendi tuhaflığı, aynen alındı).
- `Guardian.aiStep` (istemci, canlıyken): kuyruk hızı karada 2, suda hareketliyse 4 (zaten 0,5'ten hızlıysa 0,5'e
  yaklaşır), durgunsa 0,125'e yaklaşır; diken animasyonu karada rastgele, suda hareketliyse 0'a, durgunsa 1'e
  yaklaşır; ışın hedefi varsa sayaç saldırı süresine (gardiyan 80, elder 60) dek artar.
- `onSyncedDataUpdated(DATA_ID_ATTACK_TARGET)`: veride hedef geldikçe sayaç sıfırlanır.
- `Mob.baseTick` her tik aynı rastgele kaynaktan ortam sesi için bir değer çeker; karadaki diken değerleri bu
  yüzden her ikinci çekilişten gelir (bulgu: oracle ile eşleşmeden önce bu çekiliş eksikti).
- Veriler: hareket 16 (boolean), saldırı hedefi 17 (int, sıfır hedef yok).
- Kuyruğun başlangıcı yapıcıda tohumsuz kaynaktan çekilir; oracle'da sabitlenir (`guardian_random`).

## Kapsam

`entity/guardian-model` 26 satır (gardiyan ve elderde bakış, diken nefesi ve geri çekilmesi, kuyruk, göz: yukarı,
aşağı, aynı hizada, yan, tam karşı, kendi yerinde); `entity/variant-fields`'e iki satır;
`living-entity-animations`'a 8 satır (suda, hareket edip durma, karada, karada hareket, hedef alma ve yeniden
gönderilmesi, hedefi bırakma, elderin süresi).

## Sapmalar (açık)

- Işın çizilmez; hedefe bakış denetimi (`LookControl`, `setYRot(yHeadRot)`), ışının ve hareketin kabarcıkları ve
  karadaki çırpınış (`onGround` dalı: rastgele itme ve dönüş) yok. Bu dallar da aynı rastgele kaynaktan çektiğinden,
  karadaki diken değerleri yalnız bunlar çalışmıyorken Java ile eşleşir. Göz, bakışı hedefin göz konumuna çevirir
  ama baş hedefe dönmez. Hepsi T35i'de.
- Sesler, parçacıklar, gölge yok. Canlı gözlem yapılmadı.
- Doku yolları (`guardian`, `guardian_elder`) renderer sabitinden elle yazıldı, oracle ile karşılaştırılmaz.
