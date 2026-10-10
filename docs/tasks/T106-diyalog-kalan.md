# T106 — Diyaloglar 2: çok eylemli, liste, sunucu bağlantıları ve girdiler

**Amaç:** `MultiButtonDialogScreen`, `DialogListDialogScreen`, `ServerLinksDialogScreen` ve girdi denetimleri (metin, onay kutusu, tek seçim, sayı aralığı).
**Bağımlılık:** T105, T100. **Boy:** L.

## Oku
- Java: `client/gui/screens/dialog/{MultiButtonDialogScreen,ButtonListDialogScreen,DialogListDialogScreen,ServerLinksDialogScreen,DialogControlSet}.java`, `dialog/input/*`.
- Rust: T105'in ekranı.

## Yapılacaklar
`ClientboundServerLinksPacket` çözümü (bileşenli bağlantı türü) T100'den buraya devredildi. Tarife göre bölün: çok eylemli+liste / girdiler / sunucu bağlantıları.

## Oracle
G satırları her ekran için.

## Kabul
`CHECK OK`; canlıda Pumpkin'e karşı bir bağlantı (`PHOTON_SCRIPT`), `server-conformance` kaydıdır, `client-parity` değildir.

## Kapsam dışı
—

## Belgeler
`docs/gui-oracle.md`, kayıt güncellemesi.

## Devir notu
- Yapıldı: çok eylemli diyalog (`MultiButtonDialogScreen`) — `Dialog::MultiAction`, `button_list()`, ekranda sütun ızgarası + kaydırılan/kırpılan gövde düğmeleri + çıkış düğmesi/kısa altbilgi; oracle `dialog_screen` +41 satır (65 senaryo hepsi eşit).
- Bölündü: `DialogListDialogScreen` (kayıt defteri + `#etiket` çözümü, `computeExternalTitle`) → T106b; `ServerLinksDialogScreen` (`ClientboundServerLinksPacket` çözümü, `OpenUrl`) → T106c; girdi denetimleri (metin, onay kutusu, tek seçim, sayı aralığı, `dynamic/*` eylemleri) → T106d.
- Java'dan sapma: yok (kapsamdaki satırlarda). Canlı Pumpkin denemesi yapılmadı.
