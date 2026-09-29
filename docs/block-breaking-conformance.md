# P12.c — Blok kırma döngüsü ve tahmin — client-parity kaydı

**Tarih:** 2026-09-29 · **Referans:** Java Edition 26.2 / protokol 776 / Java 25
**Kanıt sınıfı:** `client-parity`

## Ne pinlendi

Saldırı tuşunun blok kırma yolu, istemcinin kendi durumu üzerinden, tick tick:

- `Minecraft.startAttack` ve `continueAttack`: tıklamaların harcanması,
  ıskalama beklemesi (`missTime`, creative'de yok), tuş bırakılınca
  beklemenin sıfırlanması ve kırmanın bırakılması, kolun sallanması.
- `MultiPlayerGameMode` kırma durumu: `startDestroyBlock`,
  `continueDestroyBlock`, `stopDestroyBlock`, `destroyBlock`; ilerleme,
  kırma tick'i, beş tick'lik gecikme, aynı hedef kontrolü (konum ve aynı
  eşya/bileşenler), hedef değişince eski bloğun iptali, creative'de anında
  kırma, creative'de kıramayan aletler (kılıç), adventure ve spectator
  kısıtı, dünya sınırı.
- `BlockBehaviour.getDestroyProgress` ve `Player.getDestroySpeed`: aletin
  kuralları (etiket ya da blok listesi, hız, doğru alet), madencilik
  verimliliği, haste/conduit power, mining fatigue, `block_break_speed`,
  su altında `submerged_mining_speed`, havadayken beşte bir.
- Tahmin: `BlockStatePredictionHandler` sıra numarası, tahmin edilen
  bloğun sunucu durumunu saklaması, tahmin sürerken gelen blok
  güncellemesinin yalnız saklanan durumu değiştirmesi, onay (`ack`)
  gelince saklanan durumun geri konması; kırılan bloğun yerine tuttuğu
  sıvının bloğu (waterlogged → su).
- `ClientLevel.destroyBlockProgress`: oyuncunun çatlak aşaması.
- `Player.tick`: eldeki eşya değişince saldırı gücü sayacının sıfırlanması.

Oracle, [movement fixture'ındaki](movement-fixture-record.md) gerçek
`LocalPlayer`, `ClientLevel` ve `MultiPlayerGameMode` üzerinde
`Minecraft.startAttack` ve `continueAttack`'i yansımayla çağırıyor. Sahne
hedefi veriyor, hedef seçilmiyor (ışın testi ayrı kayıtta). Parçacıklar ve
blok çizimi headless'ta olmadığından boş sprite, parçacık atan motor ve
yeniden kurulum istemeyen bir `LevelExtractor` konuyor; bunlar kırma
kararlarının okuduğu hiçbir şeyi değiştirmiyor. Varsayılan item
bileşenleri, sunucunun kaynak yüklemesindeki gibi
`DATA_COMPONENT_INITIALIZERS` ile bağlanıyor.

Rust: `src/game_mode.rs`, `src/world/prediction.rs`, `src/item.rs`.
Adaptör vanilla blok etiketlerini yerel jar'daki veri paketinden çözüp
sunucunun göndereceği gibi dünyaya veriyor; istemci etiketleri sunucudan
alıyor.

## Tablolar

İki tablo referanstan yakalanıp üretiliyor ve her parity koşusunda baştan
sona karşılaştırılıyor:

- `src/world/block_destroy_table.rs` (`tools/generate/block_destroy.py`,
  `block/destroy-properties`): durum başına kırma süresi, doğru alet
  gereksinimi, hava olması, yerine kalan sıvı bloğu; game master blokları.
- `src/item_table.rs` (`tools/generate/item_tools.py`, `block/item-tools`):
  registry sırasıyla bütün itemler ve varsayılan `tool` bileşeni.

## Senaryolar

`tools/behavior/cases.py` içindeki `movement/breaking`. Her tick, hareket
çıktısına ek olarak: kırma durumu, ilerleme ve kırma tick'i (f32 bitleri),
gecikme, ıskalama beklemesi, çatlak aşaması, tahmin sıra numarası, saldırı
gücü sayacı, izlenen blokların durumu ve gönderilen `player_action` ile
`swing` paketlerinin baytları.

Dallar: çıplak elle toprak (ardından sunucu onayı), doğru aletle taş, yanlış
aletle taş, bırakma, hedef değiştirme, sunucunun kırmayı reddetmesi (onayla
geri gelme), haste, mining fatigue, zıplarken, su altında, waterlogged
yarım blok, ıskalayıp basılı tutma, creative'de basılı tutma, creative'de
kılıç, adventure.

## Sonuç

`compare --family block --family movement`: `parity_passed`.

## Canlı oturum

`play` modu sol tık ve basılı tutmayı her tick `GameMode`'a veriyor
(`PlaySession::attack`, `Minecraft.handleKeybinds` gibi oyuncunun tick'inden
önce). Hedef `GameRenderer.pick` gibi göz konumundan erişim içinde (4,5;
creative'de 5 blok) blok çerçevesine ışın testiyle seçiliyor. Oturum
envanteri, efektleri ve attribute'ları sunucudan izliyor; blok
güncellemeleri tahmin sürerken yalnız saklanan durumu değiştiriyor, onay
paketi saklananı geri koyup gerekirse oyuncuyu geri çekiyor, diğer
oyuncuların çatlakları (`block_destruction`) dünyaya yazılıyor.

Hedef blok çerçevesi her karede kısmi tick'teki göz konumundan seçilip
`forAllEdges` kenarlarıyla çiziliyor: %40 siyah, `max(2,5, genişlik/1920·2,5)`
piksel genişliğinde ekran hizalı şeritler, referansın çizgi indeks
sırasıyla, görünüm (1−1/4096)·(1−1/256) oranında kameraya çekilmiş, arazi
sisiyle.

Ölçülen: vanilla 26.2 sunucusunda betikli koşu (`VOXEL_SCRIPT` içinde
`attack` tıklama, `mine` basılı tutma) aşağı bakarak dokuz blok kazdı;
sunucu kazılan sütunu hava olarak doğruladı, fazladan teleport ya da
düzeltme göndermedi. Çerçeve referans karelerle karşılaştırılmadı.

## Eşitlik olmayanlar ve açıklar

- Kırma parçacıkları ve vuruş sesleri çalışmıyor (M7, M8);
  `BlockState.attack` ve `playerWillDestroy` yan etkileri (kızıltaş
  cevherinin yanması, kapı/yatak/çift bitkinin diğer yarısı) ve tahmin
  edilen kırmadan sonra komşu şekil güncellemeleri tahmin edilmiyor;
  sunucunun blok güncellemesi bunları düzeltiyor.
- İstemci ışığı yeniden hesaplamıyor; ışık sunucunun paketiyle geliyor.
- Eşyanın yamasındaki `tool` ve `minimum_attack_charge` bileşenleri
  okunmuyor (varsayılan kullanılıyor); adventure `can_break` yüklemi
  değerlendirilmiyor; operatör izin seviyesi izlenmediğinden game master
  blokları hiç kırılmıyor; mızrak (`piercing_weapon`) ve entity hedefi yok
  (M4).
- Çatlakların 400 tick sonra silinmesi yok.
- Eşya kullanımı (`isUsingItem`) henüz olmadığı için saldırıyı engelleyen
  kullanım dalı çalışmıyor (M1.d).
- Çatlak dokusu çizilmiyor (M1.c devamı). Çerçeve yarı saydam bloklarda
  yarı saydam yüzlerden önce çiziliyor; adventure ve spectator'da hiç
  çizilmiyor (eşyanın `can_break`/`can_place_on` yüklemleri ve menü
  sağlayan bloklar taşınmadı); dünya sınırı kontrolü ve yüksek kontrast
  seçeneği yok.
- Tanınmayan bir item bileşeni içeren envanter paketi oturumu düşürmüyor ama
  o yuva bilinmiyor.
