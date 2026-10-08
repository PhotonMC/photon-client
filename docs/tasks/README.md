# Görev panosu ve oturum protokolü

Her görev **tek oturum, tek commit**tir ve sıfır bağlamla başlanabilecek kadar kendi kendine yeterlidir. Oturum bitince bağlam sıfırlanabilir; sürekliliği bu pano ve görev dosyalarındaki devir notları taşır. Uzun vadeli kapsam kaydı [kalan işlerde](../remaining-work.md), bölüm/kabul kuralı [roadmap](../roadmap.md)tedir; ikisi de yalnız görev isterse okunur.

## Oturum protokolü

1. **İlerleme** bloğundaki "sıradaki adaylar" listesinden kritik yola en yakın görevi seç (kullanıcı kimlik verdiyse onu).
2. Yalnız o görev dosyasını ve onun **Oku** listesini oku. Başka belge, alt ajan veya plan modu gerekmez; arama `grep -n`/`find` ile doğrudan yapılır.
3. Java sınıfını satır satır oku (`.data/minecraft/source/…`), oracle senaryosunu yaz, Rust'ı yaz. İterasyonda: `cargo build -p <crate> 2>&1 | grep -E '^(error|warning)' -A6 | head -60`, `cargo test -p <crate> <filtre>`, `python3 tools/photon.py parity --family gui`.
4. Commit öncesi **bir kez** `python3 tools/photon.py check` (oracle kaynağı veya `tools/behavior/cases/` değiştiyse `--capture`). `CHECK OK` görmeden commit yok; başarısızsa yalnız basılan hatayı düzelt.
5. Belge: görev dosyasının **Belgeler** satırı neyi güncelleyeceğini söyler; gui operation'ı eklediysen `docs/gui-oracle.md` tablosuna satır ekle. `python3 tools/photon.py check` belge indeksini ve hücre sınırını da denetler.
6. Görev dosyasının sonuna **Devir notu** yaz (≤10 satır: yapılan, Java'dan sapma, açık kalan), panoda kutuyu işaretle (`- [x]`, satır sonuna `bitti (<kısa hash>)`), `python3 tools/task-progress.py` ile yüzdeleri yenile, tek commit at (Türkçe mesaj, **AI imzası/Co-Authored-By yok**, kod yorumu İngilizce). Push/merge yok; sormadan yapılmaz.
7. Oturuma sığmıyorsa: kapıdan geçen kısmı commit et, kalanı için aynı grupta yeni madde + dosyası aç, bu görevi işaretleyip `bölündü (<hash>)` yaz. Bir `K-` kapsam maddesi görevlere bölününce görevler eklenir ve `K-` maddesi silinir.

Bağlamı korumak için: büyük dosyaları bütün okuma (`AGENTS.md` "Oturum ve token düzeni"). Canlı oyun yalnız görevin **Kabul**ü isterse, `python3 tools/photon.py shot` ile.

## GUI oracle tarifi (G)

GUI görevleri bu beş adımı izler; görev dosyaları "G" diye anar.

1. Java: `tests/behavior/java/GuiOracle.java` — operation yöntemi ekle (örnek `pauseScreen`, `grep -n "pauseScreen"`), `switch` koluna bağla (~485–496). Gerçek Java ekranı Unsafe/yansıma sahtelerle kurulur (`screen(...)`, `set/get` yardımcıları üstte).
2. `tools/behavior/cases/gui/` — konuya göre modülde `gui_<ad>_cases()` üreticisi (örnek `gui_pause_screen_cases`, `screens.py`); `cases/gui/__init__.py` içindeki `gui_generators` zincirine ekle.
3. Rust: `crates/photon-oracle/src/gui.rs` içindeki `gui_result` match kolu (örnek `"pause_screen"`). Rust ekranı üretimdeki tipin kendisidir; adaptör yalnız kurar ve `draw` çağırır.
4. `docs/gui-oracle.md` kapsam tablosuna operation satırı.
5. `python3 tools/photon.py parity --capture --family gui` yeşil olana dek yinele. Fark çıkarsa Rust'ı düzelt; beklenen değeri Rust çıktısından türetme.

Rust ekran kalıbı: `photon-render/src/gui/screen/{pause,death,disconnected}.rs` (ızgara: `gui/layout.rs`, düğme/simge: `gui/widgets.rs`, sprite: `gui/atlas.rs`); oyuna bağlama `crates/photon/src/{controls,gui,play}.rs` ve `controls/{pause,death,confirm}.rs` (`open_pause`, `receive_death` örnek).

## İlerleme

Yüzdeler aşağıdaki kutulardan `python3 tools/task-progress.py` ile hesaplanır; elle yazılmaz (`photon.py check` bayatsa düşer). Ağırlık S 1, M 2, L 3. `K-` maddeleri henüz görevlere bölünmemiş kapsamdır ve boyutlarındaki `×N` tahmini görev sayısıdır (`M×12` = on iki M görev); bölündükçe yerini görev maddeleri alır, bu yüzden yüzde kapsam büyüdükçe düşebilir. Yüzde ilerlemenin ölçüsüdür, Java eşitliği iddiası değildir: bölümler ancak `client-parity` ile kapanır (M10).

<!-- progress:start -->
**Toplam: %78 bitti** (ağırlıklı) · 239/288 madde işaretli, **49 madde kaldı** (tahmini 93 M-görev eşdeğeri iş).

| Grup | İlerleme | % | Madde |
| --- | --- | ---: | ---: |
| Temel (pano öncesi) | `████████████████████` | 100 | 7/7 |
| M3.d Sunucunun açtığı kaplar | `████████████░░░░░░░░` | 58 | 7/11 |
| M4 Entity'ler | `████████████████████` | 99 | 130/131 |
| M5 Metin ve sohbet | `██████████████░░░░░░` | 68 | 5/7 |
| M6 Ekranlar ve ayarlar | `█████████████░░░░░░░` | 65 | 24/35 |
| M7 Ses | `███████████████░░░░░` | 77 | 10/12 |
| M8 Partikül, efekt ve blok entity | `████████████████░░░░` | 79 | 34/42 |
| M9 Sunucu uyumluluğu | `░░░░░░░░░░░░░░░░░░░░` | 0 | 0/2 |
| M10 Eşitlik ve dağıtım | `░░░░░░░░░░░░░░░░░░░░` | 0 | 0/4 |
| Paralel borçlar | `░░░░░░░░░░░░░░░░░░░░` | 0 | 0/4 |
| Kod kalitesi ve performans | `█████████████░░░░░░░` | 63 | 22/33 |

Bağımlılıkları bitmiş açık görevler (sıradaki adaylar): T51, T52, T58, T59, T85b, T88, T89, T90, T91, T92, T93, T94b, T95, T96, T64d, T70i, T74c, T76c2, T78e, T78d, T81, T82, T83b, PLAN-M9, Q04, V05b, A01c, A03, A04, A05b, D02, D03, D04, D05, D06.
<!-- progress:end -->

## Görevler

Boyut: S ≈ yarım oturum, M ≈ bir oturum, L ≈ zor oturum (sığmazsa böl); `×N` tahmini görev sayısı. `[x]` bitti; `bölündü` kalanını alt görevlerine devretmiştir.

### Temel (pano öncesi)

Görev panosundan önce bitmiş temel; `client-parity` kapanışları M10'da.

- [x] **K-P06** Bağlantı, giriş, yapılandırma ve kayıt defterleri (P06–P07) — M×10
- [x] **K-P08** Dünya kopyası: chunk, ışık, blok entity, entity durumu (P08) — M×30
- [x] **K-P10** Çarpışma, yerel hareket ve hareket senkronu (P09–P11) — M×15
- [x] **K-P16** Dünya çizimi: bloklar, ışık, gökyüzü, bulut, hava, sis (P15–P16) — M×15
- [x] **K-M1** M1 Blok eylemleri: kırma, koyma, tahmin ve ack — M×5
- [x] **K-M2** M2 Eşya çizimi: modeller, ikonlar, eldeki eşya — M×5
- [x] **K-M3** M3 Envanter: hayatta kalma ve yaratıcı ekranları, tıklama tahmini — M×6

### M3.d Sunucunun açtığı kaplar

- [x] [PLAN-M3d](PLAN-M3d.md) Kalan kap ekranları ve tarif kitabını görevlere böl — S · bitti (f64024d2)
- [x] [T50](T50-loom-cartography-smithing-oracle.md) Loom, cartography ve smithing ekranlarının oracle'ı — M · bitti (12e5d8ea)
- [ ] [T51](T51-anvil-sonuc-tahmini.md) Örs ve taş çarkı sonucunun istemci tahmini — L
- [ ] [T52](T52-ipucu-bilesen-hash.md) Bileşenli eşyada tıklama paketi hash'i ve bundle tıklaması — M
- [x] [T53](T53-beacon.md) İşaret ışığı menüsü ve ekranı (`SetBeacon`) — M · bağımlı: T50 · bitti (184cdc4b)
- [x] [T54](T54-brewing-stand.md) Simya standı menüsü ve ekranı — M · bitti (20572c47)
- [x] [T55](T55-tuccar-menusu.md) Tüccar menüsü ve `MerchantOffers` — L · bağımlı: T50 · bitti (968caf60)
- [x] [T56](T56-lectern-kitap.md) Kürsü menüsü ve kitap görünümü — M · bitti (8519cd70)
- [x] [T57](T57-crafter.md) Crafter menüsü ve ekranı — M · bitti (3b303cb3)
- [ ] [T58](T58-binek-envanteri.md) At, eşek, katır ve nautilus envanteri — L · bağımlı: T50
- [ ] [T59](T59-kap-ipuclari.md) Kap içeriği ipucu, `#etiket` araması ve eşya süsleri — M

### M4 Entity'ler

- [x] **PLAN-M4** Kalan entity içeriğini görevlere böl (insansı/dört ayaklı aileler, zırh, ad etiketi, hasar/ölüm (T27 ayrıldı: kırmızı örtü ve ölüm yatışı), binekler, mermiler, çerçeve/tablo) — S · bağımlı: T24 · bitti (8ab67211)
- [x] [T20](T20-entity-paketleri.md) Entity paketleri canlı oturumda — L · bitti
- [x] [T21](T21-entity-tick.md) Entity tick ve interpolasyon canlı döngüde — M · bağımlı: T20 · bitti
- [x] [T21b](T21b-mob-interpolasyonu.md) Mob'ların (`LivingEntity`) konum/dönüş interpolasyonu — M · bağımlı: T21 · bitti
- [x] [T22](T22-uzak-oyuncu.md) Uzaktaki oyuncu çizimi — L · bağımlı: T21 · bitti
- [x] [T22b](T22b-ad-etiketi.md) Uzaktaki oyuncunun ad etiketi — M · bağımlı: T22 · bitti (402a57a1)
- [x] [T23](T23-model-katmani-tablosu.md) Entity model katmanı tablosu (oracle'dan) — L · bitti
- [x] [T24](T24-entity-cizim-iskeleti.md) Entity çizim iskeleti + ilk mob — L · bağımlı: T22, T23 · bitti
- [x] [T24b](T24b-mob-cesitleri.md) Pig/cow çeşitleri ve entity metadata okuyucusu — M · bağımlı: T24 · bitti (8f0774ef)
- [x] [T25](T25-yerdeki-esya.md) Yerdeki eşya — M · bağımlı: T24 · bitti
- [x] [T25b](T25b-xp-kuresi.md) XP küresi — S · bağımlı: T25 · bitti (0a0fd994)
- [x] [T25c](T25c-esya-fizigi.md) Yerdeki eşyanın istemci fiziği (düşme, sekme) — M · bağımlı: T25 · bitti
- [x] [T25d](T25d-esya-akiskan.md) Yerdeki eşya akışkanda (kaldırma, akıntı) — M · bağımlı: T25c · bitti (ae397885)
- [x] [T26](T26-entity-hedefleme.md) Entity hedefleme, `Attack`/`Interact` — M · bağımlı: T21 · bitti (e9dae9f8)
- [x] [T26b](T26b-secici-ve-pick-esya.md) Bakılan varlık UUID önerisi + `PickItemFromEntity` — S · bağımlı: T26 · bitti (932f7c82)
- [x] [T27](T27-hasar-olum-geri-bildirimi.md) Mob hasar kırmızısı ve ölüm yatışı (`EntityEvent`/`HurtAnimation`/`DamageEvent`, `hurtTime`/`deathTime`); kaçış süresi gözlemi — M · bağımlı: T26, T24 · bitti (1985d1ae)
- [x] [T28](T28-insansi-zombi.md) Zombi ailesi (`HumanoidModel` + zombi kolları) — L · bağımlı: T24, T27 · bitti (33ec5128)
- [x] [T29](T29-iskelet-piglin.md) İskelet ve piglin aileleri (yay/arbalet/kılıç pozları) — M · bağımlı: T28 · bitti (05d7dc65)
- [x] [T30](T30-ciftlik-hayvanlari-2.md) Çiftlik hayvanları 2: koyun, mantar ineği, tavuk, keçi, tavşan — M · bağımlı: T24 · bölündü (b56f4b9c)
- [x] [T30b](T30b-keci-tavsan-mantar-inegi.md) Keçi ve mantar ineği — M · bağımlı: T30, T34d · bitti (9bb2c314)
- [x] [T30c](T30c-tavsan.md) Tavşan (zıplama ve baş eğme anahtar kareleri) — M · bağımlı: T43b · bitti (ed3f293b)
- [x] [T31](T31-at-deve-ailesi.md) At, eşek, katır ve ölü atlar — L · bağımlı: T24 · bölündü (73f480bb)
- [x] [T31b](T31b-deve-lama.md) Deve, deve kabuğu, lama ve tüccar lamasi — M · bağımlı: T31, T43b · bitti (a7268138)
- [x] [T32](T32-evcil-yirtici-dortayak.md) Evcil ve yırtıcı dört ayaklılar: kurt, kedi, vaşak, tilki, panda, kutup ayısı, hoglin, ravager, strider — L · bağımlı: T24 · bölündü (kurt: 500f245b)
- [x] [T32b](T32b-kedi-vasak.md) Kedi ve ocelot — M · bağımlı: T32 · bitti (a39fc02d)
- [x] [T32c](T32c-tilki-panda-kutup-ayisi.md) Tilki, panda, kutup ayısı — L · bağımlı: T32 · bölündü (kutup ayısı: c350f431)
- [x] [T32e](T32e-tilki.md) Tilki — M · bağımlı: T32c · bitti (49f6fcc8)
- [x] [T32f](T32f-panda.md) Panda — M · bağımlı: T32c · bitti (3725df2a)
- [x] [T32d](T32d-hoglin-ravager-strider.md) Hoglin, zoglin, ravager, strider — M · bağımlı: T32 · bölündü (hoglin: 1230fe92)
- [x] [T32g](T32g-ravager.md) Ravager — M · bağımlı: T32d · bitti (819fac6f)
- [x] [T32h](T32h-strider.md) Strider — M · bağımlı: T32d · bitti (dfda3dd0)
- [x] [T32i](T32i-titreme.md) Titreme: donmuş ve zombileşen mob (`isShaking`) — S · bağımlı: T32d · bitti (271dc25e)
- [x] [T33](T33-eklembacaklilar-creeper.md) Örümcek, böcek, arı ve creeper — M · bağımlı: T24 · bölündü (584d5eca)
- [x] [T33b](T33b-goz-sarj-bocek-ari.md) Gümüş balığı, endermite, arı — M · bağımlı: T33 · bitti (3fb0a4f9)
- [x] [T33c](T33c-emissive-goz-katmani.md) Emissive göz katmanı (örümcek, enderman) — S · bağımlı: T33, T34c · bitti (949699a3)
- [x] [T33d](T33d-enerji-girdabi.md) Enerji girdabı ve şarjlı creeper — M · bağımlı: T33c · bitti (39a92d59)
- [x] [T34](T34-koylu-illager-golem.md) Köylü, tüccar, illager, cadı, golemler ve enderman — L · bağımlı: T24 · bölündü (bc0ab848)
- [x] [T34b](T34b-illager-cadi-golem-enderman.md) Illager ve cadı — M · bağımlı: T34 · bitti (48e17e52)
- [x] [T34c](T34c-golem-enderman-zombi-koylu.md) Demir ve kar golemi, enderman, zombi köylü — L · bağımlı: T34b · bitti (f2c973a6)
- [x] [T34d](T34d-varlik-ustunde-blok.md) Varlık üstünde blok durumu çizimi (balkabağı, çiçek, taşınan blok) — M · bağımlı: T34c · bitti (02ea54b4)
- [x] [T34e](T34e-bakir-golem.md) Bakır golem (model, durum, doku, gözler) — M · bağımlı: T43b · bitti (21a9bb38)
- [x] [T34f](T34f-bakir-golem-katmanlari.md) Bakır golemin eli, anteni ve başı (`ItemInHandLayer`, `BlockDecorationLayer`, `CustomHeadLayer`) — M · bağımlı: T34e, T41c, T34d · bitti (907dea8a)
- [x] [T35](T35-su-canlilari.md) Su canlıları: kalamar, balıklar, yunus, aksolotl, kaplumbağa, kurbağa, nautilus, gardiyan — L · bağımlı: T24, T43b · bölündü (0369ff69)
- [x] [T35b](T35b-kalamar.md) Kalamar ve ışıklı kalamar (mob suda olma durumu, hareket azalması) — M · bağımlı: T35 · bitti (0369ff69)
- [x] [T35c](T35c-baliklar.md) Morina, somon, tropikal balık, kirpi balığı — M · bağımlı: T35b · bitti (5435e16a)
- [x] [T35d](T35d-yunus-aksolotl-kaplumbaga-kurbaga.md) Yunus, aksolotl, kaplumbağa, iribaş, kurbağa — L · bağımlı: T35b, T43b · bölündü (3df6261d)
- [x] [T35f](T35f-aksolotl.md) Aksolotl — M · bağımlı: T35d, T43b · bitti (0f5cb208)
- [x] [T35g](T35g-kurbaga-iribas.md) Kurbağa ve iribaş — M · bağımlı: T35d, T43b · bitti (3ea8c016)
- [x] [T35e](T35e-nautilus-gardiyan.md) Nautilus ve zombi nautilus — M · bağımlı: T35b · bitti (f2f5ca0c)
- [x] [T35h](T35h-gardiyan.md) Gardiyan ve elder gardiyan (model, diken/kuyruk durumu, göz) — M · bağımlı: T35b · bitti (ed8fa43e)
- [x] [T35i](T35i-gardiyan-isini.md) Gardiyanın ışını — M · bağımlı: T35h · bitti (f82729ab)
- [x] [T35j](T35j-gardiyan-hedefe-bakis.md) Gardiyanın hedefe bakışı ve karadaki çırpınışı — M · bağımlı: T35i · bitti (ef913340)
- [x] [T36](T36-ucanlar.md) Blaze, ghast ve phantom — M · bağımlı: T24 · bitti (7ba07997)
- [x] [T36b](T36b-yarasa-vex-allay.md) Yarasa, vex ve allay — M · bağımlı: T36 · bitti (23ecb288)
- [x] [T36d](T36d-vex-allay-esya.md) Vex ve allayın elindeki eşya — S · bağımlı: T36b, T41 · bitti (1cbe81f6)
- [x] [T36c](T36c-papagan-breeze-mutlu-ghast.md) Papağan — M · bağımlı: T36 · bitti (a0e8b7c1)
- [x] [T36e](T36e-breeze.md) Breeze (model, rüzgâr ve göz katmanı, animasyon durumları) — M · bağımlı: T36 · bitti (57aafdac)
- [x] [T36f](T36f-mutlu-ghast.md) Mutlu ghast — M · bağımlı: T36, T41 · bitti (360dc83a)
- [x] [T37](T37-boss-ve-ozel.md) Boss ve özel mobların çizimi: wither, ender ejderhası, warden, shulker, creaking, balçıklar, armadillo, sniffer — L · bağımlı: T43b, T36 · bölündü (fcd28dc2)
- [x] [T37a](T37a-slime-magma-kupu.md) Slime ve magma cube (squish, boyut, dış katman) — M · bağımlı: T37 · bitti (fcd28dc2)
- [x] [T37b](T37b-sulfur-kupu.md) Sulfur cube — M · bağımlı: T37a · bitti (7b7468cf)
- [x] [T37b2](T37b2-sulfur-tasidigi-blok.md) Sulfur cube'un taşıdığı blok — M · bağımlı: T37b, T34d · bitti (181fe13b)
- [x] [T37b3](T37b3-blok-kaplamasi.md) Varlık üstündeki bloğun kaplaması (hasar kırmızısı, fitil beyazı) — M · bağımlı: T37b2 · bitti (73e7be46)
- [x] [T37c](T37c-shulker.md) Shulker — M · bağımlı: T37 · bitti (0e5a3ffc)
- [x] [T37c2](T37c2-shulker-konum.md) Shulker konumu, ışınlanma kayması ve vuruş kutusu — M · bağımlı: T37c · bitti (97432bae)
- [x] [T37d](T37d-warden.md) Warden — L · bağımlı: T37, T43b · bitti (2c08a2a3)
- [x] [T37e](T37e-creaking.md) Creaking — M · bağımlı: T37, T43b · bitti (fc4791cb)
- [x] [T37f](T37f-armadillo-sniffer.md) Armadillo ve sniffer — L · bağımlı: T37, T43b · bölündü (3eeab92a)
- [x] [T37f2](T37f2-sniffer.md) Sniffer ve yavrusu — M · bağımlı: T37f · bitti (2bb2d981)
- [x] [T37g](T37g-wither.md) Wither — M · bağımlı: T37, T33d · bitti (573cb415)
- [x] [T37h](T37h-ender-ejderhasi.md) Ender ejderhası — L · bağımlı: T37 · bölündü (84b757f8)
- [x] [T37h2](T37h2-ejderha-olumu.md) Ender ejderhasının ölüm çizimi (çözünme, ışınlar) — M · bağımlı: T37h · bitti (1ae37eaf)
- [x] [T37h3](T37h3-ejderha-kristal-isini.md) Ender kristali ve ejderhanın kristal ışını — M · bağımlı: T37h · bitti (87198285)
- [x] [T37h4](T37h4-ejderha-parcalari.md) Ender ejderhasının parçaları (vuruş kutuları, seçilebilirlik) — M · bağımlı: T37h · bitti (537d491f)
- [x] [T38](T38-araclar.md) Tekneler (tekne, sandıklı tekne, bambu salı) — M · bağımlı: T24 · bitti (21875028)
- [x] [T38b](T38b-minecart.md) Minecart ailesi — M · bağımlı: T38 · bölündü (2a1f94cc)
- [x] [T38c](T38c-minecart-yeni-davranis.md) Minecart'ın yeni (deneysel) davranışı — M · bağımlı: T38b · bitti (2fc6b325)
- [x] [T38d](T38d-minecart-tnt-spawner.md) TNT ve spawner minecart — M · bağımlı: T38b · bitti (cf92963b)
- [x] [T39](T39-mermiler.md) Mermiler ve fırlatılanlar — M · bağımlı: T24 · bölündü (690ac67a)
- [x] [T39b](T39b-trident.md) Trident — M · bağımlı: T39 · bitti (c78306ac)
- [x] [T39c](T39c-firlatilan-esya.md) Fırlatılan eşyalar ve havai fişek (ateş topları dahil) — M · bağımlı: T39 · bitti (9148b3ec)
- [x] [T39d](T39d-atesler-ruzgar-yukleri.md) Ejderha topu ve rüzgâr yükleri — M · bağımlı: T39 · bitti (0fc44108)
- [x] [T39e](T39e-shulker-mermisi-tukuruk-disler.md) Shulker mermisi, lama tükürüğü, evoker dişleri — M · bağımlı: T39 · bitti (f4942705)
- [x] [T39g](T39g-mermi-hareketi.md) Zarar veren mermilerin istemci hareketi (shulker mermisi, tükürük, ejderha topu, rüzgâr yükleri) — M · bağımlı: T39d, T39e · bitti (6c907ae4)
- [x] [T39f](T39f-olta-samandirasi.md) Olta şamandırası — M · bağımlı: T39 · bitti (c017b9ef)
- [x] [T40](T40-dekor-nesneler.md) Dekor ve nesneler: zırh askısı, çerçeve, tablo, bağ düğümü, kristal, TNT, düşen blok, yıldırım, display'ler — L · bağımlı: T24 · bölündü (bde91dfc)
- [x] [T40b](T40b-cerceve-tablo-dekor.md) Çerçeve, tablo, bağ düğümü, kristal, TNT, düşen blok, yıldırım, display'ler — L · bağımlı: T40 · bölündü (T40c–T40i)
- [x] [T40c](T40c-tnt.md) Yanan TNT — M · bağımlı: T40, T38d · bitti (4541b0f3)
- [x] [T40d](T40d-dusen-blok.md) Düşen blok (moving block mesh'i) — M · bağımlı: T40 · bitti (63f1928e)
- [x] [T40e](T40e-yildirim.md) Yıldırım — S · bağımlı: T40 · bitti (1749897d)
- [x] [T40j](T40j-gokyuzu-cakmasi.md) Yıldırımın gökyüzü çakması (`skyFlashTime`) — S · bağımlı: T40e · bitti (10fca2a4)
- [x] [T40f](T40f-bag-dugumu.md) Bağ düğümü (yalnız düğüm; çizgi T40k) — S · bağımlı: T40 · bitti (de557daf)
- [x] [T40k](T40k-bag-cizgisi.md) Bağ çizgisi (`SetEntityLink`, `LeashFeatureRenderer`) — M · bağımlı: T40f · bitti (fdcd2c0e)
- [x] [T40g](T40g-esya-cercevesi.md) Eşya çerçevesi — L · bağımlı: T40 · bitti (43fb7744)
- [x] [T40g2](T40g2-cerceve-haritasi.md) Çerçevedeki haritanın içeriği (`MapItemData`, `MapRenderer`) — L · bağımlı: T40g · bitti (c92139dc)
- [x] [T40h](T40h-tablo.md) Tablo — M · bağımlı: T40 · bitti (f75db4a8)
- [x] [T40i](T40i-display.md) Block ve item display'leri — L · bağımlı: T40 · bitti (302fc404)
- [x] [T40i2](T40i2-metin-display.md) Metin display'i — L · bağımlı: T40i · bitti (dbf5c822)
- [x] [T40i3](T40i3-display-golge-ayiklama.md) Display gölgesi, ayıklama ve parıltı — M · bağımlı: T40i, T42 · bitti (9313bd89)
- [x] [T41](T41-ekipman-katmanlari.md) Ekipman katmanları: `SetEquipment`, elde eşya, zırh, kafa eşyası, pelerin/elytra — L · bağımlı: T28, T22 · bölündü (d20fb045)
- [x] [T41b](T41b-zirh-katmani.md) Zırh katmanı (`HumanoidArmorLayer`, boya, süs, parıltı) — L · bağımlı: T41 · bitti (c2bf0545)
- [x] [T41e](T41e-zirh-susu-parilti.md) Zırh süsü, büyü parıltısı ve bileşen yaması — M · bağımlı: T41b · bölündü (adbcff46)
- [x] [T41e2](T41e2-zirh-parilti-equippable.md) Zırh büyü parıltısı — M · bağımlı: T41e · bitti (a4d74bdd)
- [x] [T41e3](T41e3-equippable-yamasi.md) Eşya yamasındaki `equippable` bileşeni — S · bağımlı: T41e2 · bitti (798f51d0)
- [x] [T41c](T41c-kafa-pelerin-elytra.md) Kafa eşyası, pelerin, elytra, dürbün — M · bağımlı: T41 · bölündü (kafa eşyası: 4c7e5a5a)
- [x] [T41f](T41f-elytra-kafatasi.md) Elytra, kafatası modelleri, gözde dürbün — M · bağımlı: T41c · bölündü (ce6f64c8)
- [x] [T41f2](T41f2-kafatasi-yerel-elytra.md) Yerel oyuncunun ve mobların elytrası — S · bağımlı: T41f · bitti (1fc20242)
- [x] [T41f3](T41f3-kafatasi-durbun-suzulme.md) Oyuncunun gövde dönüşü: süzülme eğimi ve riptide — S · bağımlı: T41f2 · bitti (aefb3f3a)
- [x] [T41f4](T41f4-kafatasi-durbun.md) Kafatası modelleri ve gözde dürbün — M · bağımlı: T41f3 · bitti (b1227345)
- [x] [T41g](T41g-koylu-cadi-esya.md) Köylü, tüccar ve cadının tuttuğu ve taktığı eşya — S · bağımlı: T34b, T41c · bitti (2b8c193d)
- [x] [T41d](T41d-kullanim-pozlari.md) Eşya kullanım pozları ve silah kol pozları — M · bağımlı: T41 · bölündü (acbea4de)
- [x] [T41d2](T41d2-mizrak-piglin-yerel-kullanim.md) Piglinin kol pozları — S · bağımlı: T41d · bitti (9c0b8c20)
- [x] [T41d3](T41d3-mizrak-pozlari.md) Mızrak pozları — M · bağımlı: T41d · bitti (a12c5ded)
- [x] [T41d4](T41d4-yerel-kullanim.md) Yerel oyuncunun eşya kullanımı ve sallama süresi — M · bağımlı: T41d3 · bitti (b0f60ebc)
- [x] [T42](T42-entity-golgesi.md) Entity gölgesi (`submitShadow`) — M · bağımlı: T24 · bitti (d2c2f477)
- [x] [T43](T43-keyframe-animasyon-ve-animate.md) `KeyframeAnimation` oynatıcısı, `Animate` paketi ve `EntityEvent` çizim durumları — L · bağımlı: T24 · bölündü (66840389)
- [x] [T43b](T43b-keyframe-oynatici.md) `KeyframeAnimation` oynatıcısı ve `AnimationState` (ilk tüketici tavşan, T30c) — M · bağımlı: T43 · bitti (ed3f293b)
- [x] [T44](T44-binek-surusu.md) Binek sürüşü: `PlayerInput`, `PlayerCommand`, `MoveVehicle`, `PaddleBoat` — L · bağımlı: T38, T31 · bölündü (T44a, T44b)
- [x] [T44a](T44a-tekne-surusu.md) Oyuncunun tekneyi sürmesi (tekne tiki, yolcu tiki, `MoveVehicle`, `PaddleBoat`) — L · bağımlı: T38 · bitti (a3b30282)
- [x] [T44b](T44b-binek-hayvanlar-minecart.md) Binek hayvanlar ve minecart sürüşü (at ailesi, domuz, deve, strider, minecart), binme/inme — L · bağımlı: T44a, T31 · bölündü (T44b1, T44c)
- [x] [T44b1](T44b1-eyerli-at.md) Oyuncunun eyerli atı sürmesi (at, eşek, katır, iskelet ve zombi atı; atlama yükü) — L · bağımlı: T44a, T31 · bitti (7606cdb6)
- [x] [T44c](T44c-diger-bineklar.md) Öteki binekler: eyerli domuz, strider, deve, minecart; binme/inme — L · bağımlı: T44b1 · bölündü (T44c1, T44c2)
- [x] [T44c1](T44c1-domuz-yolcu.md) Eyerli domuz ve yönetilmeyen araçta yolcu — M · bağımlı: T44b1 · bitti (db4b6e7b)
- [x] [T44c2](T44c2-strider-deve-minecart.md) Strider (lavda yürüme), deve, minecart; binme/inme — L · bağımlı: T44c1 · bölündü (7a02f132)
- [x] [T44c3](T44c3-deve-minecart.md) Deve (koşu, oturma) sürüşü — M · bağımlı: T44c2 · bitti (2ba44475)
- [x] [T44c4](T44c4-minecart-binme-inme.md) Minecart yolcusu ve binme/inme — M · bağımlı: T44c3 · bitti (37f8c4ea)
- [x] [T45](T45-saldiri-gucu-ve-combat.md) Saldırı gücü göstergesi ve `PlayerCombat*` paketleri — S · bağımlı: T26 · bitti (fd8d309d)
- [x] [T46](T46-mob-oznitelikleri.md) Mob öznitelikleri (`UpdateAttributes`): can üstü, ölçek — S · bağımlı: T34c · bitti (99446dc2)
- [ ] **K-M4** P24 kapsam denetimi: her entity türünün çizilip çizilmediğinin otomatik listesi, eksikler — M×2
- [x] [T47](T47-entity-uzaklik-ayiklamasi.md) Entity uzaklık ve bölüm görünürlüğü ayıklaması — S · bağımlı: T24 · bitti (380838be)
- [x] [T47b](T47b-asili-kutu-mermi-tiki.md) Asılı entity kutusu ve kalan mermilerin istemci tiki — M · bağımlı: T47 · bitti (de6c1cd5)

### M5 Metin ve sohbet

- [x] **K-M5** Sohbet paneli, komut önerisi/ayrıştırma, başlık, tablist, scoreboard, bossbar — M×10 · pano öncesi
- [x] [T12](T12-click-hover-modeli.md) `ClickEvent`/`HoverEvent` stil modeli + sohbet ipucu — M · bitti
- [x] [T13](T13-tiklama-eylemleri.md) Sohbet tıklama eylemleri + `ConfirmLinkScreen` — M · bağımlı: T03, T12 · bitti
- [x] [T14](T14-locator-bar.md) `TrackedWaypoint` + `LocatorBar` — M · bitti
- [x] [T15](T15-sohbet-kalanlari.md) Sohbet taslağı, seçici izin düzeyi — S · bitti
- [ ] **K-M5a** Yazı motoru kalanı: unifont yedeği, bidi, IME, metin seçimi — L×3
- [ ] **K-M5b** Sohbet kalanları: kaydırma çubuğu, `DeleteChat`, güven simgesi, onay pencereleri — M×2

### M6 Ekranlar ve ayarlar

- [x] **K-M6a** Duraklatma, ölüm ve bağlantı kopması ekranları — M×3 · pano öncesi
- [x] [T01](T01-adapter-gui-modulu.md) `behavior_adapter` GUI modülüne böl — S · bitti
- [x] [T02](T02-cases-gui-modulu.md) `cases.py` GUI üreticilerini ayır — S · bağımlı: T01 · bitti
- [x] [T03](T03-ekran-yigini-confirm.md) Ekran yığını + `ConfirmScreen` — M · bitti
- [x] [T04](T04-options-durumu.md) Seçenek değerleri ve `options.txt` — M · bitti
- [x] [T05](T05-slider-cycle-widget.md) Kaydırıcı ve döngü düğmesi widget'ları — M · bağımlı: T04 · bitti
- [x] [T06](T06-options-ekrani.md) `OptionsScreen` — M · bağımlı: T03, T05 · bitti
- [x] [T07](T07-video-ayarlari.md) `OptionsList` + `VideoSettingsScreen` (FOV, GUI ölçeği) — M · bağımlı: T06 · bitti
- [x] [T08](T08-sohbet-ayarlari.md) `ChatOptionsScreen` + sohbet ayarları çizime — M · bağımlı: T06 · bitti
- [x] [T09](T09-fare-ayarlari.md) `MouseSettingsScreen` + duyarlılık — S · bağımlı: T06 · bitti
- [x] [T10](T10-dil-secimi.md) `LanguageSelectScreen` + dil yeniden yükleme — L · bağımlı: T06 · bitti
- [x] [T10b](T10b-sohbet-dil-yenileme.md) Dil değişince sohbet geçmişini yeniden çöz — S · bağımlı: T10 · bitti
- [x] [T11](T11-ipucu-ok-odagi.md) Widget ipuçları + ok tuşlarıyla odak — M · bağımlı: T05 · bitti
- [x] [T11b](T11b-liste-klavye-odagi.md) Ayar listelerinde klavye odağı (Tab, ok) — M · bağımlı: T11 · bitti
- [x] [T66](T66-hud-durum-oracle.md) HUD durum çubukları oracle'ı (kalpler, zırh, yiyecek, hava, deneyim) — M · bağımlı: T45 · bitti (5814a8cd)
- [x] [PLAN-M6](PLAN-M3d.md) Ana menü, sunucu listesi, ilerlemeler, kitap/tabela ve kalan ekranları görevlere böl — S · bitti
- [x] [T85](T85-uygulama-durumu-baslik.md) Uygulama durumu (menü ↔ oyun) ve başlık ekranı — L · bitti (5558d989)
- [ ] [T85b](T85b-panorama.md) Başlık ekranının panoraması ve menü bulanıklığı — M · bağımlı: T85
- [x] [T86](T86-sunucu-listesi.md) Sunucu listesi ve `servers.dat` — L · bağımlı: T85 · bitti (3563fe38)
- [x] [T86c](T86c-sunucu-ekle-duzenle.md) Sunucu ekleme, düzenleme, silme ve doğrudan bağlanma — M · bağımlı: T86 · bitti (adee2f1a)
- [x] [T86b](T86b-sunucu-durumu.md) Sunucu durum sorgusu: MOTD, ping, oyuncu sayısı — M · bağımlı: T86 · bitti (3c8c8a46)
- [x] [T86d](T86d-sunucu-simgesi.md) Sunucu simgesi (favicon) çizimi ve kaydı — M · bağımlı: T86b · bitti (a183b8e4)
- [x] [T87](T87-baglanma-yukleme.md) Bağlanma ve dünya yükleme ekranları — M · bağımlı: T85 · bitti (e4b946ec)
- [ ] [T88](T88-kitap-duzenleme.md) Kitap açma ve düzenleme (`OpenBook`, `EditBook`) — M · bağımlı: T56
- [ ] [T89](T89-tabela-duzenleme.md) Tabela düzenleme (`OpenSignEditor`, `SignUpdate`) — M · bağımlı: T80d
- [ ] [T90](T90-komut-yapi-bloklari.md) Komut bloğu, yapı bloğu ve jigsaw ekranları — L · bağımlı: T85
- [ ] [T91](T91-ilerlemeler.md) İlerlemeler (`UpdateAdvancements`, `AdvancementsScreen`) — L · bağımlı: T85
- [ ] [T91b](T91b-bildirim-kutulari.md) Bildirim kutuları (toast) — M · bağımlı: T91
- [ ] [T92](T92-istatistikler.md) İstatistikler (`AwardStats`, `StatsScreen`) — M · bağımlı: T85
- [ ] [T93](T93-hata-ayiklama-katmani.md) Hata ayıklama katmanı (F3), F1 ve F2 — L
- [x] [T94](T94-tus-atamalari.md) Tuş atamaları ekranı ve atanabilir kontroller — M · bölündü (d6b5f303)
- [ ] [T94b](T94b-tus-atama-ekrani.md) Tuş atamaları ekranı (`KeyBindsScreen`) — M · bağımlı: T94
- [x] [T95a](T95a-gorus-mesafesi.md) Görüş mesafesi seçeneği (sunucuya bildirim, çizim sınırı) — S · bitti (c354678d)
- [ ] [T95](T95-kalan-ayar-ekranlari.md) Kalan ayar ekranları ve etkisiz video ayarlarının bağlanması — L
- [ ] [T96](T96-kredi-sosyal-oyun-modu.md) Kredi ekranı, sosyal etkileşimler, oyun modu seçici — M

### M7 Ses

- [x] **PLAN-M7** Ses: bağımlılık kararı ([karar](../audio-backend-decision.md): çıkış `cpal`, çözücü `lewton`) ve görevlere bölme — S · bitti
- [x] [T60](T60-sounds-json-secim.md) `sounds.json` çözümü ve ağırlıklı ses seçimi — M · bitti (bcbec5b0)
- [x] [T61](T61-ses-paketleri-kanal.md) `Sound`/`SoundEntity`/`StopSound` paketleri ve `SoundEngine` kanal parametreleri — M · bağımlı: T60 · bölündü (ba2f3f4e)
- [x] [T61b](T61b-ses-motoru.md) `SoundEngine` mantığı ve kanal parametreleri (Channel gölgesiyle oracle) — L · bağımlı: T61 · bitti (19323b90)
- [x] [T62](T62-ogg-vorbis-cozumu.md) Ogg/Vorbis çözümü (`lewton`, JOrbis ile PCM karşılaştırması) — M · bitti (16260689)
- [x] [T63](T63-ses-cikisi-cpal.md) Ses çıkışı (`cpal`): kanal havuzu, sönüm, perde, miks — L · bağımlı: T61b, T62 · bitti (f4381a44)
- [x] [T64](T64-ses-tetikleri-muzik.md) Oyun ses tetikleri (adım, kırma, hasar, ortam) ve müzik — L · bağımlı: T63 · bölündü (3659d38e)
- [x] [T64b](T64b-ses-tetikleri-kalan.md) Oyuncunun hareket sesleri ve tetik oracle'ı (adım, yüzme, sıçrama, iniş, kabarcık sütunu) — L · bağımlı: T64 · bitti (323dfa99)
- [x] [T64c](T64c-ses-tetikleri-son.md) Oyuncunun hasar ve ölüm sesi — M · bağımlı: T64b · bitti (a008d958)
- [ ] [T64d](T64d-ses-tetikleri-ortam-muzik.md) Ortam, olay sesleri ve müzik — L · bağımlı: T64c
- [x] [T65](T65-ses-ayarlari-ekrani.md) Ses ayarları ekranı — M · bağımlı: T63, T06 · bitti (a9e6230d)
- [ ] **K-M7** Altyazılar, duraklatmada süren kategoriler, kalan ses ayarları — M×2

### M8 Partikül, efekt ve blok entity

- [x] **PLAN-M8** Blok entity çizicileri + `LevelEvent` + partikül sistemini görevlere böl (`TakeItemEntity` paketi + `ItemPickupParticle` eşya toplama animasyonu da burada: parçacık motoru olmadan yapılmaz) — S · bağımlı: T24 · bitti (c09922a4)
- [x] [T70](T70-parcacik-cekirdegi.md) Partikül motoru çekirdeği ve `LevelParticles` paketi — L · bölündü (T70b, T70c)
- [x] [T70b](T70b-parcacik-motoru.md) Partikül motoru, `LevelParticles` paketi ve ekleme geçidi — M · bağımlı: T70 · bitti
- [x] [T70c](T70c-parcacik-cizimi.md) Partikül dörtgenlerinin çizimi (atlas, boru hattı) — L · bağımlı: T70b · bitti
- [x] [T71](T71-blok-parcaciklari.md) Blok parçacıkları: kırılma ve çatlama (`TerrainParticle`) — M · bağımlı: T70b · bölündü (T71b)
- [x] [T71b](T71b-blok-parcacigi-cizimi.md) Blok kırıntılarının çizimi, tintler ve oyuna bağlama — M · bağımlı: T71 · bitti
- [x] [T72](T72-yukselen-parcaciklar.md) Yükselen duman/ateş aileleri — M · bağımlı: T70b · bitti (347718df)
- [x] [T70e](T70e-blok-animasyon-tiki.md) Blokların rastgele görünüm tiki (`animateTick`): meşale, lav, su — M · bağımlı: T70b, T75 · bitti (55e57c33)
- [x] [T70f](T70f-kalan-blok-animasyonlari.md) Ateş, kamp ateşi, mum ve fırın animasyonları — L · bağımlı: T70e, T73b · bölündü (a751f4fd)
- [x] [T70g](T70g-doga-portal-animasyonlari.md) Doğa, portal ve kızıltaş blok animasyonları, ortam parçacıkları — L · bağımlı: T70f · bölündü (fb6faf8a)
- [x] [T70h](T70h-yaprak-kalan-animasyonlar.md) Düşen yapraklar — L · bağımlı: T70g · bölündü (94d54846)
- [ ] [T70i](T70i-kalan-blok-animasyonlari.md) Kalan blok animasyonları, yaprak yağmur damlası, ortam parçacıkları — L · bağımlı: T70h
- [x] [T73](T73-su-parcaciklari.md) Su ve kabarcık parçacıkları — M · bağımlı: T70b · bitti (430873ec)
- [x] [T73b](T73b-damlama-parcaciklari.md) Damlama parçacıkları (`DripParticle` ailesi) — M · bağımlı: T73 · bitti (b58c9f67)
- [x] [T74](T74-toz-buyu-parcaciklari.md) Toz, büyü ve renkli parçacıklar — M · bağımlı: T70b · bitti (e82e6d38)
- [x] [T74b](T74b-dusen-toz-havai-fisek.md) Düşen toz ve havai fişek parçacıkları — M · bağımlı: T74 · bölündü (0eced9d5)
- [ ] [T74c](T74c-havai-fisek.md) Havai fişek parçacıkları — L · bağımlı: T74b
- [x] [T75](T75-cekim-parcaciklari.md) Portal, büyü masası ve çeken parçacıklar — M · bağımlı: T70b · bitti (40baea57)
- [x] [T75b](T75b-titresim-iz.md) Titreşim ve iz parçacıkları — M · bağımlı: T75 · bitti (4e863cc9)
- [x] [T76](T76-vurus-patlama-parcaciklari.md) Vuruş, patlama ve ifade parçacıkları — M · bağımlı: T70b · bitti (76093776)
- [x] [T76b](T76b-ciglik-gardiyan-patlama.md) Çığlık, elder gardiyan parçacığı ve `Explode` paketi — M · bağımlı: T76 · bölündü (d18ef7d8)
- [x] [T76c](T76c-ciglik-gardiyan.md) Çığlık ve elder gardiyan parçacıkları — M · bağımlı: T76b · bölündü (3f9f7ca3)
- [ ] [T76c2](T76c2-elder-gardiyan.md) Elder gardiyan parçacığı (ekran çakması) — M · bağımlı: T76c, T34c
- [x] [T77](T77-esya-parcaciklari.md) Eşya parçacıkları ve eşya toplama animasyonu — M · bağımlı: T70b, T25 · bitti (ba01b9b3)
- [x] [T77b](T77b-kirilan-esya-parcaciklari.md) Kırılan eşya parçacıkları ve `ItemParticleOption` — M · bağımlı: T77 · bitti (7e60ee72)
- [x] [T78](T78-seviye-olaylari.md) `LevelEvent` ve `BlockEvent` işleyicileri — L · bağımlı: T71b, T64b · bölündü (3898df28)
- [x] [T78b](T78b-kalan-seviye-olaylari.md) Kalan seviye olayları (iksir, kemik tozu, 3000'ler, küresel, jukebox) — L · bağımlı: T78, T77b · bölündü (710bfd9a)
- [ ] [T78e](T78e-son-seviye-olaylari.md) Son seviye olayları (kemik tozu, ejderha nefesi, 3000'ler, küresel, jukebox) — L · bağımlı: T78b
- [x] [T78c](T78c-blok-olaylari.md) `BlockEvent` işleyicileri (nota bloğu, piston, çan) — M · bağımlı: T78 · bitti (cdbcfa0c)
- [ ] [T78d](T78d-kalan-blok-olaylari.md) Kalan blok olayları (piston, saksı, spawner, geçit) — L · bağımlı: T78c
- [x] [T79](T79-blok-entity-cizici-iskeleti.md) Blok entity çizici iskeleti ve sandık ailesi — L · bitti (864fe2b0)
- [x] [T79b](T79b-shulker-yatak.md) Shulker kutusu çizimi — M · bağımlı: T79 · bitti (de6b3611)
- [x] [T80](T80-tabela-bayrak-kafa.md) Tabela, bayrak, kafa ve çan blok entity çizimi — L · bağımlı: T79 · bölündü (4077713c)
- [x] [T80b](T80b-kafa-blogu.md) Yerleştirilmiş kafa ve kafatası blokları — M · bağımlı: T80 · bitti (a911614e)
- [x] [T80c](T80c-bayrak-blogu.md) Bayrak bloğu — M · bağımlı: T80 · bitti (090c9716)
- [x] [T80d](T80d-tabela-blogu.md) Tabela bloğu (ayakta ve asılı) — L · bağımlı: T80 · bitti (2aa20c9f)
- [ ] [T81](T81-isin-ve-portal-blok-entity.md) Işın, portal ve efektli blok entity'ler — M · bağımlı: T79
- [ ] [T82](T82-spawner-vault-kafes.md) Spawner, vault ve deneme spawner'ı çizimi — M · bağımlı: T79, T24
- [x] [T83](T83-hasar-ve-etki-orturleri.md) Hasarda kameranın eğilmesi (`bobHurt`) — M · bitti (4641bad8)
- [ ] [T83b](T83b-ekran-orturleri.md) Ekran örtüleri ve etki sisleri — M · bağımlı: T83
- [x] [T84](T84-dunya-siniri.md) Dünya sınırı: paketler ve çarpışma — M · bitti (33f8b5f5)
- [x] [T84b](T84b-sinir-cizimi.md) Dünya sınırının çizimi — M · bağımlı: T84 · bitti (dcb06eff)

### M9 Sunucu uyumluluğu

- [ ] **PLAN-M9** Çevrimiçi kimlik, yeniden yapılandırma, kaynak paketi görevlere böl — S
- [ ] **K-M9** Çevrimiçi kimlik, yeniden yapılandırma, kaynak paketi, transfer, diyalog — M×8 · bağımlı: PLAN-M9

### M10 Eşitlik ve dağıtım

- [ ] **K-M10a** `client-parity` kapanışları: P08, P12, P13, P16, P17, P22 — M×12
- [ ] **K-M10b** P15.a Java kare karşılaştırması — L×2
- [ ] **K-M10c** Platform matrisi ve paketleme (P04.d asset kökü) — L×2
- [ ] **K-M10d** P26 birleşik oturum: Java ve Rust aynı senaryoda — L×2

### Paralel borçlar

- [ ] **K-B1** Hareket kalanları: uçuş, elytra, auto-jump, portal, dimension (P10.d/e, P11.c) — L×3
- [ ] **K-B2** Kamera kalanları: üçüncü şahıs çarpışması, yumuşatma, sıvı kontrolü (P14) — M×2
- [ ] **K-B3** Işık/görüntü kalanları: AO, nether ışığı, gece görüşü, bulanıklık — M×3
- [ ] [Q04](Q04-math-log-intrinsic.md) `Math.log`'un JVM intrinsic'i ile bit eşitliği (kapıdaki particle-engine farkı; lisans kararı bekliyor) — M

### Kod kalitesi ve performans

[Kod kalitesi incelemesinden](../code-quality-review-2026-10-07.md) gelir; oyun ilerlemesine değil sağlığa sayılır.

- [x] [Q01](Q01-repo-hijyeni-ci.md) Public repo hijyeni ve CI — S · bitti
- [x] [Q02](Q02-ingilizce-readme.md) İngilizce README, katkı belgeleri, belge ayrımı — M · bağımlı: Q01 · bitti
- [x] [Q02b](Q02b-ingilizce-cekirdek-belgeler.md) Çekirdek belgelerin İngilizce sürümü — M · bağımlı: Q02 · bitti
- [x] [Q03](Q03-log-safety-lint.md) Log altyapısı, SAFETY yorumları, lint sıkılaştırma — M · bağımlı: Q01 · bitti
- [x] [V00](V00-performans-olcumu.md) Performans ölçümü ve bench — M · bitti
- [x] [V01](V01-vulkan-modulleri.md) `vulkan.rs` modüllere bölme ve RAII — M · bağımlı: V00 · bölündü
- [x] [V01b](V01b-vulkan-sahiplik.md) Vulkan nesnelerinde sahiplik, `create`'in bölünmesi, `DrawList` — M · bağımlı: V01 · bitti (22f03b5a)
- [x] [V02](V02-shader-dosyalari.md) Shader dosyaları, build-time SPIR-V, pipeline cache — M · bağımlı: V01 · bitti (22f03b5a)
- [x] [V03](V03-kare-ucusta.md) İki kare uçuşta, halka tampon, `RenderType` tablosu — L · bağımlı: V01 · bölündü
- [x] [V03b](V03b-render-type-tablosu.md) `RenderType` tablosu ve `DrawList` — M · bağımlı: V03 · bölündü
- [x] [V03c](V03c-record-scene-tablosu.md) `RenderType` tablosu ve `record_scene`'in tek döngüye inmesi — M · bağımlı: V03b · bitti (1190f189)
- [x] [V04](V04-present-modu.md) Present modu ve kare sınırı — S · bağımlı: V03 · bitti
- [x] [V05](V05-indexed-kompakt-vertex.md) Indexed quad ve kompakt köşe formatı — L · bağımlı: V03 · bölündü (ölçüldü: V05b)
- [ ] [V05b](V05b-paylasimli-quad-indeksi.md) Paylaşımlı opaque quad indeks tamponu — M · bağımlı: V05
- [x] [V06](V06-multi-draw.md) Bölümler için indirect/multi-draw — L · bağımlı: V05 · kapandı (ölçüm: gerekmedi)
- [x] [A01](A01-newtype-degerler.md) `JavaF64`/`DVec3` ve kimlik newtype'ları — M · bölündü
- [x] [A01b](A01b-kalan-newtype.md) Kalan `*_bits` alanları ve kimlik newtype'ları — M · bağımlı: A01 · bölündü (A01c, 43ccde6d)
- [ ] [A01c](A01c-kimlik-newtype.md) `EntityId`, `BlockStateId`, `EntityTypeId` newtype'ları — L · bağımlı: A01b
- [x] [A02](A02-entity-kind.md) `ClientEntity` → `EntityBase` + `EntityKind` enum — L · bağımlı: A01 · bitti
- [ ] [A03](A03-extract-dispatch.md) Extract → render state → tek geçişli dispatcher — L · bağımlı: A02
- [ ] [A04](A04-paket-enum-listener.md) Tipli paket enum'u, listener, izlemeyi üretimden ayırma — L · bağımlı: A01
- [x] [A05](A05-clientworld-bolme.md) `ClientWorld` alt sistemlere bölme — M · bağımlı: A02 · bölündü (A05b, 87ff5126)
- [ ] [A05b](A05b-chunk-isik-hava.md) `Chunks`, ışık, hava ve düzey bilgisi alt sistemleri — M · bağımlı: A05
- [x] [A06](A06-dev-dosyalar.md) Dev dosyaları ve oracle adaptörünü bölme — M · bölündü
- [x] [A06b](A06b-dev-islevler.md) Tek dev işlev olan dosyalar ve oracle adaptörünün crate'i — L · bağımlı: A06 · bölündü (88f77901; `mode.rs`: A04)
- [x] [D01](D01-generated-envanter.md) Üretilmiş tabloların envanteri ve sınıflandırması — S · bitti
- [ ] [D02](D02-animasyonlar-elle.md) Animasyonlar: Java sınıfı başına elle modül — M · bağımlı: D01
- [ ] [D03](D03-modeller-elle.md) Entity modelleri: model sınıfı başına `create_body_layer` — L · bağımlı: D01
- [ ] [D04](D04-bloklar-elle.md) Bloklar ve blok sınıfları elle — L · bağımlı: D01
- [ ] [D05](D05-esyalar-elle.md) Eşyalar, bileşen türleri ve yaratıcı sekmeleri elle — L · bağımlı: D01
- [ ] [D06](D06-kalan-tablolar.md) Sesler, kalan tablolar ve çalışma anı veri okuyucu — M · bağımlı: D01
- [ ] [D07](D07-uretici-kaldir.md) Üreticiyi ve `generated/` dizinini kaldırma — S · bağımlı: D02, D03, D04, D05, D06
- [x] **K-PERF** Kare süresi: entity ayıklama, kalıcı eşleme, tek çıktı tamponu — M · bitti (82384c52)

Yeni görev, kilometre taşının grubuna bir satır olarak eklenir: `- [ ] ` ve görev dosyasına bağlanan kimlik, başlık, sonra `— Boy · bağımlı: …`. Görev yazarken şablon: Amaç · Bağımlılık · Oku · Yapılacaklar · Oracle · Kabul · Kapsam dışı · Belgeler · Devir notu (≤ 60 satır).
