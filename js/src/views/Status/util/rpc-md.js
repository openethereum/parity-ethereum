
import { isPlainObject } from 'lodash';

export function formatRpcMd (val) {
  if (!isPlainObject(val)) {
    return val;
  }

  return val.description + Object.keys(val.details)
                            .map(key => `- \`${key}\`: ${val.details[key]}`)
                            .join('\n');
}
