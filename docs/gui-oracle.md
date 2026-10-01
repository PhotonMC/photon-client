# GUI çizim oracle'ı (`gui` ailesi)

Arayüzün yerleşimi ve davranışı, Java'nın **kendi GUI kodunun** başsız çalıştırılıp çizim çağrılarının kaydedilmesiyle doğrulanır. Amaç, "Java'yı okuyup tahmin ettim" türü sapmaları ekran görüntüsüne ihtiyaç duymadan yakalamaktır.

## Nasıl çalışır

- `tests/behavior/java/GuiOracle.java`: sahte `Minecraft` (Unsafe ile), `Options`, `Hud`, `ChatListener`, `TextureManager` ve sprite/atlas kalıpları kurulur. `GuiGraphicsExtractor` gerçek sınıftır; `GuiRenderState` üzerinden her `fill`, sprite (`BlitRenderState`) ve metin (`GuiTextRenderState`) pozu uygulanmış olarak okunur.
- Yazı tipi: Java'nın kendi `font/default.json` sağlayıcıları (bitmap ve space) jar'dan okunur; glif çizilmez, yalnız ilerleme (advance) kullanılır. Yerleşim bu yüzden Java'nın yerleşimidir.
- Rust tarafı: `GuiFrame.log` (çizim günlüğü) `DrawOp` kayıtlarını tutar; `behavior_adapter` aynı senaryoyu Rust ekranıyla çalıştırıp aynı JSON'u üretir.
- Karşılaştırma: Java'nın katman sırası (çakışanı üste koyma) GPU içindir; bu yüzden iki taraf da fill → sprite → metin ve sonra yukarıdan aşağı/soldan sağa sıralanır.

## Kapsam

| operation | Java tarafı | Senaryolar |
| --- | --- | --- |
| `chat` | `ChatComponent.extractRenderState` | sarma, `\n`, solma, açık sohbet, kaydırma çubuğu |
| `suggestions` | `CommandSuggestions` + `EditBox` | seçili ilk öğe, hayalet sonek, Tab/Shift+Tab/ok/Esc, sıralama, 10 satırlık pencere |
| `titles` | `Hud.extractTitle/extractOverlayMessage` | alfa eğrisi, ölçek, süre, eylem çubuğu |
| `boss_bar` | `BossHealthOverlay.extractRenderState` | renk, çentik, ilerleme genişliği, çubuk sayısı sınırı |
| `command_parse` | `CommandDispatcher.parse`, `CommandSuggestions.formatText/updateUsageInfo` | düğüm aralıkları, istisna iletileri, renkler, kullanım/hata satırları; ağaç gerçek paket baytlarıyla |
| `command_suggest` | `CommandDispatcher.getCompletionSuggestions` | öneri metinleri, başlangıç, sunucuya sorulup sorulmadığı; `registry_backed` satırları (Java kayıt defterinden yerel, bizde sunucudan) işaretle eşitlenir |
| `pause_screen` | gerçek `PauseScreen` (init + extractBackground + extractRenderState) | ızgara düzeni, düğme/simge sprite'ları, etiket ve başlık yerleri, üzerine gelince vurgu, menü arka planı; ekran boyutları |
| `sidebar` | `Hud.extractScoreboardSidebar/displayScoreboardSidebar` | sıralama, gizli sahipler, 15 satır, sayı biçimleri, takım giydirme, takım rengine özel kenar çubuğu |
| `tab_list` | `PlayerTabOverlay` | sütunlar, sıralama (takım dahil), ping, üst/alt yazı, 80 sınırı, puan sütunu, kalpler |
| `container_screen` | `AnvilScreen`, `GrindstoneScreen`, `StonecutterScreen`, `LoomScreen`, `EnchantmentScreen` Java'nın kendi menüsüyle (`MovementOracle.openMenu`) | etiketler, doku/sprite yerleri, ad kutusu, taş kesici girdileri/kaydırma/vurgu, büyü satırları ve adları |

Yeni bir ekran için: Java'da gerçek sınıfı çağıran bir operation, `cases.py`'de senaryo satırları, adaptörde aynı sahnenin Rust karşılığı.

## Bilinen sınırlar

- Glif çizimi, eşya, varlık ve GPU katmanlama doğrulanmaz; yalnız dikdörtgen, sprite ve metin konumu/rengi.
- Seçim vurgusu (`GUI_TEXT_HIGHLIGHT`) Java'da özel bir boru hattıdır; Rust'ta düz dolgudur.
- Tarif kitabı ve ipuçları için oracle yoktur (`RecipeBookComponent` daha çok sahte durum ister); Java kaynağından satır satır okunarak birim testlerle doğrulanır. Ekranlar ipucu çizmeden karşılaştırılır (ertelenmiş ipucu `extractDeferredElements` ile çizilir, çağrılmaz).
- Kap ekranlarında Java menüsü girdi değişince sonucu kendisi hesaplar (`AnvilMenu.createResult` istemcide de çalışır): sonuç yuvası dolu senaryolar (anvil maliyet etiketi, grindstone sonucu) bu yüzden oracle'da yok; bu portlanmadı (bkz. `remaining-work.md`).
- Loom desenleri (afiş atlası), cartography ve smithing için kap ekranı oracle satırı yok.
- Taş kesici için Java `AnyFuel` görüntüsünü ilk yakıtla (lav kovası) çözer; bizim kayıt sırasıyla farklı, ipucu çizilmediği için karşılaştırmaya girmez.
- Sonraki adımlar: `blit` ile çizilen tam dokular, anvil/tarif kitabı/menü ekranları.
