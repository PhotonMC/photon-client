# P22.d.1 — Envanter ekranı — canlı oturum kaydı

**Tarih:** 2026-09-29 · **Referans:** Java Edition 26.2 / protokol 776 / Java 25
**Kanıt sınıfı:** canlı gözlem; tıklamaların kendisi
[P13.a client-parity kaydında](inventory-click-conformance.md)

## Ne yapıldı

`crates/voxel-render/src/gui/screen/inventory.rs`, hayatta kalma envanteri
(`InventoryScreen`, tarif kitabı kapalı) ve `AbstractContainerScreen`'in
girdisi:

- E ile açılır (yaratıcı ve izleyici hariç), fare bırakılır ve pencerenin
  ortasına konur; E ya da Esc ile kapanır: `container_close` paketi,
  `InventoryMenu.removed`'ın sonuç yuvasını boşaltması, fare yeniden tutulur.
  Ekran açıkken tuşlar ve fare ekrana gider, oyuncu durur.
- Fare: basınca imleç boşsa alma/shift/pencere dışına atma, doluysa sürükleme
  başlar; bırakınca koyma ya da sürüklenen yuvalara `QUICK_CRAFT` adımları;
  250 ms içinde aynı düğme ve aynı yuva çift tıktır (`PICKUP_ALL`, shift ile
  aynı eşyaları taşıma). Sürükleme sırasında yuvalar `mouseDragged` ile
  toplanır ve kalan sayı yeniden hesaplanır.
- Tuşlar: 1–9 ve F imleç boşken üzerindeki yuvayla takas, Q (Ctrl ile
  hepsi) atma.
- Oyuncu modeli kutusunda fareye bakar (`extractEntityInInventoryFollowsMouse`:
  gövde `180 + 20·atan((merkez − fare)/40)`, kafa ve eğim aynı açılardan,
  `ENTITY_IN_UI` ışıkları, kutu içinde 30 birim/blok); deri varsayılan deri.
  Zırh, elde tutulan eşya ve pelerin çizilmez.
- Çizim: `extractTransparentBackground` degradesi, `container/inventory`
  dokusu, tarif kitabı düğmesi (üzerine gelince vurgulu), "Crafting" başlığı
  (`en_us.json`), vurgu arka ve ön sprite'ları, boş zırh ve sol el ikonları,
  eşyalar ve sayıları, sürükleme önizlemesi (sayı ve beyaz örtü, sınırda sarı
  sayı), fare altında imleçteki yığın.
- Oturum (`PlaySession::container_click`, `close_inventory`) menüyü dünyanın
  kopyasından yükler, tıklamayı tahmin eder, dünyaya geri yazar, önce kol
  sallama sonra tıklama paketini gönderir.

## Canlı sonuç

Vanilla 26.2 sunucusunda betikli (`VOXEL_SCRIPT` içinde `inventory` ve
`click/YUVA/DÜĞME/TÜR`) tıklamalarla taş hotbar'dan envantere taşındı,
miğfer shift ile zırh yuvasına geçti, toprak sağ tıkla ikiye bölündü.
Sunucu düzeltme göndermedi ve yeni oturumda aynı dizilim geldi. Ekran
görüntüsünde düzen referanstaki yerlerde.

## Açıklar

- Eşya ipuçları [ayrı kayıtta](tooltip-record.md).
- Tarif kitabı açılmıyor, düğmesi yalnız tıklamayı yutar; etkiler
  paneli yok.
- Referans sürüklenen yuvaları kimlik hash'li bir kümede tutar, gönderim
  sırası çalıştırmadan çalıştırmaya değişir; burada ekleme sırası. Sunucunun
  dağıtımı sıradan bağımsızdır.
- Menü arka planı bulanıklığı yok (oyun içi ekranda referans da
  bulanıklaştırmaz). Ana kol hep sağ, fare tuşlarına atanmış hotbar tuşu yok.
- Yaratıcı mod envanteri ayrı dilim (M3.c). Açık bir sandık gibi başka
  kapların ekranları yok.
