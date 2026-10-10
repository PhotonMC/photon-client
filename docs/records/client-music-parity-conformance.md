# T64e — Müzik (`MusicManager`) — [client-parity]

Kapsam: `MusicManager.tick` (ilk şarkıdan önceki bekleme, şarkı bitince bekleme, ortamın müziği
çalan şarkının yerini alırsa durdurma, müzik sesine göre solma), müzik sıklığı seçeneği
(`DEFAULT`, `FREQUENT`, `CONSTANT`) ve `BackgroundMusic.select` (yaratıcı, su altı).
Kanıt: `gui/music-manager` satırları gerçek `MusicManager`'ı sahte bir `Minecraft` ve kayıt tutan
`SoundManager` ile tikletir; bekleme, çalan şarkı, ses kazancı ve her tikte sese istenenler bit bit
eşittir (`client-parity`).

Oyunda: oyuncunun durduğu biyomun `minecraft:audio/background_music` ve `music_volume`
özniteliği, yoksa boyutunki (`crates/photon/src/music.rs`); şarkı `SoundSource::Music` altında,
ölçeksiz ve göreli çalar. Canlıda orman biyomunda `music.overworld.forest` çalmaya başladı
(`photon::sound` günlüğü); kulakla dinlenmedi.

## Sapmalar (açık)

- Ekranların müziği (menü, krediler), Ender ejderhası patlaması (`END_BOSS`), yükleme ekranı
  beklemesi ve "şimdi çalıyor" bildirimi yok; `Screen.getBackgroundMusic` bağlanmadı.
- Öznitelik sistemi yalnız müzik için okunur: zaman çizelgesi ve hava değiştiricileri yok, biyom
  değeri boyutunkini yerine koyar. Biyom, oyuncunun bulunduğu bloktan okunur (kamera değil).
- Müzik ses seviyesi (`music_volume`) biyomdan okunur; sınırlar arası harmanlanmaz.
