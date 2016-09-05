export function retrieveAccount (address, accounts, contacts, contracts, tokens) {
  const cmp = (_account) => _account.address === address;

  let account = accounts.find(cmp);
  if (!account) {
    account = contacts.find(cmp);
    if (!account) {
      account = contracts.find(cmp);
      if (!account) {
        account = tokens.find(cmp);
      }
    }
  }

  return account;
}
