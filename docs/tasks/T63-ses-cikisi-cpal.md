# T63 — Ses çıkışı (`cpal`): kanal havuzu ve miks

**Amaç:** T61'in kanal arayüzünü `cpal` aygıt çıkışına bağla: kaynak başına konum/perde/ses/döngü, doğrusal mesafe sönümü (`AL_LINEAR_DISTANCE_CLAMPED` modeli), dinleyici yönüne göre panlama, akış (kuyruk), duraklatma. Yeni üretim bağımlılığı: `cpal` (kararlı sürüm 0.18.2).
**Bağımlılık:** T61, T62. Ses yığını kararı: [`audio-backend-decision.md`](../audio-backend-decision.md) (çıkış `cpal`, çözücü `lewton`).

## Oku
- Java: `com/mojang/blaze3d/audio/{Library,Channel,Listener,ListenerTransform}.java`, `client/sounds/{SoundEngine,ChannelAccess}.java`.
- Rust: T61 kanal arayüzü; `crates/voxel/src/play.rs` (karenin ses güncellemesi).

## Yapılacaklar
1. Mikser: perde için yeniden örnekleme, doğrusal sönüm, sol/sağ panlama, kaynak sayısı sınırı.
2. Dinleyici güncellemesi (konum, yön) her karede (`Listener`); aygıt yoksa sessiz çalış (hata değil).
3. Duyulan çıktı yakalama yöntemi bu görevin kaydında yazılır (eşitlik iddiası yok).

## Oracle
Mantık T61'in oracle'ında; mikser için birim testleri (sönüm, panlama, perde) ve kural: beklenen değer Java'nın `AL_*` tanımlı modelinden yazılır, Rust çıktısından türetilmez.

## Kabul
`tools/voxel.py check --capture` → `CHECK OK`; canlı: `summon`/blok kır, ses duyulur (kullanıcı değil, yakalama ya da günlük).

## Kapsam dışı
HRTF, aygıt listesi/değişimi (`ALC_SOFT_system_events`), OpenAL Soft bağlama (ileride P26).

## Belgeler
`docs/index.md` satırı, yeni kayıt.

## Devir notu
- Yapıldı: `voxel-audio` mikser/çıkış/kanal (`cpal` 0.18.2), `voxel::sound` (motor + paketler + dinleyici + `options.txt` seviyeleri). Kayıt: `docs/client-sound-output-conformance.md` (server-conformance; duyulan eşitlik yok).
- Açık: kulakla doğrulama kullanıcıda; akış beslemesi iş parçacığı; aygıt kaynak sayısı sorulmuyor; T64 (tetikler), T65 (ayar ekranı).
