# Wordle Solver API

This is an API for the wordle solver app.
Wordle word list obtained [here](https://gist.github.com/cfreshman/a7b776506c73284511034e63af1017ee)

## Endpoints

- `GET /all-words` - returns all the words in the official NYT wordle answers list.

- `POST /possible-words` - returns all possible words based on the constraints given by grey, yellow and green letters.

### Example Payload

```json
{
    [
        {"turn": 0, "letter": "b", "position": 0, "color": "GREY"},
        {"turn": 0, "letter": "l", "position": 1, "color": "GREY"},
        {"turn": 0, "letter": "o", "position": 2, "color": "YELLOW"},
        {"turn": 0, "letter": "o", "position": 3, "color": "GREEN"},
        {"turn": 0, "letter": "d", "position": 4, "color": "GREY"}
    ]
}
```

### Example Response

```json
{
  "word_list": [
    {
      "word": "dogma",
      "entropy": 0.23473366
    }
  ],
  "number_of_words": 1,
  "total_number_of_words": 2309
}
```

## Road Map

- [x] Get possible words based off letter constraints
- [x] Distinguish between a possible guess and a possible answer
- [ ] Get best next guess
- [ ] Analyse the way the game was played
