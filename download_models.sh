#!/bin/bash

models_list=(
    "https://www.dropbox.com/s/qq970kin8zagql7/lang_detect.ftz"
    "https://www.dropbox.com/s/23x35wuet280eh6/ru_cat_v5.ftz"
    "https://www.dropbox.com/s/luh60dd0uw8p9ar/en_cat_v5.ftz"
    "https://www.dropbox.com/s/vttjivmmxw7leea/ru_vectors_v3.bin"
    "https://www.dropbox.com/s/6aaucelizfx7xl6/en_vectors_v3.bin"
    "https://www.dropbox.com/s/0o9xr2pwuqeh17k/pagerank_rating.txt"
    "https://www.dropbox.com/s/fry1gsd1mans9jm/alexa_rating_4_fixed.txt"
    "https://www.dropbox.com/s/hoapmnvqlknmu6v/lang_detect_v10.ftz"
    "https://www.dropbox.com/s/5r1mhplhnnfr9xh/ru_idfs.txt"
    "https://www.dropbox.com/s/0up3us2ey999mgc/ru_tfidf_tsvd_embedder.pt"

    "https://github.com/IlyaGusev/tgcontest/blob/master/models/en_sentence_embedder.pt?raw=true"
    "https://github.com/IlyaGusev/tgcontest/blob/master/models/en_sentence_embedder_v2.pt?raw=true"
    "https://github.com/IlyaGusev/tgcontest/blob/master/models/en_sentence_embedder_v3.pt?raw=true"
    "https://github.com/IlyaGusev/tgcontest/blob/master/models/ru_sentence_embedder.pt?raw=true"
    "https://github.com/IlyaGusev/tgcontest/blob/master/models/ru_sentence_embedder_v2.pt?raw=true"
    "https://github.com/IlyaGusev/tgcontest/blob/master/models/ru_sentence_embedder_v3.pt?raw=true"
)

cd news_nlp
mkdir models
cd models
for url in ${models_list[@]}; do
    echo $url
    wget --content-disposition -nc -q $url
done