# Wordle Solver API

This is an API for the wordle solver app.
Wordle word list obtained [here](https://gist.github.com/dracos/dd0668f281e685bad51479e5acaadb93)

## Endpoints

- `GET /all-words` - returns all the words in the official wordle word list.

- `POST /possible-words` - returns all possible words based on the constraints given by grey, yellow and green letters.

### Example Payload
```json
{
  "incorrectLetters": [
    "a",
    "b",
    "c"
  ],
  "yellowLetters": [
    { "letter": "d", "position": 1 },
    { "letter": "e", "position": 1 }
  ],
  "greenLetters": [
    { "letter": "f", "position": 2 }
  ]
}
```

## Road Map
- [x] Get possible words based off letter constraints
- [ ] Get best next guess
- [ ] Analyse the way the game was played
