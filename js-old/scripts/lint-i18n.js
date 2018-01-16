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
import { LANGUAGES, MESSAGES } from '../src/i18n/store';

const SKIP_LANG = ['en'];
const defaultKeys = Object.keys(flatten(Object.assign({}, defaults, LANGUAGES)));

Object
  .keys(MESSAGES)
  .filter((lang) => !SKIP_LANG.includes(lang))
  .forEach((lang) => {
    const messageKeys = Object.keys(MESSAGES[lang]);
    let extra = 0;
    let found = 0;
    let missing = 0;

    console.log(`*** Checking translations for ${lang}`);

    defaultKeys.forEach((key) => {
      if (messageKeys.includes(key)) {
        found++;
      } else {
        missing++;
        console.log(`  Missing ${key}`);
      }
    });

    messageKeys.forEach((key) => {
      if (!defaultKeys.includes(key)) {
        extra++;
        console.log(`  Extra ${key}`);
      }
    });

    console.log(`Found ${found} keys, missing ${missing} keys, ${extra} extraneous keys\n`);
  });
