# P06.a — VarInt21 frame sınırı sözleşmesi

Bu dilim, Java istemcisinde paket çözümlemesinden önce kullanılan uzunluk başlığı sınırını taşır. Prepender okunabilir gövdeyi alır, gövde uzunluğunu VarInt olarak başa yazar ve gövde baytlarını aynı sırada ekler. Başlık üç bayta sığmıyorsa yazma işlemi encoder hatasıyla sonuçlanır; gövde boş olsa bile `00` başlığı geçerlidir. Yazma yolu okunabilir gövdenin başlangıcını kullanır ve kaynak mesajın reader index'ini ilerletmez.

Frame decoder her çağrıda girişin mevcut reader konumunu işaretler ve en fazla üç başlık baytı toplar. Başlık veya gövde eksikse işaretlenen konuma döner; böylece sonraki veriyle aynı frame yeniden denenir. Üç devam baytı genişlik hatasıdır, sıfır uzunluk hatasıdır. Başarıda başlık ile tam bir gövde tüketilir ve tek frame çıkarılır; aynı çağrıda kalan veri sonraki çağrıya bırakılır.

Rust tarafındaki `frame::prepend` ve `Varint21FrameDecoder::feed`, bu sınır davranışını byte slice ve parçalı input ile sunar. Decoder sonucu frame, eksik durum veya hata olabilir; tüketilen byte sayısı ve kalan input ayrıca gözlemlenebilir.

Bu sözleşme socket/channel yaşam döngüsünü, packet türlerini, protocol state geçişini, compression/encryption katmanlarını ve login akışını kapsamaz. Test girdileri özgün sentetik gövde ve framing byte dizileridir; referans kaynakları `.data/` altında yerel kalır.
