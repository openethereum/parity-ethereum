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
import { isPlainObject } from 'lodash';

import { info, warn, error } from './helpers/log';
import { Dummy } from '../src/jsonrpc/helpers';
import interfaces from '../src/jsonrpc';
import rustMethods from './helpers/parsed-rpc-traits';

const ROOT_DIR = path.join(__dirname, '../docs');

if (!fs.existsSync(ROOT_DIR)) {
  fs.mkdirSync(ROOT_DIR);
}

Object.keys(rustMethods).forEach((group) => {
  Object.keys(rustMethods[group]).forEach((method) => {
    if (interfaces[group] == null || interfaces[group][method] == null) {
      error(`${group}_${method} is defined in Rust traits, but not in js/src/jsonrpc/interfaces`);
    }
  });
});

function printType (type) {
  return type.print || `\`${type.name}\``;
}

function formatDescription (obj, prefix = '', indent = '') {
  const optional = obj.optional ? '(optional) ' : '';
  const defaults = obj.default ? `(default: \`${obj.default}\`) ` : '';

  return `${indent}${prefix}${printType(obj.type)} - ${optional}${defaults}${obj.desc}`;
}

function formatType (obj) {
  if (obj == null || obj.type == null) {
    return obj;
  }

  const details = obj.details || obj.type.details;

  if (details) {
    const sub = Object.keys(details).map((key) => {
      return formatDescription(details[key], `\`${key}\`: `, '    - ');
    }).join('\n');

    return `${formatDescription(obj)}\n${sub}`;
  } else if (obj.type && obj.type.name) {
    return formatDescription(obj);
  }

  return obj;
}

const rpcReqTemplate = {
  method: 'web3_clientVersion',
  params: [],
  id: 1,
  jsonrpc: '2.0'
};

const { isDummy } = Dummy;
const { isArray } = Array;

// Checks if a field definition has an example,
// or describes an object with fields that recursively have examples of their own,
// or is optional.
function hasExample ({ optional, example, details } = {}) {
  if (optional || example !== undefined) {
    return true;
  }

  if (details !== undefined) {
    const values = Object.keys(details).map((key) => details[key]);

    return values.every(hasExample);
  }

  return false;
}

// Remove all optional (trailing) params without examples from an array
function removeOptionalWithoutExamples (arr) {
  return arr.filter(({ optional, example, details }) => {
    return !optional || example !== undefined || details !== undefined;
  });
}

// Grabs JSON compatible
function getExample (obj) {
  if (isArray(obj)) {
    return removeOptionalWithoutExamples(obj).map(getExample);
  }

  const { example, details } = obj;

  if (example === undefined && details !== undefined) {
    const nested = {};

    Object.keys(details).forEach((key) => {
      nested[key] = getExample(details[key]);
    });

    return nested;
  }

  return example;
}

function stringifyExample (example, dent = '') {
  const indent = `${dent}  `;

  if (isDummy(example)) {
    return example.toString();
  }

  if (isArray(example)) {
    const last = example.length - 1;

    // If all elements are dummies, print out a single line.
    // Also covers empty arrays.
    if (example.every(isDummy)) {
      const dummies = example.map(d => d.toString());

      return `[${dummies.join(', ')}]`;
    }

    // For arrays containing just one object or string, don't unwind the array to multiline
    if (last === 0 && (isPlainObject(example[0]) || typeof example[0] === 'string')) {
      return `[${stringifyExample(example[0], dent)}]`;
    }

    const elements = example.map((value, index) => {
      const comma = index !== last ? ',' : '';
      const comment = value != null && value._comment ? ` // ${value._comment}` : '';

      return `${stringifyExample(value, indent)}${comma}${comment}`;
    });

    return `[\n${indent}${elements.join(`\n${indent}`)}\n${dent}]`;
  }

  if (isPlainObject(example)) {
    const keys = Object.keys(example);
    const last = keys.length - 1;

    // print out an empty object
    if (last === -1) {
      return '{}';
    }

    const elements = keys.map((key, index) => {
      const value = example[key];
      const comma = index !== last ? ',' : '';
      const comment = value && value._comment ? ` // ${example[key]._comment}` : '';

      return `${JSON.stringify(key)}: ${stringifyExample(value, indent)}${comma}${comment}`;
    });

    return `{\n${indent}${elements.join(`\n${indent}`)}\n${dent}}`;
  }

  return JSON.stringify(example);
}

