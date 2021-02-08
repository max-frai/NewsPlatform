#!/bin/bash


# NLP ----------------------------------------------------------------
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

    "https://github.com/IlyaGusev/tgcontest/raw/master/models/en_sentence_embedder.pt"
    "https://github.com/IlyaGusev/tgcontest/raw/master/models/en_sentence_embedder_v2.pt"
    "https://github.com/IlyaGusev/tgcontest/raw/master/models/en_sentence_embedder_v3.pt"
    "https://github.com/IlyaGusev/tgcontest/raw/master/models/ru_sentence_embedder.pt"
    "https://github.com/IlyaGusev/tgcontest/raw/master/models/ru_sentence_embedder_v2.pt"
    "https://github.com/IlyaGusev/tgcontest/raw/master/models/ru_sentence_embedder_v3.pt"
    "https://github.com/IlyaGusev/tgcontest/raw/master/models/ru_sentence_embedder_v4_text.pt"
    "https://github.com/IlyaGusev/tgcontest/raw/master/models/ru_sentence_embedder_v4_title.pt"
)

cd news_nlp
mkdir -p models
cd models
for url in ${models_list[@]}; do
    echo $url
    wget --content-disposition -nc -q $url
done


# RSMORPHY DICTS -------------------------------------------------------

models_list=(
    "https://github.com/irbis-labs/rsmorphy/raw/master/dict/ru/data/grammemes.json.gz"
    "https://github.com/irbis-labs/rsmorphy/raw/master/dict/ru/data/gramtab-opencorpora-ext.json.gz"
    "https://github.com/irbis-labs/rsmorphy/raw/master/dict/ru/data/gramtab-opencorpora-int.json.gz"
    "https://github.com/irbis-labs/rsmorphy/raw/master/dict/ru/data/meta.json.gz"
    "https://github.com/irbis-labs/rsmorphy/raw/master/dict/ru/data/p_t_given_w.intdawg.gz"
    "https://github.com/irbis-labs/rsmorphy/raw/master/dict/ru/data/paradigms.array.gz"
    "https://github.com/irbis-labs/rsmorphy/raw/master/dict/ru/data/prediction-prefixes.dawg.gz"
    "https://github.com/irbis-labs/rsmorphy/raw/master/dict/ru/data/prediction-suffixes-0.dawg.gz"
    "https://github.com/irbis-labs/rsmorphy/raw/master/dict/ru/data/prediction-suffixes-1.dawg.gz"
    "https://github.com/irbis-labs/rsmorphy/raw/master/dict/ru/data/prediction-suffixes-2.dawg.gz"
    "https://github.com/irbis-labs/rsmorphy/raw/master/dict/ru/data/suffixes.json.gz"
    "https://github.com/irbis-labs/rsmorphy/raw/master/dict/ru/data/words.dawg.gz"
)

cd ..
cd ..
mkdir -p news_rsmorphy
cd news_rsmorphy
for url in ${models_list[@]}; do
    echo $url
    wget --content-disposition -nc -q $url
done