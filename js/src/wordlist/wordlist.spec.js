import { expect } from 'chai';

import { randomBytes, randomNumber, randomWord, randomPhrase } from './';

describe('wordlist', () => {

  describe('Random Bytes', () => {
    it('creates random bytes of specified length', () => {
      const bytes = randomBytes(12);
      expect(bytes.length).to.equal(12);
    });
  });

  describe('Random number', () => {
    it('creates random number', () => {
      const number = randomNumber();
      expect(typeof number === 'number').to.be.true;
    });
  });

  describe('Random word', () => {
    it('creates random bytes', () => {
      const word = randomWord();
      expect(typeof word === 'string').to.be.true;
    });
  });

  describe('Random phrase', () => {
    const phrase = randomPhrase(12).split(' ');
    it('creates random phrase of specified length', () => {
      expect(phrase.length).to.equal(12);
    });
    it('creates random phrase that was converted to array', () => {
      expect(Array.isArray(phrase)).to.be.true;
    });
  });

});
