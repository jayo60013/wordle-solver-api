# AGENTS.md - Wordle Solver API

## Project Overview

**Wordle Solver API** is a Rust-based REST API that filters possible Wordle words based on game constraints (letter colors) and ranks them by information-theoretic entropy. It serves both official Wordle answers and valid guesses, providing clients with optimal word suggestions.

- **Framework**: Actix-web (Rust web framework)
- **Key Dependencies**: Rayon (parallel processing), itertools, serde (JSON serialization)
- **Data Files**: `wordle-nyt-answers.txt` (2309 words) and `wordle-nyt-allowed-guesses.txt` (valid guesses)
- **Port**: 5307

## Critical Architecture Patterns

### 1. Global Word List Initialization
- Uses `OnceLock<Vec<Word>>` (thread-safe lazy static) loaded once at startup in `main()` via `get_all_words_from_file()`
- Combines answers and allowed guesses into single vector for filtering/scoring
- **Why**: Avoid repeated file I/O and support immutable shared access across async request handlers
- **When modifying**: Changes to word loading logic affect all endpoints; test with sample word lists first

### 2. Request Pipeline: Filter → Entropy Scoring → Response
The `/possible-words` endpoint follows this exact flow:
1. **Rate Limit Check** (`rate_limit::IpRateLimiter`): Validates IP hasn't exceeded 10 req/sec quota
2. **Cache Check** (`EMPTY_BODY_CACHE`): Returns cached result for empty request bodies (all words)
3. **Filter** (`filters::filter_words_by_guesses`): Parallel filtering using Rayon to match letter constraints
4. **Score** (`entropy::calculate_entropy_for_words`): Ranks results by entropy (information gain), sorted descending
5. **Cache Store** (if empty body): Stores response for future empty requests
6. **Serialize** (`models.rs`): JSON response with word list, counts, and entropy bounds

**Example request payload** (see README):
```json
[{"turn": 0, "letter": "o", "position": 2, "color": "YELLOW"}, ...]
```

### 3. Rate Limiting (New: Production Safety)
`rate_limit.rs` implements token bucket algorithm per IP address:
- **Quota**: 10 requests per second per client IP
- **Implementation**: `IpRateLimiter` tracks token buckets in `Arc<Mutex<HashMap<IpAddr, TokenBucket>>>`
- **Response**: Returns 429 Too Many Requests if exceeded
- **Why**: Protects against spam; entropy calculation is computationally expensive (O(n × 243 × m))
- **Configuration**: Change quota in `main.rs` line ~127: `IpRateLimiter::new(10, 1.0)` where first param is requests/sec

### 4. Empty Body Response Caching (New: Performance)
`EMPTY_BODY_CACHE` stores the JSON response for requests with zero guesses:
- **Trigger**: When `guesses.is_empty()` is true
- **Benefit**: Eliminates full word list filtering + entropy calculation on repeated "all words" queries
- **Implementation**: `OnceLock<Vec<u8>>` (immutable after first set)
- **Invalidation**: None; cache is valid for app lifetime (static word list)
- **Impact**: Reduces expensive O(n × 243 × m) operations for common queries

### 5. Constraint Matching Logic (Complex)
`filter_word_by_guesses()` implements Wordle-specific rules:
- **Green (exact match)**: `word_chars[position] == letter`
- **Yellow (exists elsewhere)**: Letter count ≥ expected from turn, NOT at this position
- **Grey (excluded)**: Letter count == expected (allows for letter reuse if already green/yellow), NOT at this position

**Critical Detail**: `get_expected_total_of_letters()` counts green+yellow occurrences per turn to handle multi-occurrence letters (e.g., two 'e's). This determines if a grey 'e' means "no e's exist" or "no additional e's beyond those marked green/yellow."

**Tests verify**: Double letters, grey blocking, yellow constraints—check `filters.rs` test suite before modifying logic.

### 6. Entropy Calculation (Computationally Intensive)
`entropy.rs` uses **information theory**: entropy = Σ P(outcome) × log₂(1/P(outcome))
- Generates all 3^5 = 243 possible color permutations for each word
- Filters word list for each permutation to compute probability
- Applies **repeat penalty** (0.5× reduction per duplicate letter) to favor diverse letters
- **Parallelized via Rayon** at word level, not permutation level

**Performance note**: This is O(n × 243 × m) where n=words, m=remaining candidates. Acceptable for 2300 words but will be bottleneck for larger corpora.

## Developer Workflows

### Build & Run
```bash
# Local development
cargo build
cargo run

# Release build (also used in Docker)
cargo build --release

# Run tests (test cases in filters.rs)
cargo test
```

