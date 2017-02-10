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

import fs from 'fs';
import path from 'path';

import i18nstrings from '../.build/i18n/i18n/en.json';

const FILE_HEADER = `// Copyright 2015-2017 Parity Technologies (UK) Ltd.
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
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.\n\n`;
const SECTION_HEADER = 'export default ';
const SECTION_FOOTER = ';\n';
const INDENT = '  ';
const I18NPATH = path.join(__dirname, '../src/i18n/_default');

// FIXME: Expanding it this way is probably not quite optimal, some would say hacky.
// However JSON.stringify (the sane solution) let us end up with both the keys
// and values with "'s which is not intended. We want keys without and values with `
// (Better idea welcome, at this point not critical since we control inputs)
function createExportString (section, indent) {
  if (Object.prototype.toString.call(section) === '[object String]') {
    return `\`${section}\``;
  }

  const keys = Object
    .keys(section)
    .sort()
    .map((key) => {
      return `${indent}${key}: ${createExportString(section[key], indent + INDENT)}`;
    })
    .join(',\n');

  return `{\n${keys}\n${indent.substr(2)}}`;
}

const sections = {};

// create an object map of the actual inputs
Object.keys(i18nstrings).forEach((fullKey) => {
  const defaultMessage = i18nstrings[fullKey].defaultMessage;
  const keys = fullKey.split('.');
  let outputs = sections;

  keys.forEach((key, index) => {
    if (index === keys.length - 1) {
      outputs[key] = defaultMessage;
    } else {
      if (!outputs[key]) {
        outputs[key] = {};
      }

      outputs = outputs[key];
    }
  });
});

// create the index.js file
const sectionKeys = Object.keys(sections).sort();
const exports = sectionKeys
  .map((key) => {
    return `export ${key} from './${key}';`;
  })
  .join('\n');

fs.writeFileSync(path.join(I18NPATH, 'index.js'), `${FILE_HEADER}${exports}\n`, 'utf8');

// create the individual section files
sectionKeys.forEach((key) => {
  // const sectionText = JSON.stringify(sections[key], null, 2);
  const sectionText = createExportString(sections[key], INDENT);

  fs.writeFileSync(path.join(I18NPATH, `${key}.js`), `${FILE_HEADER}${SECTION_HEADER}${sectionText}${SECTION_FOOTER}`, 'utf8');
});
