const initialState = {};

export default (state = initialState, action) => {
  if (action.type === 'addresses set') {
    const contacts = action.addresses
      .filter((address) => !address.isAccount)
      .reduce((contacts, contact) => {
        contacts[contact.address] = contact;
        return contacts;
      }, {});
    return contacts;
  }

  return state;
};
