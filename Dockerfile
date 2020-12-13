
FROM rust:1.48-buster as planner
RUN echo "PREPARE CARGO CHEF PLANNER"
WORKDIR /newsplatform/
ADD news_general ./news_general
ADD news_parser ./news_parser
ADD news_server ./news_server
ADD Cargo.toml .
ADD Cargo.lock .
RUN cargo install cargo-chef
RUN cargo chef prepare  --recipe-path recipe.json
RUN ls -la

FROM rust:1.48-buster as cacher
RUN echo "PREPARE CARGO CHEF CACHER"
WORKDIR /newsplatform/
RUN cargo install cargo-chef
COPY --from=planner /newsplatform/recipe.json recipe.json
RUN cat recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
ADD download_models.sh .
RUN chmod u+x download_models.sh && ./download_models.sh
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
WORKDIR /newsplatform/
ADD run.sh .
ADD news_server/templates ./templates
ADD news_server/postcss.config.js .
ADD news_server/tailwind.config.js .
ADD Config.toml .

COPY --from=builder /newsplatform/target/release/news_server .
COPY --from=builder /newsplatform/target/release/news_parser .
COPY --from=builder /newsplatform/news_parser/rewritebinary_linux .
COPY --from=builder /newsplatform/news_parser/parserbinary_linux .
COPY --from=cacher /newsplatform/models ./models

RUN ls -la
ENTRYPOINT ["./run.sh"]

# RUN chmod u+x parserbinary_linux && chmod u+x rewritebinary_linux
# RUN chmod u+x server && chmod u+x parser && chmod u+x run.sh

# RUN ls -la

# RUN mkdir src
# ADD newsserver/Cargo.toml .
# ADD newsserver/Cargo.lock .

# RUN chmod u+x parserbinary_linux && chmod u+x rewritebinary_linux && chmod u+x parserbinary_macos && chmod u+x rewritebinary_macos

# RUN cp target/release/newsserver newsserver
# RUN chmod u+x newsserver && chmod u+x run.sh

# RUN mkdir configs