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

const fs = require('fs');
const _ = require('lodash');
const path = require('path');

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
const ENPATH = path.join(__dirname, '../src/i18n/en');
const SRCPATH = path.join(__dirname, '../.build/i18n/i18n/en.json');

// main entry point
(function main () {
  const { sections, sectionNames } = createSectionMap();

  sectionNames.forEach((name) => outputSection(name, sections[name]));
  outputIndex(sectionNames);
})();

// helper to merge in section into defaults
function deepMerge (object, source) {
  return _.mergeWith(object, source, (objValue, srcValue) => {
    if (_.isObject(objValue) && srcValue) {
      return deepMerge(objValue, srcValue);
    }
  });
}

// export a section as a flatenned string (non-JSON, rather JS export)
function createExportString (section, indent = INDENT) {
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

// load the available deafults (non-exported strings) for a section
function readDefaults (sectionName) {
  let defaults = {};

  try {
    defaults = require(path.join(ENPATH, `${sectionName}.js`)).default;
  } catch (error) {
    defaults = {};
  }

  return defaults;
}

// create the index.js file
function outputIndex (sectionNames) {
  console.log(`Writing index.js to ${DESTPATH}`);

  const defaults = readDefaults('index');
  const dest = path.join(DESTPATH, 'index.js');
  const exports = _.uniq(Object.keys(defaults).concat(sectionNames))
    .sort()
    .map((name) => `export ${name} from './${name}';`)
    .join('\n');

  fs.writeFileSync(dest, `${FILE_HEADER}${exports}\n`, 'utf8');
}

// create the individual section files
function outputSection (sectionName, _section) {
  console.log(`Writing ${sectionName}.js to ${DESTPATH}`);

  const defaults = readDefaults(sectionName);
  const section = deepMerge(defaults, _section);
  const dest = path.join(DESTPATH, `${sectionName}.js`);
  const sectionText = createExportString(section, INDENT);

  fs.writeFileSync(dest, `${FILE_HEADER}${SECTION_HEADER}${sectionText}${SECTION_FOOTER}`, 'utf8');
}
