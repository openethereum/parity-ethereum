// Copyright 2015-2017 Parity Technologies (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

import dictionary from './dictionary';
import {
  phraseToAddress,
  phraseToWallet,
  randomNumber,
  randomWord,
  randomPhrase
} from './';

describe('api/local/ethkey', () => {
  describe('randomNumber', () => {
    it('generates numbers in range', () => {
      for (let i = 0; i < 100; i++) {
        const number = randomNumber(7777);

        expect(number).to.be.at.least(0);
        expect(number).to.be.below(7777);
        expect(number % 1).to.be.equal(0);
      }
    });
  });

  describe('randomWord', () => {
    it('generates a random word from the dictionary', () => {
      const word = randomWord();

      expect(dictionary.includes(word)).to.be.equal(true);
    });
  });

  describe('randomPhrase', () => {
    it('generates a random phrase from the dictionary', () => {
      const phrase = randomPhrase(7).split(' ');

      expect(phrase.length).to.be.equal(7);

      phrase.forEach((word) => {
        expect(dictionary.includes(word)).to.be.equal(true);
      });
    });
  });

  describe('phraseToAddress', function () {
    this.timeout(10000);

    it('generates a valid address', () => {
      const phrase = randomPhrase(12);

      return phraseToAddress(phrase).then((address) => {
        expect(address.length).to.be.equal(42);
        expect(address.slice(0, 4)).to.be.equal('0x00');
      });
    });

    it('generates valid address for empty phrase', () => {
      return phraseToAddress('').then((address) => {
        expect(address).to.be.equal('0x00a329c0648769a73afac7f9381e08fb43dbea72');
      });
    });
  });

  describe('phraseToWallet', function () {
    this.timeout(10000);

    it('generates a valid wallet object', () => {
      const phrase = randomPhrase(12);

      return phraseToWallet(phrase).then((wallet) => {
        expect(wallet.address.length).to.be.equal(42);
        expect(wallet.secret.length).to.be.equal(66);
        expect(wallet.public.length).to.be.equal(130);

        expect(wallet.address.slice(0, 4)).to.be.equal('0x00');
        expect(wallet.secret.slice(0, 2)).to.be.equal('0x');
        expect(wallet.public.slice(0, 2)).to.be.equal('0x');
      });
    });
  });
});
