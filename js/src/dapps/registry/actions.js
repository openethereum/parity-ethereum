export const fetchContract = () => ({ type: 'fetch contract' });
export const setContract = (contract) => ({ type: 'set contract', contract });

export const fetchFee = () => ({ type: 'fetch fee' });
export const setFee = (fee) => ({ type: 'set fee', fee });

export const fetchOwner = () => ({ type: 'fetch owner' });
export const setOwner = (owner) => ({ type: 'set owner', owner });