### Testing
- Only `filters.rs` has unit tests—covers constraint matching edge cases
- **Add tests** when modifying constraint logic or entropy calculation
- Run tests before deployment: `cargo test --release`

### Docker Deployment
Multi-stage build (Dockerfile):
1. Builder stage: Rust 1.83.0, compiles to release binary
2. Runtime stage: `distroless/cc-debian12` (minimal, no shell)
3. **Critical files must be copied**: Both `wordle-nyt-*.txt` files required at runtime

### Debugging
- Environment variable `RUST_LOG=actix_web=info,wordle_solver=info` controls logging
- Request logging via `Logger::default()` middleware
- Entropy calculation has no logging—add via `log::info!()` in entropy.rs if needed

## Project Conventions & Patterns

### Module Organization
- `models.rs`: Data structures (Word, Guess, Color enum, PossibleWords response)
- `filters.rs`: Word filtering logic + tests
- `entropy.rs`: Entropy calculation + ranking
- `rate_limit.rs`: IP-based rate limiting with token bucket algorithm + tests
- `main.rs`: Actix setup, endpoints, word list loading, caching

### Error Handling
- `main()` returns `io::Result<()>` for server startup errors
- Endpoint handlers use simple Option matching: graceful 500 on word list uninitialized
- No custom error types—leverage Actix default error responses

### Parallelism
- `par_iter()` from Rayon used in: word filtering (filters.rs), entropy calculation (entropy.rs)
- **Not used in**: Entropy permutation loop (itertools cartesian product) or entropy scoring per permutation
- **Opportunity**: Permutation loop could be parallelized further

### Data Flow & Boundaries
```
HTTP Request (Vec<Guess>)
    ↓
filter_words_by_guesses() [filters.rs] → partial word list
    ↓
calculate_entropy_for_words() [entropy.rs] → scored & sorted
    ↓
aggregate stats (min/max entropy, counts) [main.rs]
    ↓
JSON response (PossibleWords)
```

### Type Design
- **Word**: Contains `word: String`, `entropy: f32`, `is_answer: bool` (distinguishes answers from valid guesses)
- **Guess**: Deserialized from request; contains `letter: char`, `position: usize`, `color: Color` (enum), `turn: usize`
- **Color**: Enum (Grey, Yellow, Green)—no Display impl, must pattern match
- All types use `#[derive(Serialize, Deserialize)]` for actix JSON handling

### CORS Configuration
Hardcoded to accept requests from `http://localhost:5173` (assumed frontend dev server). Allowed methods: GET, POST, OPTIONS.
- **To change frontend URL**: Modify Cors builder in main.rs line ~96

### Word List Lifecycle
1. Load: Both `.txt` files read sequentially in `read_words_from_file()` 
2. Merge: Answers marked `is_answer: true`, guesses marked `is_answer: false`
3. Store: Global `WORD_LIST` OnceLock
4. Access: Immutable borrow via `.get()` in request handlers
5. No mutations after init—thread-safe by design

## Integration Points & External Dependencies

- **Actix-web 4**: HTTP server; handles routing, middleware, async runtime
- **Actix-cors 0.7**: CORS middleware (hardcoded frontend origin)
- **Rayon 1.10**: Data parallelism for filtering & entropy
- **Itertools 0.14**: `multi_cartesian_product()` for color permutations
- **Serde 1.0 + Serde_json 1.0**: JSON serialization; derives on models.rs
- **env_logger 0.11.6**: Logging; filters by RUST_LOG env var
- **Distroless base image**: Runtime doesn't have shell/utilities

## Common Tasks & Where to Find Them

| Task | Location | Key Functions |
|------|----------|---|
| Add a new endpoint | main.rs | Define `#[post]` or `#[get]` handler |
| Modify word filtering logic | filters.rs:8-40 | `filter_word_by_guesses()` |
| Adjust entropy weighting | entropy.rs:22-45 | `calculate_entropy_for_word()`, line 27 repeat_penalty |
| Change rate limit quota | main.rs:127 | `IpRateLimiter::new(10, 1.0)` where 10=requests/sec |
| Change allowed frontend | main.rs:142 | `Cors::default().allowed_origin()` |
| Add request logging | entropy.rs | Use `log::info!()` in calculation loop |
| Handle new Guess field | models.rs:20-27 | Update Guess struct & filter logic |

## Known Limitations & Future Work

- Entropy calculation not cached—recomputed for every request (roadmap item: "Get best next guess")
- No pagination for large result sets
- Hardcoded word file paths (relative to CWD)
- CORS origin hardcoded (not environment-configurable)
- No input validation on Guess struct (assumes valid turn/position/letter)

