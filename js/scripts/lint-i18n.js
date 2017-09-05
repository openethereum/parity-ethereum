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
const defaultValues = flatten(Object.assign({}, defaults, LANGUAGES));
const defaultKeys = Object.keys(defaultValues);
const results = {};

Object
  .keys(MESSAGES)
  .filter((lang) => !SKIP_LANG.includes(lang))
  .forEach((lang) => {
    const messageKeys = Object.keys(MESSAGES[lang]);
    const langResults = { found: [], missing: [], extras: [] };

    console.warn(`*** Checking translations for ${lang}`);

    defaultKeys.forEach((key) => {
      if (messageKeys.includes(key)) {
        langResults.found.push(key);
      } else {
        langResults.missing.push(key);
      }
    });

    messageKeys.forEach((key) => {
      if (!defaultKeys.includes(key)) {
        langResults.extras.push(key);
      }
    });

    // Sort keys
    langResults.extras.sort((kA, kB) => kA.localeCompare(kB));
    langResults.found.sort((kA, kB) => kA.localeCompare(kB));
    langResults.missing.sort((kA, kB) => kA.localeCompare(kB));

    // Print to stderr the missing and extra keys
    langResults.missing.forEach((key) => console.warn(`  Missing ${key}`));
    langResults.extras.forEach((key) => console.warn(`  Extra ${key}`));

    results[lang] = langResults;

    console.warn(`Found ${langResults.found.length} keys, missing ${langResults.missing.length} keys, ${langResults.extras.length} extraneous keys\n`);
  });

const formattedResults = Object.keys(results)
  .reduce((res, lang) => {
    const { missing } = results[lang];

    res[lang] = missing.map((key) => ({
      key,
      default: defaultValues[key]
    }));

    return res;
  }, {});

process.stdout.write(JSON.stringify(formattedResults, null, 2) + '\n');
