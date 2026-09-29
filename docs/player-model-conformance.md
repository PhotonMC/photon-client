# Oyuncu modeli

Kanıt sınıfı: `client-parity`.

## Sözleşme

Oyuncu modeli referanstaki oyuncu ağından kuruluyor: baş ve şapka, gövde ve
ceket, iki kol ve kolluklar (geniş ya da ince kol), iki bacak ve paçalar.
Her kutunun yüzleri 64x64 deride referansın küp düzeniyle koordinat alıyor.
Aynalanmış kutuda x kenarları yer değiştiriyor, her yüzün köşeleri ters
sırayla diziliyor ve x normali dönüyor.

Poz her seferinde ağdaki başlangıç konumlarından kuruluyor:

- Baş bakışa ve gövdeye göre dönüşüne çevriliyor.
- Kollar ve bacaklar yürüme animasyonunun konumu ve hızıyla sallanıyor.
- Kollar yaşla birlikte hafifçe nefes alıyor.
- Eğilen oyuncunun gövdesi öne yatıyor, başı ve kolları iniyor.
- Vuruş, gövdeyi ve vuran kolu referansın "whack" eğrisiyle çeviriyor.
- Yüzen oyuncunun kolları kulacın üç evresinden geçiyor, bacakları çırpıyor.
- İzleyicide yalnızca baş kalıyor.

Boş eller, binek yok, elytra yok.

Parçalar referansın matematik kütüphanesinin sırasıyla ve yuvarlamasıyla
dönüştürülüyor (`src/render/joml.rs`): çarpma-toplamalar tek tek yuvarlanıyor,
matrisin özellik bayrakları hangi yolun seçildiğini belirliyor.

## Senaryolar

`tools/behavior/cases.py` içindeki `entity_cases`. İki kol genişliğinde 15
poz: duruş, nefes, yürüme, etrafa bakma, eğilerek yürüme, iki kolla vuruşun
başı/ortası/sonu, kulacın her evresi, vururken yüzme, izleyici.

Referans modeli kendi `setupAnim`'iyle pozluyor, kendi parça çizicisiyle
bir tüketiciye çiziyor. Her dörtgenin köşe konumu, doku koordinatı ve
normali f32 bitleriyle karşılaştırılıyor. Parçalar bir hash tablosunda
durduğu için çizim sırası sözleşmenin parçası değil; dörtgenler sıralanıp
karşılaştırılıyor.

## Sonuç

`python3 tools/behavior/run.py compare --family entity` → `parity_passed`.

## Dünyada çizim ve üçüncü şahıs kamerası

F5 birinci şahıs, arkadan ve önden üçüncü şahıs arasında geçiyor
(`VOXEL_CAMERA=back|front` o kipte başlatıyor).

- **Önden görünüm:** Yön 180° dönüyor, bakış eğimi ters çevriliyor.
- **Geri çekilme:** Kamera gözden bakışın tersine 4 blok çekiliyor. Göz
  çevresindeki küçük bir kutunun sekiz köşesinden atılan ışınlar bloğa
  çarparsa daha yakında kalıyor (`Camera.getMaxZoom`). Görsel şekil yerine
  çarpışma şekli kullanılıyor; bunlar toz kar, bariyer ve ışık bloğu
  dışında aynı.

Oyuncu yalnız kamera gözden ayrılınca çiziliyor.

**Skin:** Profilin kendi skini henüz okunmuyor; referanstaki gibi profil
kimliğinin hash'ine göre 18 varsayılan skinden biri ve onun kol genişliği
seçiliyor.

**Yerleştirme** referansın canlı çizicisindeki sırayla yapılıyor:

- Gövde yönünün tersine dönme.
- Yüzerken öne yatma.
- Ters çevirme.
- 15/16 ölçek.
- 1,501 blok indirme.
- Eğilince 2/16 blok daha alçalma.

**Işık ve çizim:**

- Varlık shader'ı her köşenin normaline göre seviyenin iki yaygın ışığını
  yüzün önü ve arkası için ayrı hesaplıyor.
- Ardından sırasıyla dokunun onda biri altındaki alfa kesiliyor, ışık
  haritası uygulanıyor (gözün bloğundaki blok ve gök ışığı) ve sis
  ekleniyor.
- Karışım saydam, arka yüz ayıklaması kapalı. Çizim sırası: opak araziden
  sonra, sudan önce.

Bu kısım referans karelerle karşılaştırılmadı. Nether'in ışık yönleri,
vurulma kızarıklığı ve isim etiketi yok.

## Birinci şahıs el ve vuruş

Birinci şahısta boş sağ el çiziliyor; izleyicide çizilmiyor. El görünüm
uzayında kuruluyor:

1. Bakış sallanması uygulanıyor.
2. El bakışın gecikmeli kopyasının (`xBob/yBob`, her tik farkın yarısını
   kapatıyor) arkasından farkın onda biri kadar dönüyor.
3. Referansın boş el dönüşüm zinciri uygulanıyor.
4. Kol, ağdaki konumunda 0,1 radyan dışa açılmış olarak kolluğuyla
   çiziliyor.

Elin kendi projeksiyonu var (görüş 70°, suda daralıyor; yakın 0,05, uzak
100) ve derinlik temizlenip en son çiziliyor. `xBob/yBob` movement
ailesinde her tick bit bit eşleşiyor.

**Vuruş:**

- Sol tık (ya da betikte `attack`) bir sonraki tikte oyuncudan önce
  işleniyor ve sunucuya `Swing` gidiyor.
- 4,5 bloklük erişimde blok yoksa ıska sayılıyor: hayatta kalmada 10 tik
  bekleme başlıyor, saldırı gücü ve eşya değiştirme sayacı sıfırlanıyor.
  Blok varsa yalnızca kol sallanıyor; blok kırma henüz yok.
- Boş elin sallanması altı tik sürüyor (`updateSwingTime`). İlk yarısında
  yeni vuruş sallanmayı baştan başlatmıyor.
- El, eşya değiştirme ölçeğinin küpüne doğru tik başına en fazla 0,4
  yükseliyor. Girişte ve ıskadan sonra aşağıdan kalkıyor.
- Vuruş hem birinci şahıs elinde hem üçüncü şahıs modelinde oynuyor.

Bu kısım referans karelerle karşılaştırılmadı.

## Kapsam dışı

Eldeki eşyanın kol pozları, eşya kullanımı, binek, elytra, pelerin, zırh ve
bebek modeli.
