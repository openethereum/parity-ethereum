/* eslint-env mocha */
/**
 * Important: This test also serves as a point to
 * import the entire lib for coverage reporting
 */

import { assert } from 'chai';
import * as DAppsUI from './';

describe('Dapps UI', () => it('should have exports', () => assert.ok(DAppsUI)));
