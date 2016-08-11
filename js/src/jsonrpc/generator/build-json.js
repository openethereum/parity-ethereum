import fs from 'fs';
import path from 'path';

import interfaces from '../';

const INDEX_JSON = path.join(__dirname, '../../release/index.json');
const methods = [];

function formatDescription (obj) {
  const optional = obj.optional ? '(optional) ' : '';
  const defaults = obj.default ? `(default: ${obj.default}) ` : '';

  return `${obj.type.name} - ${optional}${defaults}${obj.desc}`;
}

function formatType (obj) {
  if (obj.type === Object && obj.details) {
    const formatted = {};

    Object.keys(obj.details).sort().forEach((key) => {
      formatted[key] = formatType(obj.details[key]);
    });

    return {
      desc: formatDescription(obj),
      details: formatted
    };
  } else if (obj.type && obj.type.name) {
    return formatDescription(obj);
  }

  return obj;
}

Object.keys(interfaces).sort().forEach((group) => {
  Object.keys(interfaces[group]).sort().forEach((name) => {
    const method = interfaces[group][name];
    const deprecated = method.deprecated ? ' (Deprecated and not supported, to be removed in a future version)' : '';

    methods.push({
      name: `${group}_${name}`,
      desc: `${method.desc}${deprecated}`,
      params: method.params.map(formatType),
      returns: formatType(method.returns),
      inputFormatters: method.params.map((param) => param.format || null),
      outputFormatter: method.returns.format || null
    });
  });
});

fs.writeFileSync(INDEX_JSON, JSON.stringify({ methods: methods }, null, 2), 'utf8');
