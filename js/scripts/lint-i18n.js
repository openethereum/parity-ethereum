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

import flatten from 'flat';

import * as defaults from '../src/i18n/_default';
import { MESSAGES } from '../src/i18n/store';

const SKIP_LANG = ['en'];
const DEFAULTS = flatten(defaults);

Object
  .keys(MESSAGES)
  .filter((lang) => !SKIP_LANG.includes(lang))
  .forEach((lang) => {
    const messages = MESSAGES[lang];
    let found = 0;
    let missing = 0;
    let total = 0;

    console.log(`*** Checking translations for ${lang}`);

    Object
      .keys(DEFAULTS)
      .forEach((key) => {
        total++;

        if (messages[key]) {
          found++;
        } else {
          missing++;
          console.log(`  Missing ${key}`);
        }
      });

    console.log(`Checked ${total}, found ${found} keys, missing ${missing} keys\n`);
  });
