import React from 'react';

export function renderComplete () {
  return (
    <div className='text'>
      Your transaction has been posted. Please visit the <a href='http://127.0.0.1:8180/' className='link' target='_blank'>Parity Signer</a> to authenticate the transfer.
    </div>
  );
}
