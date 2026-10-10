# Sunucu kaynak paketi isteği ve onay ekranı — [ekran: client-parity; paketler: sahte sunucu testi]

Kapsam: yapılandırma (`8`/`9`/`6`) ve oyun (`80`/`81`/`49`) evresinde `ResourcePackPop`, `ResourcePackPush` ve
`ServerboundResourcePack`. Oyunda web olmayan adres hemen `INVALID_URL` ile yanıtlanır, diğer paketler
oyuncuya sorulur (`PackConfirmScreen`: zorunlu/zorunsuz söz dizimi, sunucunun kendi istemi). Yapılandırma
evresinde ekran olmadığı için paket `DECLINED` yanıtlanır. Kabul `ACCEPTED` ve ardından `FAILED_DOWNLOAD`
gönderir; red `DECLINED` gönderir, zorunluysa bağlantıyı keser.

Kanıt: paket birim testleri, iki sahte sunucu testi (oyun ve yapılandırma) ve `confirm_screen` GUI
oracle'ında `pack-*` satırları (Java `ConfirmScreen` ile aynı çizim; metinler Java kodundaki gibi kurulur,
çünkü `PackConfirmScreen` özeldir). Canlı Pumpkin bağlantısı denenmedi.

## Sapmalar (açık)

- İndirme, doğrulama ve yükleme yok (T104): kabul edilen paket `FAILED_DOWNLOAD` ile bitirilir.
- `ServerData`'daki paket durumu (`ENABLED`/`DISABLED`) saklanmaz; ekran kapanınca önceki ekran yoktur.
- `Pop` yok sayılır (yüklü paket yok).
