const dictionary = require('./wordlist.json');

function isDefined (str) {
  return str !== 'undefined'
}

// Adapted from https://github.com/tonyg/js-scrypt
// js-scrypt is written by Tony Garnock-Jones tonygarnockjones@gmail.com and is licensed under the 2-clause BSD license:
function secureRandomBytes (count) {
  if (!isDefined(typeof Uint8Array)) {
    return null
  }

  const bs = new Uint8Array(count)
  const self = isDefined(typeof window) ? window : isDefined(typeof global) ? global : this

  if (isDefined(typeof self.crypto)) {
    if (isDefined(typeof self.crypto.getRandomValues)) {
      self.crypto.getRandomValues(bs)
      return bs
    }
  }

  if (isDefined(typeof self.msCrypto)) {
    if (isDefined(typeof self.msCrypto.getRandomValues)) {
      self.msCrypto.getRandomValues(bs)
      return bs
    }
  }

  return null
}

function randomBytes (length) {
  const random = secureRandomBytes(length)

  if (random) {
    return random
  }

  // Fallback if secure randomness is not available
  const buf = isDefined(typeof Buffer) ? Buffer.alloc(length) : Array(length)

  for (let i = 0; i < length; i++) {
    buf[i] = Math.random() * 255
  }

  return buf
}

function randomNumber (max) {
  // Use 24 bits to avoid the integer becoming signed via bitshifts
  const rand = randomBytes(3)

  const integer = (rand[0] << 16) | (rand[1] << 8) | rand[2]

  // floor to integer value via bitor 0
  return ((integer / 0xFFFFFF) * max) | 0
}

function randomWord () {
  // TODO mh: use better entropy
  const index = randomNumber(dictionary.length)

  return dictionary[index]
}

function randomPhrase (length) {
  const words = []

  while (length--) {
    words.push(randomWord())
  }

  return words.join(' ')
}

module.exports = {
  randomBytes,
  randomNumber,
  randomWord,
  randomPhrase
}
