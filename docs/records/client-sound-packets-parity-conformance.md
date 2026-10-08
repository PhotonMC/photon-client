# T61 — Ses paketleri ve ses olayı kayıt defteri — [client-parity]

Bu kayıt sunucunun üç ses paketinin (`ClientboundSoundPacket`, `ClientboundSoundEntityPacket`, `ClientboundStopSoundPacket`) çözümünü ve paketlerin taşıdığı ses olayı numaralarını (yerleşik `SOUND_EVENT` kayıt defteri) kapsar. Kanıt sınıfı `client-parity`'dir: paketler gerçek `STREAM_CODEC`'lerin çözümüne, olay tablosu gerçek kayıt defterine karşı karşılaştırılır. Paketlerin ses motoruna işlenmesi ayrı görevdedir ([T61b](../tasks/T61b-ses-motoru.md)).

## Java kaynak incelemesi

- `SoundEvent.STREAM_CODEC`: bir `Holder`: `kayıt numarası + 1`, ya da `0` ve olay tam: kimlik ve isteğe bağlı sabit menzil (`boolean` + `float`). Kayıt numarası yerleşik kayıt defterindedir (sunucudan gelen veri değil); bilinmeyen numara `DecoderException`.
- `ClientboundSoundPacket`: olay, kaynak (`SoundSource` sıra numarası, `varint`), `x`, `y`, `z` (sekizde bir blok olarak `int`; `getX()` `int / 8.0F` ile **float** böler), ses, perde (`float`), tohum (`long`). `ClientboundSoundEntityPacket`: konum yerine entity kimliği (`varint`). `ClientboundStopSoundPacket`: bayrak baytı (`1` kaynak, `2` ad; diğer bitler yok sayılır), sonra kaynak ve ad.
- Geçersiz kaynak numarası (`11`) bir istisnadır; artan bayt çözümü bozmaz ama bağlantı katmanı paketi reddeder (Rust'ta oturum artan baytı hata sayar).

## Kapsam

- `sound/events` 1968 satır (kayıt defteri numara sırasıyla, her koşuda baştan sona); `sound/packets` 20 satır: kayıtlı ilk ve sonraki olay, her kaynak, tam gönderilen olay (menzilli ve menzilsiz), büyük koordinatlar (float bölümünün kaybı), geçersiz kaynak ve numara, kesik ve artan bayt, `NaN` ses, entity paketi ve durdurma bayrak birleşimleri.
- Rust: `photon-data::sound_event` (tablo, `tools/generate/sound_events.py`), `photon-game::sounds` (`SoundSource`, paket tipleri, `SoundCommand`), `photon-net` (`read_sound_packet`, `read_sound_entity_packet`, `read_stop_sound_packet`; oturum `take_sounds`).
- Canlı (`server-conformance`): `playsound` ve `stopsound` komutları ve ortam sesleri istemcide hatasız çözüldü (`sounds=4`).

## Sapmalar ve açık kalanlar

- Paketler oyun döngüsünde yalnız sayılır; ses çalınmaz (T61b ve sonrası). Oyuncu hariç tutma (`playSeededSound(player…)`) sunucu işidir.
