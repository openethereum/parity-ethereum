import etherscan from './';

describe('etherscan/stats', () => {
  it('retrieves the latest price', () => {
    return etherscan.stats
      .price()
      .then((price) => {
        expect(price).to.be.ok;
      });
  });

  it('retrieves the ether total', () => {
    return etherscan.stats
      .supply()
      .then((supply) => {
        expect(supply).to.be.ok;
      });
  });
});
