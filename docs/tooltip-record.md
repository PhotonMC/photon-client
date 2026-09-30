# P22.e.1 — Eşya ipucu, metin bileşeni ve yaratıcı arama — oracle tabloları ve canlı oturum kaydı

**Tarih:** 2026-09-29 · **Referans:** Java Edition 26.2 / protokol 776 / Java 25
**Kanıt sınıfı:** tablolar `client-parity` (oracle, her koşuda karşılaştırılır);
metin ve ekran canlı gözlem

## Metin bileşeni ve dil

`crates/voxel-game/src/text.rs` bileşeni (`Component`: düz metin ya da çeviri anahtarı ve
argümanlar, stil, kardeşler) hem veri JSON'undan hem ağın etiketinden
(NBT) okur; `Lang` `assets/minecraft/lang/en_us.json`dan anahtarları biçim
metnine çevirir (`%s`, `%d`, `%n$s`, `%%`) ve bileşeni stilli parçalara
(`Run`) böler. Dil dosyası bu iş için yazılmış düz bir okuyucuyla okunur:
genel JSON okuyucu 7 bin anahtarda 3,8 saniye tutuyordu, bu 2 milisaniye.
Okunmayanlar: tuş bağı, skor, seçici, NBT içerik, tıklama/üzerine gelme
olayları, yazı tipi ve gölge rengi.

Yazı çizimi (`GuiFrame::styled_text`) parça başına rengi okur; kalın
harfler bir birim kaydırılıp iki kez çizilir (ilerleme +1), eğik harfler
`1 − 0,25·yükseklik` kaymasıyla kesilir. Altı çizili, üstü çizili ve karışık
(obfuscated) yazı çizilmez.

## Eşya ipucu

`crates/voxel-game/src/tooltip.rs` `ItemStack.getTooltipLines`ı yığının bileşen değerleriyle
ve eşyanın varsayılanlarıyla kurar: ad (özel ad, kitap başlığı, iksir, ok,
kalkan, oyuncu kafası, pusula; nadirlik rengi, özel ad eğik), tablo, disk
parçası, keçi borusu, havai fişek ve yıldız, iksir ve kötü alâmet şişesi
etkileri (süre, genlik, öznitelik satırları), plak, zırh süsü, büyüler
(ipucu sırası etiketi, lanet kırmızı), boya, açıklama, öznitelik
değiştiriciler (oyuncu tabanı `8 Attack Damage` yeşil satırıyla), kırılmaz,
bayrak desenleri, şüpheli yahni ve arı kovanı balı. Ekran, imleç boşken
üstünde durulan yığın için ipucu çizer (`TooltipRenderUtil`: dokuz
dilimli `tooltip/background` ve `tooltip/frame`, satır 10 birim, ilk
satırdan sonra 2, tek satır 2 kısa, fareden (12, −12), ekrana sığdırma).
Yaratıcı ekranda yığının bulunduğu sekmelerin adları adın altına mavi eklenir
(tek kategori sekmesinin kendi penceresi hariç) ve sekme düğmesinin üstünde
sekmenin adı çıkar.

Bileşen değerlerini oracle tabloları besler (`block/tooltip-registries`,
`block/potions`): büyüler (açıklama, lanet, en yüksek düzey, ipucu sırası),
tablolar, enstrümanlar, plaklar, bayrak desenleri, süs malzemeleri ve
desenleri, öznitelikler (anahtar, artış/azalış rengi), mob efektleri
(renk, açıklama anahtarı, kategori rengi, öznitelik değiştiricileri) ve
iksirler; hepsi kayıt numarasıyla (`crates/voxel-data/src/item/tooltip.rs`,
`crates/voxel-data/src/item/potions.rs`, `tools/generate/`) ve adaptörle her koşuda
referanstan karşılaştırılır.

## Yaratıcı arama

Arama sekmesinin kutusu `EditBox`ın (`crates/voxel-render/src/gui/edit_box.rs`) taşıdır: yazı,
imleç, seçim, imleci görünür tutan kayma, Backspace/Delete/oklar/Home/End
(Ctrl ile kelime), hepsini seç/kopyala/yapıştır/kes (pencere panosuyla),
elli karakter. `T` arama sekmesini açar ve o tuşun harfi yazılmaz. Arama
`SessionSearchTrees.creativeNameSearch`: arama sekmesinin eşyalarının ipucu
satırları (biçimsiz, kırpılmış, küçük harf) içinde alt dize; `ad:yol` iki
parçalı sorgu isim alanı ve yol üzerinden kimliğe ya da satırlara bakar;
sonuçlar arama sekmesinin sırasında.

## Canlı sonuç

Vanilla 26.2 sunucusunda yaratıcı ekranda: `Arrow of Swiftness / Speed II
(00:11) / When Applied: +40% Speed` gibi ipuçları; arama kutusuna `diamond`
yazınca elmas eşyalar listelendi.

## Açıklar

- `#etiket` araması sonuç vermez: istemci eşyaların etiketlerini
  tutmuyor. Arama sekmesinin etiket satırları da yok.
- Simya şablonu, harita, kap, kitap ve arı satırları, gelişmiş (F3+H) satırlar
  ve macera modu yüklemleri yok.
- Aramada kutuya fareyle tıklama, çift tıklamayla kelime seçme ve IME yok;
  seçim vurgusu referansın mantıksal işlemi yerine yarı saydam mavi.
- Büyü sırası, ipucu sırası etiketinde olmayanlar için telden geldiği gibi;
  referans bunları karma tablo sırasıyla yazar.
- Dil yalnız `en_us`.
