// Copyright 2015, 2016 Ethcore (UK) Ltd.
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

import interfaces from '../';

const MARKDOWN = path.join(__dirname, '../../release/interfaces.md');

let preamble = '# interfaces\n';
let markdown = '';

function formatDescription (obj, prefix = '', indent = '') {
  const optional = obj.optional ? '(optional) ' : '';
  const defaults = obj.default ? `(default: ${obj.default}) ` : '';

  return `${indent}- ${prefix}\`${obj.type.name}\` - ${optional}${defaults}${obj.desc}`;
}

function formatType (obj) {
  if (obj.type === Object && obj.details) {
    const sub = Object.keys(obj.details).sort().map((key) => {
      return formatDescription(obj.details[key], `\`${key}\`/`, '    ');
    }).join('\n');

    return `${formatDescription(obj)}\n${sub}`;
  } else if (obj.type && obj.type.name) {
    return formatDescription(obj);
  }

  return obj;
}

Object.keys(interfaces).sort().forEach((group) => {
  let content = '';

  preamble = `${preamble}\n- [${group}](#${group})`;
  markdown = `${markdown}\n## ${group}\n`;

  Object.keys(interfaces[group]).sort().forEach((iname) => {
    const method = interfaces[group][iname];
    const name = `${group}_${iname}`;
    const deprecated = method.deprecated ? ' (Deprecated and not supported, to be removed in a future version)' : '';
    const desc = `${method.desc}${deprecated}`;
    const params = method.params.map(formatType).join('\n');
    const returns = formatType(method.returns);

    markdown = `${markdown}\n- [${name}](#${name})`;
    content = `${content}### ${name}\n\n${desc}\n\n#### parameters\n\n${params || 'none'}\n\n#### returns\n\n${returns || 'none'}\n\n`;
  });

  markdown = `${markdown}\n\n${content}`;
});

fs.writeFileSync(MARKDOWN, `${preamble}\n\n${markdown}`, 'utf8');
