import React from 'react';
import renderHash from './hash.js';
import { IdentityIcon } from '../parity.js';

const container = {
  display: 'inline-block'
};
const align = {
  display: 'inline-block',
  verticalAlign: 'top',
  lineHeight: '32px'
};

export default (address, accounts, contacts, shortenHash = true) => {
  let caption
  if (accounts[address]) {
    caption = (<abbr title={ address } style={ align }>{ accounts[address].name }</abbr>);
  } else if (contacts[address]) {
    caption = (<abbr title={ address } style={ align }>{ contacts[address].name }</abbr>);
  } else {
    caption = (<code style={ align }>{ shortenHash ? renderHash(address) : address }</code>);
  }
  return (
    <div style={ container }>
      <IdentityIcon inline center address={ address } style={ align } />
      { caption }
    </div>
  );
};
