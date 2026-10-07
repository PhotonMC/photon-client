# T36 — Blaze, ghast ve phantom — [client-parity]

Bu kayıt, blaze'in, ghast'ın ve phantom'un modelinin ve renderer'larının pozu nasıl kurduğunun (kafa ve
çubuk halkaları, dokunaçlar, kanat ve kuyruk, phantom'un ölçeği ve eğimi) Java 26.2 ile aynı sonucu
vermesini kapsar. Kanıt sınıfı `client-parity`'dir: modeller gerçek `BlazeModel`, `GhastModel` ve
`PhantomModel`'e, phantom'un duruşu gerçek `PhantomRenderer.setupRotations` ve `scale`'e karşı karşılaştırılır.
Canlı gözlem `server-conformance`.

## Java kaynak incelemesi

- `BlazeModel.setupAnim`: on iki çubuk üç halkada başın çevresinde döner (dört, dört, dört; açı hızları
  `−0,1π`, `0,03π`, `−0,05π` yaşla; yarıçaplar 9, 7, 5; yükseklikler −2, 2, 11) ve her biri kendi fazıyla `cos` ile
  inip çıkar; baş bakışı alır. Renderer çevredeki bloklara bakmadan blok ışığını on beş yapar.
- `GhastModel.setupAnim`: dokuz dokunaç `0,2 sin(0,3 yaş + sıra) + 0,4` ile salınır. Katman 4,5 ile ölçeklenmiş
  bir ağdır; dokunaç uzunlukları sabit tohumlu (1660) kaynaktan gelir ve oracle'dan yakalanan tabloda durur.
  Doku, ateş topu yüklenirken (`DATA_IS_CHARGING`) ağzı açık olanıdır.
- `PhantomModel.setupAnim`: kanatlar `cos(uçuş zamanı × 7,448451°) × 16°` ile birlikte çırpılır (sağ kanat
  sola karşıt); kuyruk iki katı hızla `−(5 + 5 cos(2 anim))°`. Uçuş zamanı `varlık numarası × 3 + yaş`.
- `PhantomRenderer`: ölçek `1 + 0,15 × boyut`, ardından `(0, 1,3125, 0,1875)` kadar kaldırma; dönüşlerin en
  sonunda eğimi kadar x ekseninde döner. Gözleri `PhantomEyesLayer` ile ışıksız çizilir.
- Veriler: ghast yükleme bayrağı 16 (boolean), phantom boyutu 16 (int).

## Kapsam

`entity/airborne-model` 14 satır (üç model dört yaşta; blaze'in bakışı; phantom'un büyük uçuş zamanı);
`entity/phantom-rotations` 5 satır (küçük, büyümüş, tam boy, eğik, ölürken; hem dönüşler hem
`scale` ve kaldırma dahil tam duruş); `entity/variant-fields`'e iki satır.

## Sapmalar (açık)

- Blaze'in blok ışığı on beş sabiti oracle ile karşılaştırılmaz. Phantom'un gözlerinin ve ghast'ın yükleme
  dokusunun yolları renderer sabitinden elle yazıldı. Phantom'un boyutu `refreshDimensions`'ı, yani
  sınırlayıcı kutusunu büyütür; kutu burada izlenmez. Gölge, ses, parçacık, ateş topu yok. Canlı gözlem yapılmadı.
