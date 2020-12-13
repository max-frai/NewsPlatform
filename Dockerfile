
FROM rust:1.48-buster as planner
RUN echo "PREPARE CARGO CHEF PLANNER"
WORKDIR /newsplatform/
RUN cargo install cargo-chef
ADD news_general ./news_general
ADD news_parser ./news_parser
ADD news_server ./news_server
ADD Cargo.toml .
ADD Cargo.lock .
RUN cargo chef prepare  --recipe-path recipe.json
RUN ls -la

FROM rust:1.48-buster as cacher
RUN echo "PREPARE CARGO CHEF CACHER"
WORKDIR /newsplatform/
RUN cargo install cargo-chef
ADD download_models.sh .
RUN chmod u+x download_models.sh && ./download_models.sh
COPY --from=planner /newsplatform/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
RUN ls -la
RUN ls -la ./models

FROM rust:1.48-buster as builder
RUN echo "PREPARE CARGO CHEF BUILDER"
WORKDIR /newsplatform/
ADD news_general ./news_general
ADD news_parser ./news_parser
ADD news_server ./news_server
ADD Cargo.toml .
ADD Cargo.lock .
COPY --from=cacher /newsplatform/target target
COPY --from=cacher /usr/local/cargo /usr/local/cargo
RUN ls -la
RUN ls -la ./target/release
RUN ls -la /usr/local/cargo
RUN cargo build --release
RUN ls -la

FROM rust:1.48-buster as runtime
RUN echo "PREPARE CARGO CHEF RUNTIME"
RUN echo "deb http://ftp.de.debian.org/debian buster main" >> /etc/apt/sources.list
RUN apt update && apt install -y curl xvfb chromium psmisc
RUN curl -sL https://deb.nodesource.com/setup_14.x | bash -
RUN apt-get install -y nodejs cmake libboost-all-dev build-essential libjsoncpp-dev uuid-dev protobuf-compiler libprotobuf-dev
RUN npm install --save-dev autoprefixer tailwindcss postcss postcss-cli postcss-loader
RUN npm install -g autoprefixer tailwindcss postcss postcss-cli postcss-loader
WORKDIR /newsplatform/
COPY --from=builder /newsplatform/news_parser/libgomp-75eea7e8.so.1 /usr/lib/
COPY --from=builder /newsplatform/news_parser/libtorch_cpu.so /usr/lib/
COPY --from=builder /newsplatform/news_parser/libc10.so /usr/lib/
COPY --from=builder /newsplatform/news_parser/libtorch.so /usr/lib/
COPY --from=builder /newsplatform/news_parser/configs ./configs
COPY --from=cacher /newsplatform/models ./models
ADD news_server/templates ./templates
ADD news_server/postcss.config.js .
ADD news_server/tailwind.config.js .

COPY --from=builder /newsplatform/target/release/news_server .
COPY --from=builder /newsplatform/target/release/news_parser .
COPY --from=builder /newsplatform/news_parser/rewritebinary_linux .
COPY --from=builder /newsplatform/news_parser/parserbinary_linux .
COPY --from=builder /newsplatform/news_parser/nlp_linux .

# COPY --from=builder /newsplatform/news_parser/text2wikititle .
# RUN chmod 755 text2wikititle
RUN chmod u+x rewritebinary_linux && chmod u+x parserbinary_linux && chmod u+x nlp_linux

RUN ls -la
ENTRYPOINT ["./run.sh"]