function buildExample (name, method) {
  // deprecated, don't care
  if (method.deprecated) {
    return '';
  }

  const logPostfix = method.subdoc ? ` (${method.subdoc})` : '';

  const hasReqExample = method.params.every(hasExample);
  const hasResExample = hasExample(method.returns);

  if (!hasReqExample && !hasResExample) {
    error(`${name} has no examples${logPostfix}`);

    return '';
  }

  const examples = [];

  if (hasReqExample) {
    const params = getExample(method.params);
    const req = Dummy.stringifyJSON(Object.assign({}, rpcReqTemplate, { method: name, params }));

    examples.push(`Request\n\`\`\`bash\ncurl --data '${req}' -H "Content-Type: application/json" -X POST localhost:8545\n\`\`\``);
  } else {
    warn(`${name} has a response example but not a request example${logPostfix}`);
  }

  if (hasResExample) {
    const res = stringifyExample({
      id: 1,
      jsonrpc: '2.0',
      result: getExample(method.returns)
    });

    examples.push(`Response\n\`\`\`js\n${res}\n\`\`\``);
  } else {
    if (typeof method.returns === 'string') {
      info(`${name} has a request example and only text description for response${logPostfix}`);
    } else {
      warn(`${name} has a request example but not a response example${logPostfix}`);
    }
  }

  return `\n\n#### Example\n\n${examples.join('\n\n')}`;
}

function buildParameters (params) {
  if (params.length === 0) {
    return '';
  }

  let md = `0. ${params.map(formatType).join('\n0. ')}`;

  if (params.length > 0 && params.every(hasExample) && !isDummy(params[0].example)) {
    const example = getExample(params);

    md = `${md}\n\n\`\`\`js\nparams: ${stringifyExample(example)}\n\`\`\``;
  }

  return md;
}

Object.keys(interfaces).sort().forEach((group) => {
  const spec = interfaces[group];

  for (const key in spec) {
    const method = spec[key];

    if (!method || !method.subdoc) {
      continue;
    }

    const subgroup = `${group}_${method.subdoc}`;

    interfaces[subgroup] = interfaces[subgroup] || {};

    interfaces[subgroup][key] = method;
    delete spec[key];
  }
});

Object.keys(interfaces).sort().forEach((group) => {
  let preamble = `# The \`${group}\` Module`;
  let markdown = `## JSON-RPC methods\n`;

  const spec = interfaces[group];

  if (spec._preamble) {
    preamble = `${preamble}\n\n${spec._preamble}`;
  }

  const content = [];
  const tocMain = [];
  const tocSections = {};

  // Comparator that will sort by sections first, names second
  function methodComparator (a, b) {
    const sectionA = spec[a].section || '';
    const sectionB = spec[b].section || '';

    return sectionA.localeCompare(sectionB) || a.localeCompare(b);
  }

  Object.keys(spec).sort(methodComparator).forEach((iname) => {
    const method = spec[iname];
    const groupName = group.replace(/_.*$/, '');
    const name = `${groupName}_${iname}`;

    if (method.nodoc || method.deprecated) {
      info(`Skipping ${name}: ${method.nodoc || 'Deprecated'}`);

      return;
    }

    if (rustMethods[groupName] == null || rustMethods[groupName][iname] == null) {
      error(`${name} is defined in js/src/jsonrpc/interfaces, but not in Rust traits`);
    }

    const desc = method.desc;
    const params = buildParameters(method.params);
    const returns = `- ${formatType(method.returns)}`;
    const example = buildExample(name, method);

    const { section } = method;
    const toc = section ? tocSections[section] = tocSections[section] || [] : tocMain;

    toc.push(`- [${name}](#${name.toLowerCase()})`);
    content.push(`### ${name}\n\n${desc}\n\n#### Parameters\n\n${params || 'None'}\n\n#### Returns\n\n${returns || 'None'}${example}`);
  });

  markdown = `${markdown}\n${tocMain.join('\n')}`;

  Object.keys(tocSections).sort().forEach((section) => {
    markdown = `${markdown}\n\n#### ${section}\n${tocSections[section].join('\n')}`;
  });

  markdown = `${markdown}\n\n## JSON-RPC API Reference\n\n${content.join('\n\n***\n\n')}\n\n`;

  const mdFile = path.join(ROOT_DIR, `${group}.md`);

  fs.writeFileSync(mdFile, `${preamble}\n\n${markdown}`, 'utf8');
});
