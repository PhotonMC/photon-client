# F1 arayüzü gizleme ve F2 ekran görüntüsü — [server-conformance değil, kaynak okuması]

Kapsam: `key.toggleGui` (F1, yalnız ekran açık değilken: `Hud.toggle`) ve `key.screenshot` (F2,
her durumda: `Minecraft.handleGlobalKeyPress`, `Screenshot.grab`).

F1: gizliyken kalpler, hotbar, çapraz nişan, deneyim çubuğu, sohbet kayıtları, boss çubukları,
başlıklar, sidebar, oyuncu listesi ve oyuncu ad etiketleri çizilmez (`Hud.extractRenderState`;
ad etiketi etkisi `entity/name-tag` oracle satırlarında zaten karşılaştırılır). El ve dünya çizilir.
F2: kare `<oyun dizini>/screenshots/yyyy-MM-dd_HH.mm.ss[_n].png` olarak yazılır
(`Screenshot.getFile`: ad doluysa `_2`, `_3`...), sohbete `screenshot.success` (altı çizili dosya adı,
`OpenFile` tıklaması) ya da `screenshot.failure` yazılır.

Kanıt: birim testler (tarih biçimi, ad sayacı, PNG başlığı) ve canlı oturum (`PHOTON_SCRIPT`
`toggle_gui`, `screenshot` adımları; gizli arayüzlü kare ve yazılan PNG görüldü). Bu dilimin Java
oracle'ı yok; davranış kaynak okumasına dayanır, `client-parity` değildir.

## Sapmalar (açık)

- Dosya adındaki saat, sistemin `date +%z` çıktısındaki farkla UTC'den kaydırılır; komut yoksa UTC.
- Gizli arayüzde altyazı katmanı ve `Darkness`/portal/dondurma kamera örtüleri henüz yok (çizilmiyor).
- F3 birleşimleri (`keyDebugModifier` basılıyken F1/F2 yutulması) yok: F3 kapsam dışı.
- PNG, `NativeImage.writeToFile` yerine zlib (flate2) ile yazılır; bayt düzeyinde aynı değildir.
