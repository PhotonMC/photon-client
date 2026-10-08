# T76b — `Explode` paketi ve patlama izleyicisi — [client-parity]

Bu kayıt `ClientPacketListener.handleExplosion`'ı ve `ClientExplosionTracker`'ı kapsar. Kanıt sınıfı
`client-parity`'dir. `movement/particle-engine`'de gerçek `handleExplosion`'a paketler verilir ve her
tik motordan önce izleyici tiklenir. Ses, merkez parçacığı ve dağılan blok parçacıkları tohumlu motorla
bit bit eşitlenir. Paket okuyucu birim testlidir.

## Java kaynak incelemesi

- Paket sırayla merkezi, yarıçapı, blok sayısını, isteğe bağlı oyuncu itmesini, merkez parçacığını
  (`ParticleTypes.STREAM_CODEC`), ses tutucusunu ve `ExplosionParticleInfo`'ların ağırlıklı
  listesini taşır. Her bilginin parçacığı, ölçeği, hızı ve ağırlığı vardır.
- İşleyici sesi `BLOCKS`, 4 düzeyle çalar. Perde `(1+(r−r)*0.2)*0.7` seviye rastgelesindendir,
  tohum ardından çekilir. Merkez parçacığı `(1, 0, 0)` itişle eklenir. Liste boş değilse patlama
  izleyiciye kaydedilir; itme oyuncunun hızına eklenir (`addDeltaMovement`).
- İzleyici seviye tikinde, seçenek "tümü" değilse temizlenir. Değilse toplam blok sayısı kadar
  (en çok 512) parçacık çıkar. Her biri blok sayısıyla ağırlıklı seçilen bir patlamadan gelir.
  Yön birim küpten normalleştirilir, uzaklık `cbrt(r)*yarıçap`'tır; yer havaysa hız
  `0.5/(uzaklık/yarıçap+0.1)*r*r+0.3` olur. Tür ağırlıkla seçilir, yer ölçekle, hız türün hızıyla
  çarpılır.

## Kapsam ve dal kararı

Tek satır: iki patlama aynı tikte, 900 bloklu biri (512 sınırı), türsüz biri. Patlama seviye
olaylarıyla aynı kuyruğa girer, böylece rastgele çekiş sırası korunur; izleyici animasyon tikinden
önce koşar.

## Sapmalar (açık)

- Okunmayan seçenekli parçacık taşıyan paket atlanır (itme dahil).
- Doğrudan (kayıtsız) ses tutucusu çalınmaz. Çığlık ve elder gardiyan parçacıkları T76c'de.
