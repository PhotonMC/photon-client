# T96d — Sosyal etkileşimler ekranını oyuna bağla

**Amaç:** P tuşuyla açılan ekran tıklanır ve yazılır; gizle/göster düğmesi sohbet süzgecine bağlanır.
**Bağımlılık:** T96c. **Boy:** M. [T96c](T96c-sosyal-etkilesimler.md)den ayrıldı.

## Oku
Java: `client/gui/screens/social/{SocialInteractionsScreen,PlayerEntry,SocialInteractionsPlayerList,PlayerSocialManager}.java`, `Minecraft.java` (`shouldHideMessageFrom`). Rust: `photon-render/src/gui/screen/social.rs`; `crates/photon/src/{controls,gui,play}.rs`, `controls/{pause,death,confirm}.rs` (`open_pause` örnek); sohbet günlüğü ve tablist oyuncu bilgisi.

## Yapılacaklar
- Ekran: sekme ve Done tıklaması, hide/show düğmesi (sohbete "gizlendi/gösterildi" sistem iletisi), arama kutusuna yazma ve odak (kaybedince ipucu), liste kaydırma ve tekerlek, P tuşu ile kapanma (arama odakta değilken), oyuncu girişi/çıkışı (`onAddPlayer`/`onRemovePlayer`).
- Oyun: `keySocialInteractions` ile açma; gizli oyuncuların sohbet iletileri `shouldHideMessageFrom` ile süzülür; tablist'ten oyuncu listesi.
- Oracle: tıklama/yazma adımları (`click`, `type`), kaydırma satırı.

## Kapsam dışı
Raporlama ekranları (çevrimiçi servis), engelleme (çevrimiçi mod), düğme ipuçları.

## Kabul
`tools/photon.py check --capture` → `CHECK OK`.

## Belgeler
`docs/gui-oracle.md` satırını güncelle.

## Devir notu
Yapıldı: ekran etkileşimi (`photon-render/.../social/input.rs`: sekme, gizle/göster, Bitti, engelleme düğmesi, çubuk sürükleme, tekerlek, arama yazma/odak, P/Esc) Java'ya karşı oracle'da (`steps`, olaylar ve sohbet satırları dahil); oyunda P tuşu, `controls/social.rs`, gizli oyuncular (`Controls::hidden_players`), `shouldHideMessageFrom` sohbet süzgeci, `seen_players` ve `social_players()` (photon-net).
Sapma/açık: bildir düğmesi no-op (rapor ekranı çevrimiçi servis ister); Tab/ok odağı, Enter ve çift tıklama yok; sunucu adı hep `selectServer.defaultName` (liste girişi adı oyuna taşınmıyor); boyut değişince Java listeyi süzmeden bırakır, Rust süzer; bağlantı onayından dönüşte Java ekranı yeniden kurar, Rust kurmaz; sistem iletilerinde `hideMatchedNames` yok.
