FROM rust:1.83.0-bookworm AS build

WORKDIR /app
COPY . .
RUN cargo build --release

FROM gcr.io/distroless/cc-debian12
WORKDIR /app
COPY --from=build /app/target/release/wordle_solver /app/wordle_solver
COPY wordle-nyt-allowed-guesses.txt /app/wordle-nyt-allowed-guesses.txt
CMD ["/app/wordle_solver"]
