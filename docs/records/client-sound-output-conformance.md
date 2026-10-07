# T63 — Ses çıkışı (`cpal`) ve miks — [server-conformance]

Bu kayıt motorun kanallarını (T61b) ses aygıtına bağlayan katmanı kapsar: kanalların miksere, miksin `cpal` çıkışına ve oyun döngüsüne bağlanması. Kanıt sınıfı **`server-conformance`**tır: duyulan çıkış için Java ile karşılaştırma yoktur ([PLAN-M7](../audio-backend-decision.md): "aynı ses dosyasını seçmek duyulan eşitlik değildir"); mikserin sayıları birim testlerinde **referansın OpenAL tanımından** (Java'nın `Channel` çağrıları) yazılmış beklentilerle sınanır, Rust çıktısından türetilmemiştir.

## Java kaynak incelemesi

- `Channel.linearAttenuation(d)`: `AL_DISTANCE_MODEL = AL_LINEAR_DISTANCE_CLAMPED`, en uzak mesafe `d`, **başvuru mesafesi 0**, sönüm çarpanı 1 → kazanç `1 − min(uzaklık, d)/d` (`d` ötesinde sessiz); `disableAttenuation` modeli kapatır. `setPitch` oynatma hızını ölçekler, `setVolume` kazançtır, `setRelative` konumu dinleyiciye göre sayar, `setLooping` tamponu döndürür; akışlı kanalda sıraya tampon konur ve `updateStream` besler.
- `Library.init`: aygıtın verdiği tek kanallı kaynak sayısı toplamdır (söylemezse 30); akış kanalları `clamp(√toplam, 2, 8)`, durağanlar `clamp(toplam − akış, 8, 255)`. Havuz dolunca `acquire` boş döner (ses başlamaz).
- `Listener.setTransform`: konum ve (ileri, yukarı) yönü her karede `updateSource(camera)` ile verilir.

## Yapılan

- `voxel-audio::mixer`: kaynaklar (konum, perde, kazanç, döngü, doğrusal sönüm, göreli, durum) ve dinleyici → stereo; perde doğrusal aradeğerlemeyle yeniden örnekleme, sönüm yukarıdaki formül, **panlama OpenAL Soft'unki değil**: sese açıyla sabit güçte sol/sağ; çok kanallı (stereo) kaynak panlanmaz, yalnız sönümlenir.
- `voxel-audio::output`: varsayılan çıkış aygıtı (`cpal`, `f32`), mikserin her geri çağrıda sol/sağı aygıtın ilk iki kanalına; aygıt yoksa `None` ve oyun sessiz sürer.
- `voxel-audio::channel`: `Channel`/`ChannelSource` uygulaması (kaynak = mikser kaynağı, havuz sayımı `Library`'nin boyutlarıyla), dosyalar varlık dizininin nesnelerinden (`.ogg` T62 çözücüsüyle; akışlı kanallar `OggStream`/`LoopingOggStream` ile).
- `voxel::sound`: `sounds.json` (varlık dizininden), motor, paket komutları (`Sound`, `SoundEntity` — entity'yi izleyen ses —, `StopSound`) ve her tikte `tick`; dinleyici her karede kameradan (`camera_vectors`). Ses seviyeleri `options.txt`'in `soundCategory_*` satırlarından (varsayılan 1).
- Canlı (`server-conformance`): `playsound` ve ortam sesleri çalındı, çökme yok (`sound=on`); sesin kulakla doğruluğu bu kayıtta doğrulanmadı.

## Sapmalar ve açık kalanlar

- **Duyulan eşitlik iddia edilmez**: panlama, filtreleme ve HRTF OpenAL Soft'a özgüdür. Kaynak sayısı aygıttan sorulmaz (30 sayılır).
- Mikser kilidi geri çağrıda alınır ve akış çözümü o kilitte yapılır (gerçek zamanlı güvenli değil); büyük müzik akışlarının ilk okuması takılabilir. Akış beslemesi ayrı iş parçacığına alınmalı.
- `SoundEventListener` (alt yazılar), aygıt listesi/değişimi ve ses ayarları ekranı (T65) yok; oyun sesleri (adım, kırma, hasar, ambiyans, müzik) tetikleri T64'tedir.
