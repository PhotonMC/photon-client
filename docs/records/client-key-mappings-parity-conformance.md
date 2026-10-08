# T94 — Tuş eşlemeleri ve `key_` satırları — [client-parity]

Bu kayıt `Options.keyMappings`'i kapsar: adlar, kategoriler, varsayılan tuşlar, sıra ve
`InputConstants`'ın tuş adları. Kanıt sınıfı `client-parity`'dir. `gui/key-mappings` yeni bir
`Options`'ın eşlemelerini, kategori sırasını ve adı olan tüm tuşları yakalar. Rust'ın üretilmiş
tablosu her koşuda bunlarla tümüyle karşılaştırılır. Tablo
`python3 tools/generate/key_mappings.py <capture>/outputs.jsonl` ile üretilir.

## Java kaynak incelemesi

- `keyMappings` sırası saldırı, kullanma ve hareketle başlar; kısayol yuvaları sonda gelir.
  Toplam 61 eşleme ve sekiz kategori vardır (`SORT_ORDER`).
- `options.txt`'de her eşlemenin `key_<ad>:<tuş adı>` satırı bulunur. Okumada
  `InputConstants.getKey` önce ad tablosuna bakar. Yoksa türün önekinden sonraki sayıyı okur
  (fare düğmesi birden sayılır); okunamayan ad eşlemeyi değiştirmez. Yazmada ad
  `Key.getName`'dir.
- Bir tuş birden çok eşlemeye bağlı olabilir (`KeyMapping.MAP`); hepsi aynı olayı alır.

## Kapsam ve dal kararı

İstemci artık hareket, saldırı, kullanma, seçme, envanter, sohbet ve komut, bakış açısı, tam
ekran, oyuncu listesi ve kısayol yuvalarını eşlemelerden okur. Esc sabittir (eşleme değildir).
`options.txt`'deki `key_` satırları okunur ve yazılır (birim testi). Ekran T94b'dedir.

## Sapmalar (açık)

- Geçişli (toggle) çömelme/koşma, bırakma, el değiştirme, ekran görüntüsü ve hata ayıklama
  tuşları bağlı değil; bunların davranışı kendi görevlerinde.
