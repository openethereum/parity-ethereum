import { toPromise, identity } from './';

describe('util/index', () => {
  describe('toPromise', () => {
    it('rejects on error result', () => {
      const ERROR = new Error();
      const FN = function (callback) {
        callback(ERROR);
      };

      return toPromise(FN).catch(err => {
        expect(err).to.equal(ERROR);
      });
    });

    it('resolves on success result', () => {
      const SUCCESS = 'ok, we are good';
      const FN = function (callback) {
        callback(null, SUCCESS);
      };

      return toPromise(FN).then(success => {
        expect(success).to.equal(SUCCESS);
      });
    });
  });

  describe('identity', () => {
    it('returns the value passed in', () => {
      const TEST = { abc: 'def' };

      expect(identity(TEST)).to.deep.equal(TEST);
    });
  });
});
