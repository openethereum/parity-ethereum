import nock from 'nock';
import { Server as MockWsServer } from 'mock-socket';

import { isFunction } from '../src/api/util/types';

export const TEST_HTTP_URL = 'http://localhost:6688';
export const TEST_WS_URL = 'ws://localhost:8866';

export function mockHttp (requests) {
  let scope = nock(TEST_HTTP_URL);

  requests.forEach((request) => {
    scope = scope
      .post('/')
      .reply(request.code || 200, (uri, body) => {
        if (body.method !== request.method) {
          return {
            error: `Invalid method ${body.method}, expected ${request.method}`
          };
        }

        scope.body = scope.body || {};
        scope.body[request.method] = body;

        return request.reply;
      });
  });

  return scope;
}

export function mockWs (requests) {
  const scope = { requests: 0, body: {} };
  let mockServer = new MockWsServer(TEST_WS_URL);

  scope.isDone = () => scope.requests === requests.length;
  scope.stop = () => {
    if (mockServer) {
      mockServer.stop();
      mockServer = null;
    }
  };

  mockServer.on('message', (_body) => {
    const body = JSON.parse(_body);
    const request = requests[scope.requests];
    const reply = request.reply;
    const response = reply.error
      ? { id: body.id, error: { code: reply.error.code, message: reply.error.message } }
      : { id: body.id, result: reply };

    scope.body[request.method] = body;
    scope.requests++;

    mockServer.send(JSON.stringify(response));
  });

  return scope;
}

export function endpointTest (instance, moduleId, name) {
  describe(name, () => {
    it(`has the ${moduleId}.${name} endpoint`, () => {
      expect(isFunction(instance[moduleId][name])).to.be.ok;
    });

    it(`maps to ${moduleId}_${name} via RPC`, () => {
      const scope = mockHttp([{ method: `${moduleId}_${name}`, reply: {} }]);

      return instance[moduleId][name]()
        .then(() => {
          expect(scope.isDone()).to.be.true;
        })
        .catch(() => {
          nock.cleanAll();
        });
    });
  });
}
