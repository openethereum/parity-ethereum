import etherscan from './';

const TESTADDR = '0xbf885e2b55c6bcc84556a3c5f07d3040833c8d00';

describe('etherscan/account', () => {
  const checkBalance = function (balance, addr) {
    expect(balance).to.be.ok;
    expect(balance.account).to.equal(addr);
    expect(balance.balance).to.be.ok;
  };

  it('retrieves an account balance', () => {
    return etherscan.account
      .balance(TESTADDR)
      .then((balance) => {
        checkBalance(balance, TESTADDR);
      });
  });

  it('retrieves multi account balances', () => {
    const addresses = ['0xde0b295669a9fd93d5f28d9ec85e40f4cb697bae', TESTADDR];

    return etherscan.account
      .balances(addresses)
      .then((balances) => {
        expect(balances).to.be.ok;
        expect(balances.length).to.equal(2);
        balances.forEach((balance, idx) => {
          checkBalance(balance, addresses[idx]);
        });
      });
  });

  describe('transactions', () => {
    it('retrievs a list of transactions (default)', () => {
      return etherscan.account
        .transactions(TESTADDR)
        .then((transactions) => {
          expect(transactions).to.be.ok;
          expect(transactions.length).to.equal(25);
        });
    });

    it('retrievs a list of transactions (page 1)', () => {
      return etherscan.account
        .transactions(TESTADDR, 1)
        .then((transactions) => {
          expect(transactions).to.be.ok;
          expect(transactions.length).to.equal(25);
        });
    });
  });
});
