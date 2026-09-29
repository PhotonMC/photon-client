# P13.a — Envanter menüsünde tıklama tahmini — client-parity kaydı

**Tarih:** 2026-09-29 · **Referans:** Java Edition 26.2 / protokol 776 / Java 25
**Kanıt sınıfı:** `client-parity`

## Ne pinlendi

`src/inventory_menu.rs`, oyuncunun kendi envanter menüsü (`InventoryMenu`)
için `AbstractContainerMenu.doClick` ve
`MultiPlayerGameMode.handleContainerInput`:

- Yuva düzeni ve kuralları: üretim sonucu (koyulamaz, alınınca bütün yığın
  gelir), 2×2 ızgara, zırh yuvaları (en çok bir, yalnız o yuvaya
  kuşanılabilen ve oyuncunun giyebildiği eşya), ana envanter, hotbar, sol el.
  Kap sınırı 99, yığın sınırı eşyanın.
- Tıklama türleri: sol/sağ tık (alma, yarısını alma, koyma, tek koyma,
  birleştirme, takas), pencere dışına bırakma, shift (`quickMoveStack`:
  satırlar arası, zırh ve kalkanın kendi yuvasına, sonuç yuvasından geriye
  doğru), sayı tuşları ve F (`SWAP`, zırh yuvasına taşma), Q ve Ctrl+Q
  (`THROW`), çift tık (`PICKUP_ALL`, iki geçiş, iki yön), sürükleme
  (`QUICK_CRAFT` durum makinesi, sol/sağ/orta), `CLONE` (yalnız sonsuz
  malzemede).
- Sonuç yuvasından alınca istemcinin ızgarayı bir azaltması ve üretim
  artıkları (`defaultCraftingReminder`: kova, şişe), artığın envantere
  (`Inventory.add`: seçili yuva, sol el, sonra boş yuva) ya da yere gitmesi.
- İstemcide `drop` yere eşya atmaz, ana kolu sallar: pencere dışına bırakma
  ve atma, tıklama paketinden önce `swing` paketi gönderir.
- Paket: kap, durum sayacı, yuva, düğme, tür, değişen yuvalar (tıklamadan
  önceki kopyayla `ItemStack.matches`) fastutil `Int2ObjectOpenHashMap`
  yineleme sırasıyla (sıfır anahtar önce, sonra tablo sondan; 24'ü aşınca
  büyüme) ve imleçteki yığın, `HashedStack` olarak.

Tıklamanın okuduğu eşya verisi referanstan tabloya eklendi ve her koşuda
karşılaştırılıyor: yığın sınırı, dayanıklılık, `equippable` yuvası ve
oyuncunun giyip giyemediği, üretim artığı, tıklamayı değiştiren sınıf
(`block/item-stacking`, `src/item_stacking_table.rs`,
`tools/generate/item_stacking.py`).

## Senaryolar

`tools/behavior/cases.py` içindeki `movement/container-clicks`: gerçek
`LocalPlayer`, `InventoryMenu` ve `MultiPlayerGameMode` üzerinde
`handleContainerInput`. Her tıklamadan sonra giden paketlerin baytları, 46
yuva ve imleçteki yığın. 29 senaryo: yukarıdaki türlerin her dalı, 16'lık ve
tekil yığınlar, zırh yuvasının reddi, sonuç yuvası (alma, shift, sayı tuşu,
artıklar, koymayı reddetme), 36 yuvaya sürükleme (tablonun büyümesi) ve
sıfır anahtarı.

## Sonuç

`compare --family movement` ve `--family block`: `parity_passed`.

## Eşitlik olmayanlar ve açıklar

- Bileşen yaması olan yığınların bileşen hash'leri port edilmedi; pakette
  yamasız yazılır. Sunucu farkı görür ve yuvayı yeniden gönderir.
- Bundle'ın tıklama davranışı port edilmedi: imleçte ya da tıklanan yuvada
  bundle varken tahmin yapılmaz, paket yine gider.
- Hasarlı yığınlar hasarsız sayılır (`isStackable`, `Inventory.add`'in
  hasarlı dalı). Zırhı kilitleyen büyü (`PREVENT_ARMOR_CHANGE`) okunmaz.
- Yaratıcı moddaki atma paketi (`handleCreativeModeItemDrop`) yok; oyuncu
  envanter ekranındayken referans da göndermez. Yaratıcı envanter ekranı
  ayrı bir dilimdir.
- Sonuç yuvasının istatistik ve `onCraftedBy` yan etkileri yok (istemcide
  paket ya da yuva farkı doğurmuyor).
