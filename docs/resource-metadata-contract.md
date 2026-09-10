# P04.b — Pack metadata uyumluluğu sözleşmesi

Pack formatı major/minor çifti olarak karşılaştırılır; major eşitse minor değeri belirleyicidir. Java Edition 26.2 client resource-pack formatı `88.0` için declared range'in altındaki aralık `TOO_OLD`, üstündeki aralık `TOO_NEW`, kesişen aralık `COMPATIBLE` olarak sınıflandırılır. `Integer.MAX_VALUE` major ile başlayan sentinel aralık `UNKNOWN` sonucudur.

Bu dilim pack.mcmeta JSON çözümünü, description component'ını, feature flag'lerini, overlay seçimlerini ve repository keşfini kapsamaz. Test girdileri yalnızca özgün sentetik format aralıklarıdır.
