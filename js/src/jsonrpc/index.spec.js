import interfaces from './';
import { Address, BlockNumber, Data, Hash, Integer, Quantity } from './types';

const flatlist = {};

function verifyType (obj) {
  if (typeof obj !== 'string') {
    expect(obj).to.satisfy(() => {
      return obj.type === Array ||
        obj.type === Boolean ||
        obj.type === Object ||
        obj.type === String ||
        obj.type === Address ||
        obj.type === BlockNumber ||
        obj.type === Data ||
        obj.type === Hash ||
        obj.type === Integer ||
        obj.type === Quantity;
    });
  }
}

describe('jsonrpc/interfaces', () => {
  Object.keys(interfaces).forEach((group) => {
    describe(group, () => {
      Object.keys(interfaces[group]).forEach((name) => {
        const method = interfaces[group][name];

        flatlist[`${group}_${name}`] = true;

        describe(name, () => {
          it('has the correct interface', () => {
            expect(method.desc).to.be.a('string');
            expect(method.params).to.be.an('array');
            expect(method.returns).to.satisfy((returns) => {
              return typeof returns === 'string' || typeof returns === 'object';
            });

            method.params.forEach(verifyType);
            verifyType(method.returns);
          });
        });
      });
    });
  });
});
