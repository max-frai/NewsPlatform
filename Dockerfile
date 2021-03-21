# --------------------------------------------------

FROM rust:1.48-buster as planner
RUN echo "PREPARE CARGO CHEF PLANNER"
WORKDIR /newsplatform/
RUN cargo install cargo-chef
ADD news_general ./news_general
ADD news_parser ./news_parser
ADD news_server ./news_server
ADD news_websocket ./news_websocket
ADD Cargo.toml .
ADD Cargo.lock .
RUN cargo chef prepare  --recipe-path recipe.json
RUN ls -la

# --------------------------------------------------

FROM rust:1.48-buster as cacher
RUN echo "PREPARE CARGO CHEF CACHER"
WORKDIR /newsplatform/
RUN cargo install cargo-chef
COPY --from=planner /newsplatform/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
RUN ls -la

# --------------------------------------------------

FROM rust:1.48-buster as builder
RUN echo "PREPARE CARGO CHEF BUILDER"
WORKDIR /newsplatform/

# Possibly move models downloading to cacher or planner
ADD news_nlp ./news_nlp
RUN ls -la
ADD download_models.sh .
RUN chmod u+x download_models.sh && ./download_models.sh
RUN ls -la

ADD news_general ./news_general
ADD news_parser ./news_parser
ADD news_server ./news_server
ADD news_websocket ./news_websocket
ADD Cargo.toml .
ADD Cargo.lock .
COPY --from=cacher /newsplatform/target target
COPY --from=cacher /usr/local/cargo /usr/local/cargo
RUN ls -la
RUN cargo build --release
RUN ls -la

# --------------------------------------------------

FROM rust:1.48-buster as runtime
RUN echo "PREPARE CARGO CHEF RUNTIME"

RUN echo "deb http://ftp.de.debian.org/debian buster main" >> /etc/apt/sources.list

# Chromium & NodeJS
RUN apt update && apt install -y curl xvfb chromium psmisc
RUN curl -sL https://deb.nodesource.com/setup_14.x | bash -
RUN apt-get install -y nodejs cmake libboost-all-dev build-essential libjsoncpp-dev uuid-dev protobuf-compiler libprotobuf-dev python3-pip magic-wormhole vim wget
RUN npm install --save-dev autoprefixer tailwindcss postcss postcss-cli postcss-loader cssnano
RUN npm install -g autoprefixer tailwindcss postcss postcss-cli postcss-loader cssnano

# Spacy
RUN pip3 install --upgrade pip
RUN pip3 install -U pip setuptools wheel
RUN pip3 install -U spacy aiohttp
RUN python3 -m spacy download ru_core_news_lg

WORKDIR /newsplatform/

ADD news_nlp ./news_nlp
COPY --from=builder /newsplatform/news_nlp/models ./news_nlp/models
COPY --from=builder /newsplatform/news_rsmorphy ./news_rsmorphy/

RUN cd /usr/lib && wget https://www.dropbox.com/s/yqgqiasz1weej96/libc10.so?dl=0 -O libc10.so
RUN cd /usr/lib && wget https://www.dropbox.com/s/gny94fzwggussjd/libgomp-75eea7e8.so.1?dl=0 -O libgomp-75eea7e8.so.1
RUN cd /usr/lib && wget https://www.dropbox.com/s/c209ske4r4do6fm/libtorch_cpu.so?dl=0 -O libtorch_cpu.so
RUN cd /usr/lib && wget https://www.dropbox.com/s/v38nr9q5c11ic5s/libtorch.so?dl=0 -O libtorch.so

ADD news_svelte ./news_svelte
ADD news_templates ./news_templates
ADD news_ner.py .
ADD postcss.config.js .
ADD tailwind.config.js .
ADD Config.toml .
ADD cert.pem .
ADD key.pem .

RUN cd news_svelte && npm i && npm run build && cd ..

COPY --from=builder /newsplatform/target/release/news_server .
COPY --from=builder /newsplatform/target/release/news_parser .
COPY --from=builder /newsplatform/target/release/news_websocket .
COPY --from=builder /newsplatform/news_parser/rewritebinary_linux .
COPY --from=builder /newsplatform/news_parser/parserbinary_linux .

# COPY --from=builder /newsplatform/news_parser/text2wikititle .
# RUN chmod 755 text2wikititle

RUN chmod u+x rewritebinary_linux && chmod u+x parserbinary_linux && chmod u+x news_nlp/nlp_linux && chmod u+x news_websocket

RUN ls -la
ENTRYPOINT ["./run.sh"]