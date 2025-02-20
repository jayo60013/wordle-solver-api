FROM rust:1.83.0-bookworm AS build

WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
WORKDIR /app
COPY --from=build /app/target/release/wordle_solver /app/wordle_solver
COPY word_list.txt /app/word_list.txt
# CMD ["/app/wordle_solver"]
CMD ["sh", "-c", "ls -lah /app && /app/wordle_solver"]
