import React from 'react';

const styles = {
  padding: '.5em',
  border: '1px solid #777'
}

export default (address) => (
  <img
    src={ `http://localhost:8080/${address}/` }
    alt={ address }
    style={ styles }/>
);
