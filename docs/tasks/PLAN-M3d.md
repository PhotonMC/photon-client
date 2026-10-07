# PLAN-M3d / PLAN-M6: kalan kap ekranlarını ve menüleri görevlere böl

PLAN-M3d ve PLAN-M6 kod yazmaz; [`PLAN.md`](PLAN.md) usulüyle iki kilometre taşını bağımsız, tek oturumluk görev dosyalarına böler.

- **PLAN-M3d:** [`remaining-work.md`](../remaining-work.md) §3. Tezgâh + tarif kitabı çekirdeği, fırın ailesi (`ContainerSetData` ilerleme oku), örs (`RenameItem`), büyü masası (`ContainerButtonClick`), taş kesici, loom, cartography, grindstone, smithing, beacon (`SetBeacon`), simya standı, kitaplık/lectern, tüccar (`MerchantOffers`, `SelectTrade`; T34'ün köylüsüne bağlanır), at envanteri; kap menü eksikleri listesi (off hand `SWAP`, bileşen hash'i, bundle). Java kaynağı: `client/gui/screens/inventory/*`, `world/inventory/*`.
- **PLAN-M6:** §6. Ana menü ve sunucu listesi, yükleme/alma ekranları, ilerlemeler ve istatistik (`UpdateAdvancements`, `AwardStats`), kitap/tabela/komut bloğu/yapı bloğu/jigsaw ekranları, F3/F1/F2, sosyal ekran, toast, kredi; Key Binds ve ses/video/erişilebilirlik alt ayar ekranları.

Adımlar PLAN.md ile aynı; kimlikler T50'den, GUI görevleri "G" tarifini (README) izler. Bitince bu satırlar `bitti` olur. PLAN-M6 bitti: T85–T96 (pano, M6 grubu).
