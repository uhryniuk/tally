FROM rust:latest

# build and put binary in path
WORKDIR /usr/src/tally
COPY . .
RUN cargo build --release
RUN cp target/release/tally /usr/local/bin/tally

# users can test tally from here
ENTRYPOINT ["sh"]
