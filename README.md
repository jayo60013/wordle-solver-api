# Wordle Solver API

This is an API for the wordle solver app.
Wordle word list obtained [here](https://gist.github.com/cfreshman/a7b776506c73284511034e63af1017ee)

## Endpoints

- `GET /all-words` - returns all the words in the official NYT wordle answers list.

- `POST /possible-words` - returns all possible words based on the constraints given by grey, yellow and green letters.

### Example Payload
```json
{
  "grey_letters": ["b", "l", "e", "u", "t", "o", "r", "i"],
  "yellow_letters": [
    ["d", 4],
    ["o", 0],
    ["d", 3],
    ["a", 1],
    ["d", 2],
    ["o", 4]
  ],
  "green_letters": []
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
- [ ] Distinguish between a possible guess and a possible answer
- [ ] Get best next guess
- [ ] Analyse the way the game was played
