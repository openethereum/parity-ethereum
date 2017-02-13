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
const DESTPATH = path.join(__dirname, '../src/i18n/_default');
const SRCPATH = path.join(__dirname, '../.build/i18n/i18n/en.json');

// main entry point
(function main () {
  const { sections, sectionNames } = createSectionMap();

  sectionNames.forEach((name) => outputSection(name, sections[name]));
  outputIndex(sectionNames);
})();

// export a section as a flatenned string (non-JSON, rather JS export)
function createExportString (section, indent) {
  if (typeof section === 'string') {
    return `\`${section}\``;
  }

  const keys = Object
    .keys(section)
    .sort()
    .map((key) => `${indent}${key}: ${createExportString(section[key], indent + INDENT)}`)
    .join(',\n');

  return `{\n${keys}\n${indent.substr(2)}}`;
}

// create an object map of the actual inputs
function createSectionMap () {
  console.log(`Reading strings from ${SRCPATH}`);

  const i18nstrings = require(SRCPATH);
  const sections = Object
    .keys(i18nstrings)
    .reduce((sections, fullKey) => {
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

      return sections;
    }, {});
  const sectionNames = Object.keys(sections).sort();

  console.log(`Found ${sectionNames.length} sections`);

  return {
    sections,
    sectionNames
  };
}

// create the index.js file
function outputIndex (sectionNames) {
  console.log(`Writing index.js to ${DESTPATH}`);

  const dest = path.join(DESTPATH, 'index.js');
  const exports = sectionNames
    .map((name) => `export ${name} from './${name}';`)
    .join('\n');

  fs.writeFileSync(dest, `${FILE_HEADER}${exports}\n`, 'utf8');
}

// create the individual section files
function outputSection (name, section) {
  console.log(`Writing ${name}.js to ${DESTPATH}`);

  const dest = path.join(DESTPATH, `${name}.js`);
  const sectionText = createExportString(section, INDENT);

  fs.writeFileSync(dest, `${FILE_HEADER}${SECTION_HEADER}${sectionText}${SECTION_FOOTER}`, 'utf8');
}
