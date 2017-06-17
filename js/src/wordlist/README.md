# Wordlist

### Dictionary file of words for creation of secure keys.

The primary goal of this module is to provide an effective wordlist generator. Parity is known to use 12 words to ensure enough entropy when creating keys. Some functions were added for general purpose use.

---

## Usage

#### Require the module:
``` javascript

const {
  randomBytes,
  randomNumber,
  randomWord,
  randomPhrase
} = require('parity/wordlist');
```

#### Generate Random Bytes

`randomBytes`: Function(length: number)
* Output:
  - Buffer: with random input whose size is the inputs number *in bytes*

``` javascript
let result = randomBytes(10);
console.log(result);
// <Buffer e0 7d cd 33 db 30 4a 2b 9c cf>
```

#### Generate a Random Number

`randomNumber`: Function(max: number)
* Output:
  - number: between 0 and max

``` javascript
let result = randomNumber(1000);
console.log(result);
// 84
```

#### Generate a Random Word

`randomWord`: Function()
* Output
  - string: random word pulled from a pre-defined JSON list

``` javascript
let result = randomWord();
console.log(result);
// anchor
```

#### Generate a Random Phrase

`randomPhrase`: Function(size: number)
* Output
  - string: input size number of words, each separated by a space

``` javascript
let result = randomPhrase(12);
console.log(result);
// abdominal rice salami confess quickly jam umbrella freckles snub wildfowl grape roman
```
