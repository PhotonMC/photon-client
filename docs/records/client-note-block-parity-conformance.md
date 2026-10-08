# T78c — Nota bloğu (`BlockEvent`) — [client-parity]

Bu kayıt `ClientLevel.blockEvent` üzerinden `NoteBlock.triggerEvent`'i kapsar. Kanıt sınıfı
`client-parity`'dir. `movement/particle-engine`'de sahneye gerçek nota bloğu durumları konur ve
`Level.blockEvent` çağrılır. Çalınan ses (olay, kategori, ses, perde, yer) ve nota parçacığı
tohumlu motorla Rust'a bit bit eşitlenir.

## Java kaynak incelemesi

- Olayı paketteki blok değil, o anki blok durumu alır. Sandık, shulker ve çan blok
  entity'lerinde, nota bloğu blokta işlenir.
- Çalgı durumun `instrument` özelliğidir. Ayarlanabilir çalgılarda (temel blok türü) nota
  parçacığı `(x+0.5, y+1.2, z+0.5)`'te `note/24` itişle çıkar, perde
  `(float) 2^((note−12)/12)` olur. Mob kafası çalgıları perde 1 ile çalar. `custom_head` üstteki
  kafanın sesini ister; kafa yoksa ses çalmaz.
- Ses `RECORDS` kategorisinde, 3 ses düzeyiyle bloğun ortasında çalar. Seviye rastgelesinden
  tohum, parçacıktan sonra çekilir.

## Kapsam ve dal kararı

Tek satır: ayarlı (0, 12, 24), bas davul, oksitlenmiş trompet, zombi ve ejderha kafası,
`custom_head`, aynı tikte iki vuruş ve taş blok. Durum paket anında okunur; olay seviye
olaylarıyla aynı kuyruğa girer.

## Sapmalar (açık)

- `custom_head`'in üstteki kafadan okunan sesi (`note_block_sound`) yok; sessiz.
- Piston, saksı sallanması, spawner, end geçidi ve potent sulfur blok olayları açık (T78d).
