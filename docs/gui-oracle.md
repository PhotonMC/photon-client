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
| `tab_list` | `PlayerTabOverlay` | sütunlar, sıralama, ping, üst/alt yazı, 80 sınırı |

Yeni bir ekran için: Java'da gerçek sınıfı çağıran bir operation, `cases.py`'de senaryo satırları, adaptörde aynı sahnenin Rust karşılığı.

## Bilinen sınırlar

- Glif çizimi, eşya, varlık ve GPU katmanlama doğrulanmaz; yalnız dikdörtgen, sprite ve metin konumu/rengi.
- Seçim vurgusu (`GUI_TEXT_HIGHLIGHT`) Java'da özel bir boru hattıdır; Rust'ta düz dolgudur.
- Tarif kitabı, kap ekranları ve ipuçları için oracle yoktur (`RecipeBookComponent` ve menüler daha çok sahte durum ister); bu ekranlar Java kaynağından satır satır okunarak birim testlerle doğrulanır.
- Sonraki adımlar: `blit` ile çizilen tam dokular, anvil/tarif kitabı/menü ekranları.
