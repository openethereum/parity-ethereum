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
