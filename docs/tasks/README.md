# Görev panosu ve oturum protokolü

Her görev **tek oturum, tek commit**tir ve sıfır bağlamla başlanabilecek kadar kendi kendine yeterlidir. Oturum bitince bağlam sıfırlanabilir; sürekliliği bu pano ve görev dosyalarındaki devir notları taşır. Uzun vadeli kapsam kaydı [kalan işlerde](../remaining-work.md), bölüm/kabul kuralı [roadmap](../roadmap.md)tedir; ikisi de yalnız görev isterse okunur.

## Oturum protokolü

1. Aşağıdaki tabloda bağımlılıkları `bitti` olan ilk `açık` görevi seç (kullanıcı kimlik verdiyse onu).
2. Yalnız o görev dosyasını ve onun **Oku** listesini oku. Başka belge, alt ajan veya plan modu gerekmez; arama `grep -n`/`find` ile doğrudan yapılır.
3. Java sınıfını satır satır oku (`.data/minecraft/source/…`), oracle senaryosunu yaz, Rust'ı yaz. İterasyonda: `cargo build -p <crate> 2>&1 | grep -E '^(error|warning)' -A6 | head -60`, `cargo test -p <crate> <filtre>`, `python3 tools/voxel.py parity --family gui`.
4. Commit öncesi **bir kez** `python3 tools/voxel.py check` (oracle kaynağı veya `cases.py` değiştiyse `--capture`). `CHECK OK` görmeden commit yok; başarısızsa yalnız basılan hatayı düzelt.
5. Belge: görev dosyasının **Belgeler** satırı neyi güncelleyeceğini söyler; gui operation'ı eklediysen `docs/gui-oracle.md` tablosuna satır ekle. `python3 tools/voxel.py check` belge indeksini ve hücre sınırını da denetler.
6. Görev dosyasının sonuna **Devir notu** yaz (≤10 satır: yapılan, Java'dan sapma, açık kalan), bu tabloda durumu `bitti (<kısa hash>)` yap, tek commit at (Türkçe mesaj, **AI imzası/Co-Authored-By yok**, kod yorumu İngilizce). Push/merge yok; sormadan yapılmaz.
7. Oturuma sığmıyorsa: kapıdan geçen kısmı commit et, kalanı için yeni görev satırı + dosyası aç, bu görevi `bölündü` yap.

Bağlamı korumak için: büyük dosyaları bütün okuma (`AGENTS.md` "Oturum ve token düzeni"). Canlı oyun yalnız görevin **Kabul**ü isterse, `python3 tools/voxel.py shot` ile.

## GUI oracle tarifi (G)

GUI görevleri bu beş adımı izler; görev dosyaları "G" diye anar.

1. Java: `tests/behavior/java/GuiOracle.java` — operation yöntemi ekle (örnek `pauseScreen`, `grep -n "pauseScreen"`), `switch` koluna bağla (~485–496). Gerçek Java ekranı Unsafe/yansıma sahtelerle kurulur (`screen(...)`, `set/get` yardımcıları üstte).
2. `tools/behavior/cases.py` — `gui_<ad>_cases()` üreticisi (örnek `gui_pause_screen_cases`); `generators["gui"]` zincirine ekle (`grep -n 'generators\["gui"\]'`). T02'den sonra yer `cases_gui.py`'dir.
3. Rust: `crates/voxel/examples/behavior_adapter/gui.rs` içindeki `gui_result` match kolu (örnek `"pause_screen"`). Rust ekranı üretimdeki tipin kendisidir; adaptör yalnız kurar ve `draw` çağırır.
4. `docs/gui-oracle.md` kapsam tablosuna operation satırı.
5. `python3 tools/voxel.py parity --capture --family gui` yeşil olana dek yinele. Fark çıkarsa Rust'ı düzelt; beklenen değeri Rust çıktısından türetme.

Rust ekran kalıbı: `voxel-render/src/gui/{pause,death,disconnected}.rs` (ızgara: `layout.rs`, düğme/simge: `widgets.rs`, sprite: `atlas.rs`); oyuna bağlama `crates/voxel/src/{controls,gui,play}.rs` (`open_pause`, `receive_death` örnek).

## Görevler

Boyut: S ≈ yarım oturum, M ≈ bir oturum, L ≈ zor oturum (sığmazsa böl).

| Kimlik | Görev | Boy | Bağımlılık | Durum |
| --- | --- | --- | --- | --- |
| [T01](T01-adapter-gui-modulu.md) | `behavior_adapter` GUI modülüne böl | S | – | bitti |
| [T02](T02-cases-gui-modulu.md) | `cases.py` GUI üreticilerini ayır | S | T01 | bitti |
| [T03](T03-ekran-yigini-confirm.md) | Ekran yığını + `ConfirmScreen` | M | – | bitti |
| [T04](T04-options-durumu.md) | Seçenek değerleri ve `options.txt` | M | – | bitti |
| [T05](T05-slider-cycle-widget.md) | Kaydırıcı ve döngü düğmesi widget'ları | M | T04 | bitti |
| [T06](T06-options-ekrani.md) | `OptionsScreen` | M | T03, T05 | bitti |
| [T07](T07-video-ayarlari.md) | `OptionsList` + `VideoSettingsScreen` (FOV, GUI ölçeği) | M | T06 | bitti |
| [T08](T08-sohbet-ayarlari.md) | `ChatOptionsScreen` + sohbet ayarları çizime | M | T06 | bitti |
| [T09](T09-fare-ayarlari.md) | `MouseSettingsScreen` + duyarlılık | S | T06 | bitti |
| [T10](T10-dil-secimi.md) | `LanguageSelectScreen` + dil yeniden yükleme | L | T06 | bitti |
| [T10b](T10b-sohbet-dil-yenileme.md) | Dil değişince sohbet geçmişini yeniden çöz | S | T10 | bitti |
| [T11](T11-ipucu-ok-odagi.md) | Widget ipuçları + ok tuşlarıyla odak | M | T05 | bitti |
| [T11b](T11b-liste-klavye-odagi.md) | Ayar listelerinde klavye odağı (Tab, ok) | M | T11 | bitti |
| [T12](T12-click-hover-modeli.md) | `ClickEvent`/`HoverEvent` stil modeli + sohbet ipucu | M | – | bitti |
| [T13](T13-tiklama-eylemleri.md) | Sohbet tıklama eylemleri + `ConfirmLinkScreen` | M | T03, T12 | bitti |
| [T14](T14-locator-bar.md) | `TrackedWaypoint` + `LocatorBar` | M | – | bitti |
| [T15](T15-sohbet-kalanlari.md) | Sohbet taslağı, seçici izin düzeyi | S | – | bitti |
| [T20](T20-entity-paketleri.md) | Entity paketleri canlı oturumda | L | – | bitti |
| [T21](T21-entity-tick.md) | Entity tick ve interpolasyon canlı döngüde | M | T20 | bitti |
| [T21b](T21b-mob-interpolasyonu.md) | Mob'ların (`LivingEntity`) konum/dönüş interpolasyonu | M | T21 | bitti |
| [T22](T22-uzak-oyuncu.md) | Uzaktaki oyuncu çizimi | L | T21 | bitti |
| [T22b](T22b-ad-etiketi.md) | Uzaktaki oyuncunun ad etiketi | M | T22 | bitti (402a57a1) |
| [T23](T23-model-katmani-tablosu.md) | Entity model katmanı tablosu (oracle'dan) | L | – | bitti |
| [T24](T24-entity-cizim-iskeleti.md) | Entity çizim iskeleti + ilk mob | L | T22, T23 | bitti |
| [T24b](T24b-mob-cesitleri.md) | Pig/cow çeşitleri ve entity metadata okuyucusu | M | T24 | bitti (8f0774ef) |
| [T25](T25-yerdeki-esya.md) | Yerdeki eşya | M | T24 | bitti |
| [T25b](T25b-xp-kuresi.md) | XP küresi | S | T25 | bitti (0a0fd994) |
| [T25c](T25c-esya-fizigi.md) | Yerdeki eşyanın istemci fiziği (düşme, sekme) | M | T25 | bitti |
| [T25d](T25d-esya-akiskan.md) | Yerdeki eşya akışkanda (kaldırma, akıntı) | M | T25c | bitti (ae397885) |
| [T26](T26-entity-hedefleme.md) | Entity hedefleme, `Attack`/`Interact` | M | T21 | bitti (e9dae9f8) |
| [T26b](T26b-secici-ve-pick-esya.md) | Bakılan varlık UUID önerisi + `PickItemFromEntity` | S | T26 | bitti (932f7c82) |
| [T27](T27-hasar-olum-geri-bildirimi.md) | Mob hasar kırmızısı ve ölüm yatışı (`EntityEvent`/`HurtAnimation`/`DamageEvent`, `hurtTime`/`deathTime`); kaçış süresi gözlemi | M | T26, T24 | bitti (1985d1ae) |
| [T28](T28-insansi-zombi.md) | Zombi ailesi (`HumanoidModel` + zombi kolları) | L | T24, T27 | bitti (33ec5128) |
| [T29](T29-iskelet-piglin.md) | İskelet ve piglin aileleri (yay/arbalet/kılıç pozları) | M | T28 | bitti (05d7dc65) |
| [T30](T30-ciftlik-hayvanlari-2.md) | Çiftlik hayvanları 2: koyun, mantar ineği, tavuk, keçi, tavşan | M | T24 | bölündü (b56f4b9c) |
| [T30b](T30b-keci-tavsan-mantar-inegi.md) | Keçi ve mantar ineği | M | T30, T34d | bitti (9bb2c314) |
| [T30c](T30c-tavsan.md) | Tavşan (zıplama ve baş eğme anahtar kareleri) | M | T43b | bitti (ed3f293b) |
| [T31](T31-at-deve-ailesi.md) | At, eşek, katır ve ölü atlar | L | T24 | bölündü (73f480bb) |
| [T31b](T31b-deve-lama.md) | Deve, deve kabuğu, lama ve tüccar lamasi | M | T31, T43b | bitti (a7268138) |
| [T32](T32-evcil-yirtici-dortayak.md) | Evcil ve yırtıcı dört ayaklılar: kurt, kedi, vaşak, tilki, panda, kutup ayısı, hoglin, ravager, strider | L | T24 | bölündü (kurt: 500f245b) |
| [T32b](T32b-kedi-vasak.md) | Kedi ve ocelot | M | T32 | bitti (a39fc02d) |
| [T32c](T32c-tilki-panda-kutup-ayisi.md) | Tilki, panda, kutup ayısı | L | T32 | bölündü (kutup ayısı: c350f431) |
| [T32e](T32e-tilki.md) | Tilki | M | T32c | bitti (49f6fcc8) |
| [T32f](T32f-panda.md) | Panda | M | T32c | bitti (3725df2a) |
| [T32d](T32d-hoglin-ravager-strider.md) | Hoglin, zoglin, ravager, strider | M | T32 | bölündü (hoglin: 1230fe92) |
| [T32g](T32g-ravager.md) | Ravager | M | T32d | bitti (819fac6f) |
| [T32h](T32h-strider.md) | Strider | M | T32d | bitti (dfda3dd0) |
| [T32i](T32i-titreme.md) | Titreme: donmuş ve zombileşen mob (`isShaking`) | S | T32d | bitti (271dc25e) |
| [T33](T33-eklembacaklilar-creeper.md) | Örümcek, böcek, arı ve creeper | M | T24 | bölündü (584d5eca) |
| [T33b](T33b-goz-sarj-bocek-ari.md) | Gümüş balığı, endermite, arı | M | T33 | bitti (3fb0a4f9) |
| [T33c](T33c-emissive-goz-katmani.md) | Emissive göz katmanı (örümcek, enderman) | S | T33, T34c | bitti (949699a3) |
| [T33d](T33d-enerji-girdabi.md) | Enerji girdabı ve şarjlı creeper | M | T33c | bitti (39a92d59) |
| [T34](T34-koylu-illager-golem.md) | Köylü, tüccar, illager, cadı, golemler ve enderman | L | T24 | bölündü (bc0ab848) |
| [T34b](T34b-illager-cadi-golem-enderman.md) | Illager ve cadı | M | T34 | bitti (48e17e52) |
| [T34c](T34c-golem-enderman-zombi-koylu.md) | Demir ve kar golemi, enderman, zombi köylü | L | T34b | bitti (f2c973a6) |
| [T34d](T34d-varlik-ustunde-blok.md) | Varlık üstünde blok durumu çizimi (balkabağı, çiçek, taşınan blok) | M | T34c | bitti (02ea54b4) |
| [T34e](T34e-bakir-golem.md) | Bakır golem (model, durum, doku, gözler) | M | T43b | bitti (21a9bb38) |
| [T34f](T34f-bakir-golem-katmanlari.md) | Bakır golemin eli, anteni ve başı (`ItemInHandLayer`, `BlockDecorationLayer`, `CustomHeadLayer`) | M | T34e, T41c, T34d | bitti (907dea8a) |
| [T35](T35-su-canlilari.md) | Su canlıları: kalamar, balıklar, yunus, aksolotl, kaplumbağa, kurbağa, nautilus, gardiyan | L | T24, T43b | bölündü (0369ff69) |
| [T35b](T35b-kalamar.md) | Kalamar ve ışıklı kalamar (mob suda olma durumu, hareket azalması) | M | T35 | bitti (0369ff69) |
| [T35c](T35c-baliklar.md) | Morina, somon, tropikal balık, kirpi balığı | M | T35b | bitti (5435e16a) |
| [T35d](T35d-yunus-aksolotl-kaplumbaga-kurbaga.md) | Yunus, aksolotl, kaplumbağa, iribaş, kurbağa | L | T35b, T43b | bölündü (3df6261d) |
| [T35f](T35f-aksolotl.md) | Aksolotl | M | T35d, T43b | bitti (0f5cb208) |
| [T35g](T35g-kurbaga-iribas.md) | Kurbağa ve iribaş | M | T35d, T43b | bitti (3ea8c016) |
| [T35e](T35e-nautilus-gardiyan.md) | Nautilus ve zombi nautilus | M | T35b | bitti (f2f5ca0c) |
| [T35h](T35h-gardiyan.md) | Gardiyan ve elder gardiyan (model, diken/kuyruk durumu, göz) | M | T35b | bitti (ed8fa43e) |
| [T35i](T35i-gardiyan-isini.md) | Gardiyanın ışını | M | T35h | bitti (f82729ab) |
| [T35j](T35j-gardiyan-hedefe-bakis.md) | Gardiyanın hedefe bakışı ve karadaki çırpınışı | M | T35i | bitti (ef913340) |
| [T36](T36-ucanlar.md) | Blaze, ghast ve phantom | M | T24 | bitti (7ba07997) |
| [T36b](T36b-yarasa-vex-allay.md) | Yarasa, vex ve allay | M | T36 | bitti (23ecb288) |
| [T36d](T36d-vex-allay-esya.md) | Vex ve allayın elindeki eşya | S | T36b, T41 | bitti (1cbe81f6) |
| [T36c](T36c-papagan-breeze-mutlu-ghast.md) | Papağan | M | T36 | bitti (a0e8b7c1) |
| [T36e](T36e-breeze.md) | Breeze (model, rüzgâr ve göz katmanı, animasyon durumları) | M | T36 | bitti (57aafdac) |
| [T36f](T36f-mutlu-ghast.md) | Mutlu ghast | M | T36, T41 | bitti (360dc83a) |
| [T37](T37-boss-ve-ozel.md) | Boss ve özel mobların çizimi: wither, ender ejderhası, warden, shulker, creaking, balçıklar, armadillo, sniffer | L | T43b, T36 | bölündü (fcd28dc2) |
| [T37a](T37a-slime-magma-kupu.md) | Slime ve magma cube (squish, boyut, dış katman) | M | T37 | bitti (fcd28dc2) |
| [T37b](T37b-sulfur-kupu.md) | Sulfur cube | M | T37a | bitti (7b7468cf) |
| [T37b2](T37b2-sulfur-tasidigi-blok.md) | Sulfur cube'un taşıdığı blok | M | T37b, T34d | bitti (181fe13b) |
| [T37b3](T37b3-blok-kaplamasi.md) | Varlık üstündeki bloğun kaplaması (hasar kırmızısı, fitil beyazı) | M | T37b2 | bitti (73e7be46) |
| [T37c](T37c-shulker.md) | Shulker | M | T37 | bitti (0e5a3ffc) |
| [T37c2](T37c2-shulker-konum.md) | Shulker konumu, ışınlanma kayması ve vuruş kutusu | M | T37c | bitti (97432bae) |
| [T37d](T37d-warden.md) | Warden | L | T37, T43b | bitti (2c08a2a3) |
| [T37e](T37e-creaking.md) | Creaking | M | T37, T43b | bitti (fc4791cb) |
| [T37f](T37f-armadillo-sniffer.md) | Armadillo ve sniffer | L | T37, T43b | bölündü (3eeab92a) |
| [T37f2](T37f2-sniffer.md) | Sniffer ve yavrusu | M | T37f | bitti (2bb2d981) |
| [T37g](T37g-wither.md) | Wither | M | T37, T33d | bitti (573cb415) |
| [T37h](T37h-ender-ejderhasi.md) | Ender ejderhası | L | T37 | bölündü (84b757f8) |
| [T37h2](T37h2-ejderha-olumu.md) | Ender ejderhasının ölüm çizimi (çözünme, ışınlar) | M | T37h | bitti (1ae37eaf) |
| [T37h3](T37h3-ejderha-kristal-isini.md) | Ender kristali ve ejderhanın kristal ışını | M | T37h | bitti (87198285) |
| [T37h4](T37h4-ejderha-parcalari.md) | Ender ejderhasının parçaları (vuruş kutuları, seçilebilirlik) | M | T37h | bitti (537d491f) |
| [T38](T38-araclar.md) | Tekneler (tekne, sandıklı tekne, bambu salı) | M | T24 | bitti (21875028) |
| [T38b](T38b-minecart.md) | Minecart ailesi | M | T38 | bölündü (2a1f94cc) |
| [T38c](T38c-minecart-yeni-davranis.md) | Minecart'ın yeni (deneysel) davranışı | M | T38b | bitti (2fc6b325) |
| [T38d](T38d-minecart-tnt-spawner.md) | TNT ve spawner minecart | M | T38b | bitti (cf92963b) |
| [T39](T39-mermiler.md) | Mermiler ve fırlatılanlar | M | T24 | bölündü (690ac67a) |
| [T39b](T39b-trident.md) | Trident | M | T39 | bitti (c78306ac) |
| [T39c](T39c-firlatilan-esya.md) | Fırlatılan eşyalar ve havai fişek (ateş topları dahil) | M | T39 | bitti (9148b3ec) |
| [T39d](T39d-atesler-ruzgar-yukleri.md) | Ejderha topu ve rüzgâr yükleri | M | T39 | bitti (0fc44108) |
| [T39e](T39e-shulker-mermisi-tukuruk-disler.md) | Shulker mermisi, lama tükürüğü, evoker dişleri | M | T39 | bitti (f4942705) |
| [T39g](T39g-mermi-hareketi.md) | Zarar veren mermilerin istemci hareketi (shulker mermisi, tükürük, ejderha topu, rüzgâr yükleri) | M | T39d, T39e | bitti (6c907ae4) |
| [T39f](T39f-olta-samandirasi.md) | Olta şamandırası | M | T39 | bitti (c017b9ef) |
| [T40](T40-dekor-nesneler.md) | Dekor ve nesneler: zırh askısı, çerçeve, tablo, bağ düğümü, kristal, TNT, düşen blok, yıldırım, display'ler | L | T24 | bölündü (bde91dfc) |
| [T40b](T40b-cerceve-tablo-dekor.md) | Çerçeve, tablo, bağ düğümü, kristal, TNT, düşen blok, yıldırım, display'ler | L | T40 | bölündü (T40c–T40i) |
| [T40c](T40c-tnt.md) | Yanan TNT | M | T40, T38d | bitti (4541b0f3) |
| [T40d](T40d-dusen-blok.md) | Düşen blok (moving block mesh'i) | M | T40 | bitti (63f1928e) |
| [T40e](T40e-yildirim.md) | Yıldırım | S | T40 | bitti (1749897d) |
| [T40j](T40j-gokyuzu-cakmasi.md) | Yıldırımın gökyüzü çakması (`skyFlashTime`) | S | T40e | bitti (10fca2a4) |
| [T40f](T40f-bag-dugumu.md) | Bağ düğümü (yalnız düğüm; çizgi T40k) | S | T40 | bitti (de557daf) |
| [T40k](T40k-bag-cizgisi.md) | Bağ çizgisi (`SetEntityLink`, `LeashFeatureRenderer`) | M | T40f | bitti (fdcd2c0e) |
| [T40g](T40g-esya-cercevesi.md) | Eşya çerçevesi | L | T40 | bitti (43fb7744) |
| [T40g2](T40g2-cerceve-haritasi.md) | Çerçevedeki haritanın içeriği (`MapItemData`, `MapRenderer`) | L | T40g | bitti (c92139dc) |
| [T40h](T40h-tablo.md) | Tablo | M | T40 | bitti (f75db4a8) |
| [T40i](T40i-display.md) | Block ve item display'leri | L | T40 | bitti (302fc404) |
| [T40i2](T40i2-metin-display.md) | Metin display'i | L | T40i | bitti (dbf5c822) |
| [T40i3](T40i3-display-golge-ayiklama.md) | Display gölgesi, ayıklama ve parıltı | M | T40i, T42 | bitti (9313bd89) |
| [T41](T41-ekipman-katmanlari.md) | Ekipman katmanları: `SetEquipment`, elde eşya, zırh, kafa eşyası, pelerin/elytra | L | T28, T22 | bölündü (d20fb045) |
| [T41b](T41b-zirh-katmani.md) | Zırh katmanı (`HumanoidArmorLayer`, boya, süs, parıltı) | L | T41 | bitti (c2bf0545) |
| [T41e](T41e-zirh-susu-parilti.md) | Zırh süsü, büyü parıltısı ve bileşen yaması | M | T41b | bölündü (adbcff46) |
| [T41e2](T41e2-zirh-parilti-equippable.md) | Zırh büyü parıltısı | M | T41e | bitti (a4d74bdd) |
| [T41e3](T41e3-equippable-yamasi.md) | Eşya yamasındaki `equippable` bileşeni | S | T41e2 | bitti (798f51d0) |
| [T41c](T41c-kafa-pelerin-elytra.md) | Kafa eşyası, pelerin, elytra, dürbün | M | T41 | bölündü (kafa eşyası: 4c7e5a5a) |
| [T41f](T41f-elytra-kafatasi.md) | Elytra, kafatası modelleri, gözde dürbün | M | T41c | bölündü (ce6f64c8) |
| [T41f2](T41f2-kafatasi-yerel-elytra.md) | Yerel oyuncunun ve mobların elytrası | S | T41f | bitti (1fc20242) |
| [T41f3](T41f3-kafatasi-durbun-suzulme.md) | Oyuncunun gövde dönüşü: süzülme eğimi ve riptide | S | T41f2 | bitti (aefb3f3a) |
| [T41f4](T41f4-kafatasi-durbun.md) | Kafatası modelleri ve gözde dürbün | M | T41f3 | bitti (b1227345) |
| [T41g](T41g-koylu-cadi-esya.md) | Köylü, tüccar ve cadının tuttuğu ve taktığı eşya | S | T34b, T41c | bitti (2b8c193d) |
| [T41d](T41d-kullanim-pozlari.md) | Eşya kullanım pozları ve silah kol pozları | M | T41 | bölündü (acbea4de) |
| [T41d2](T41d2-mizrak-piglin-yerel-kullanim.md) | Piglinin kol pozları | S | T41d | bitti (9c0b8c20) |
| [T41d3](T41d3-mizrak-pozlari.md) | Mızrak pozları | M | T41d | bitti (a12c5ded) |
| [T41d4](T41d4-yerel-kullanim.md) | Yerel oyuncunun eşya kullanımı ve sallama süresi | M | T41d3 | bitti (b0f60ebc) |
| [T42](T42-entity-golgesi.md) | Entity gölgesi (`submitShadow`) | M | T24 | bitti (d2c2f477) |
| [T43](T43-keyframe-animasyon-ve-animate.md) | `KeyframeAnimation` oynatıcısı, `Animate` paketi ve `EntityEvent` çizim durumları | L | T24 | bölündü (66840389) |
| [T43b](T43b-keyframe-oynatici.md) | `KeyframeAnimation` oynatıcısı ve `AnimationState` (ilk tüketici tavşan, T30c) | M | T43 | bitti (ed3f293b) |
| [T44](T44-binek-surusu.md) | Binek sürüşü: `PlayerInput`, `PlayerCommand`, `MoveVehicle`, `PaddleBoat` | L | T38, T31 | bölündü (T44a, T44b) |
| [T44a](T44a-tekne-surusu.md) | Oyuncunun tekneyi sürmesi (tekne tiki, yolcu tiki, `MoveVehicle`, `PaddleBoat`) | L | T38 | bitti (a3b30282) |
| [T44b](T44b-binek-hayvanlar-minecart.md) | Binek hayvanlar ve minecart sürüşü (at ailesi, domuz, deve, strider, minecart), binme/inme | L | T44a, T31 | bölündü (T44b1, T44c) |
| [T44b1](T44b1-eyerli-at.md) | Oyuncunun eyerli atı sürmesi (at, eşek, katır, iskelet ve zombi atı; atlama yükü) | L | T44a, T31 | bitti (7606cdb6) |
| [T44c](T44c-diger-bineklar.md) | Öteki binekler: eyerli domuz, strider, deve, minecart; binme/inme | L | T44b1 | bölündü (T44c1, T44c2) |
| [T44c1](T44c1-domuz-yolcu.md) | Eyerli domuz ve yönetilmeyen araçta yolcu | M | T44b1 | bitti (db4b6e7b) |
| [T44c2](T44c2-strider-deve-minecart.md) | Strider (lavda yürüme), deve, minecart; binme/inme | L | T44c1 | bölündü (7a02f132) |
| [T44c3](T44c3-deve-minecart.md) | Deve (koşu, oturma) sürüşü | M | T44c2 | bitti (2ba44475) |
| [T44c4](T44c4-minecart-binme-inme.md) | Minecart yolcusu ve binme/inme | M | T44c3 | bitti (37f8c4ea) |
| [T45](T45-saldiri-gucu-ve-combat.md) | Saldırı gücü göstergesi ve `PlayerCombat*` paketleri | S | T26 | bitti (fd8d309d) |
| PLAN-M4 | Kalan entity içeriğini görevlere böl (insansı/dört ayaklı aileler, zırh, ad etiketi, hasar/ölüm (T27 ayrıldı: kırmızı örtü ve ölüm yatışı), binekler, mermiler, çerçeve/tablo) | S | T24 | bitti (8ab67211) |
| PLAN-M8 | Blok entity çizicileri + `LevelEvent` + partikül sistemini görevlere böl (`TakeItemEntity` paketi + `ItemPickupParticle` eşya toplama animasyonu da burada: parçacık motoru olmadan yapılmaz) | S | T24 | açık |
| [T46](T46-mob-oznitelikleri.md) | Mob öznitelikleri (`UpdateAttributes`): can üstü, ölçek | S | T34c | bitti (99446dc2) |
| [PLAN-M3d](PLAN-M3d.md) | Kalan kap ekranları ve tarif kitabını görevlere böl | S | – | bitti (f64024d2) |
| [T50](T50-loom-cartography-smithing-oracle.md) | Loom, cartography ve smithing ekranlarının oracle'ı | M | – | bitti (12e5d8ea) |
| [T51](T51-anvil-sonuc-tahmini.md) | Örs ve taş çarkı sonucunun istemci tahmini | L | – | açık |
| [T52](T52-ipucu-bilesen-hash.md) | Bileşenli eşyada tıklama paketi hash'i ve bundle tıklaması | M | – | açık |
| [T53](T53-beacon.md) | İşaret ışığı menüsü ve ekranı (`SetBeacon`) | M | T50 | bitti (184cdc4b) |
| [T54](T54-brewing-stand.md) | Simya standı menüsü ve ekranı | M | – | bitti (20572c47) |
| [T55](T55-tuccar-menusu.md) | Tüccar menüsü ve `MerchantOffers` | L | T50 | açık |
| [T56](T56-lectern-kitap.md) | Kürsü menüsü ve kitap görünümü | M | – | açık |
| [T57](T57-crafter.md) | Crafter menüsü ve ekranı | M | – | bitti (HASH) |
| [T58](T58-binek-envanteri.md) | At, eşek, katır ve nautilus envanteri | L | T50 | açık |
| [T59](T59-kap-ipuclari.md) | Kap içeriği ipucu, `#etiket` araması ve eşya süsleri | M | – | açık |
| [PLAN-M6](PLAN-M3d.md) | Ana menü, sunucu listesi, ilerlemeler, kitap/tabela ve kalan ekranları görevlere böl | S | – | açık |
| PLAN-M7 | Ses: bağımlılık kararı ([karar](../audio-backend-decision.md): çıkış `cpal`, çözücü `lewton`) ve görevlere bölme | S | – | bitti |
| [T60](T60-sounds-json-secim.md) | `sounds.json` çözümü ve ağırlıklı ses seçimi | M | – | bitti (bcbec5b0) |
| [T61](T61-ses-paketleri-kanal.md) | `Sound`/`SoundEntity`/`StopSound` paketleri ve `SoundEngine` kanal parametreleri | M | T60 | bölündü (ba2f3f4e) |
| [T61b](T61b-ses-motoru.md) | `SoundEngine` mantığı ve kanal parametreleri (Channel gölgesiyle oracle) | L | T61 | bitti (19323b90) |
| [T62](T62-ogg-vorbis-cozumu.md) | Ogg/Vorbis çözümü (`lewton`, JOrbis ile PCM karşılaştırması) | M | – | bitti (16260689) |
| [T63](T63-ses-cikisi-cpal.md) | Ses çıkışı (`cpal`): kanal havuzu, sönüm, perde, miks | L | T61b, T62 | bitti (f4381a44) |
| [T64](T64-ses-tetikleri-muzik.md) | Oyun ses tetikleri (adım, kırma, hasar, ortam) ve müzik | L | T63 | bölündü (3659d38e) |
| [T64b](T64b-ses-tetikleri-kalan.md) | Oyuncunun hareket sesleri ve tetik oracle'ı (adım, yüzme, sıçrama, iniş, kabarcık sütunu) | L | T64 | bitti (323dfa99) |
| [T64c](T64c-ses-tetikleri-son.md) | Oyuncunun hasar ve ölüm sesi | M | T64b | bitti (a008d958) |
| [T64d](T64d-ses-tetikleri-ortam-muzik.md) | Ortam, olay sesleri ve müzik | L | T64c | açık |
| [T65](T65-ses-ayarlari-ekrani.md) | Ses ayarları ekranı | M | T63, T06 | bitti (a9e6230d) |
| [T66](T66-hud-durum-oracle.md) | HUD durum çubukları oracle'ı (kalpler, zırh, yiyecek, hava, deneyim) | M | T45 | bitti (5814a8cd) |
| PLAN-M9 | Çevrimiçi kimlik, yeniden yapılandırma, kaynak paketi görevlere böl | S | – | açık |

Görev yazarken şablon: Amaç · Bağımlılık · Oku · Yapılacaklar · Oracle · Kabul · Kapsam dışı · Belgeler · Devir notu (≤ 60 satır).
