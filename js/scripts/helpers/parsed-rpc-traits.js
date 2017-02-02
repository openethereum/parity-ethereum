import fs from 'fs';
import path from 'path';

// ```js
// rustMethods['eth']['call'] === true
// ```
const rustMethods = {};

export default rustMethods;

// Get a list of JSON-RPC from Rust trait source code
function parseMethodsFromRust (source) {
  // Matching the custom `rpc` attribute with it's doc comment
  const attributePattern = /((?:\s*\/\/\/.*$)*)\s*#\[rpc\(([^)]+)\)]/gm;
  const commentPattern = /\s*\/\/\/\s*/g;
  const separatorPattern = /\s*,\s*/g;
  const assignPattern = /([\S]+)\s*=\s*"([^"]*)"/;
  const ignorePattern = /@(ignore|deprecated|unimplemented|alias)\b/i;

  const methods = [];

  source.toString().replace(attributePattern, (match, comment, props) => {
    comment = comment.replace(commentPattern, '\n').trim();

    // Skip deprecated methods
    if (ignorePattern.test(comment)) {
      return match;
    }

    props.split(separatorPattern).forEach((prop) => {
      const [, key, value] = prop.split(assignPattern) || [];

      if (key === 'name' && value != null) {
        methods.push(value);
      }
    });

    return match;
  });

  return methods;
}

// Get a list of all JSON-RPC methods from all defined traits
function getMethodsFromRustTraits () {
  const traitsDir = path.join(__dirname, '../../../rpc/src/v1/traits');

  return fs.readdirSync(traitsDir)
            .filter((name) => name !== 'mod.rs' && /\.rs$/.test(name))
            .map((name) => fs.readFileSync(path.join(traitsDir, name)))
            .map(parseMethodsFromRust)
            .reduce((a, b) => a.concat(b));
}

getMethodsFromRustTraits().sort().forEach((method) => {
  const [group, name] = method.split('_');

  // Skip methods with malformed names
  if (group == null || name == null) {
    return;
  }

  rustMethods[group] = rustMethods[group] || {};
  rustMethods[group][name] = true;
});
