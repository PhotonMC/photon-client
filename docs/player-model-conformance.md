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

## Kapsam dışı

Eldeki eşyanın kol pozları, eşya kullanımı, binek, elytra, pelerin, zırh ve
bebek modeli.
