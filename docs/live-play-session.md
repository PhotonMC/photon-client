# P11.e — Canlı oyun oturumu — server-conformance kaydı

**Tarih:** 2026-09-27 · **Referans:** Java Edition 26.2 / protokol 776 / Java 25
**Kanıt sınıfı:** `server-conformance`

Bu kayıt istemci eşitliği göstermez. Gösterdiği şey, [pinli LocalPlayer
tick'inin](local-player-tick-conformance.md) canlı bir vanilla sunucuda
sürüldüğünde sunucunun gönderilen hareketleri kabul etmesi.

## Ne yapıldı

`src/connection/play.rs` kalıcı bir oyun oturumu: giriş ve konfigürasyon
probe'larla ortak (`open_play_connection`), paketleri bir okuyucu iş parçacığı
alıyor, çağıran kareler arasında boşaltıyor. Yığın, yığın unutma, blok ve
bölüm blok güncellemeleri, yığın önbelleği merkezi/yarıçapı, yığın grubu
onayı, keep-alive, ping, oyun modu olayı, yetenekler, respawn ve teleport
işleniyor. Seviye yüklenince (oyuncunun konumu ve altındaki yığın gelince)
`player_loaded` gönderiliyor ve oyuncu tick'lenmeye başlıyor. Her istemci
tick'i oyuncuyu çalıştırıp istediği paketleri, ardından tick sonu paketini
gönderiyor.

`voxel-client play <sunucu> [genişlik yükseklik]` bir pencere açıyor: WASD,
boşluk, sol shift, sol ctrl ve fare. Kamera oyuncunun gözünde; çizilen kutu
oyuncu kenarına yaklaşınca yeniden kuruluyor.

## Ölçülen sonuç

Hayatta kalma modunda, üretilmiş normal arazide (tohum 776), betikli
girdiyle (`VOXEL_SCRIPT`) yürüme, sprint, zıplama, göle girip çıkma ve eğilme
içeren 166 tick: sunucu girişteki bir yerleştirme dışında hiç teleport
göndermedi ve günlüğünde "moved wrongly" ya da "moved too quickly" yok.

## Koşma

```
cd .data/render/walk && java -jar ../../minecraft/runtime/server-26.2.jar nogui
VOXEL_ASSETS=.data/minecraft/assets/jar/assets \
  target/release/voxel-client play 127.0.0.1:25600
```

Sunucu klasörü yerel; `server.properties` çevrimdışı mod, hayatta kalma,
barışçıl ve `server-port=25600` içeriyor.

## Eşitlik olmayanlar

- Kamera referansın kamerası değil: göz yüksekliği yumuşatılmıyor, FOV ve
  bakış sallanması yok (P14).
- Fare hassasiyeti varsayılan değerin ölçeği; yumuşatma ve ters eksen yok.
- Çizim [dünya çizim kaydındaki](server-world-render.md) eksikleri taşıyor.
  Yüz gizleme artık komşunun çarpışma şekli tam blok olduğunda yüz atıyor;
  suyun altındaki zemin ve çiçek yanındaki yüzler görünüyor, ama referansın
  yüz yüze örtülme kuralı hâlâ uygulanmıyor.
- Kare zamanlaması referansın zamanlayıcısı değil; tick'ler 50 ms birikimle
  sürülüyor.
