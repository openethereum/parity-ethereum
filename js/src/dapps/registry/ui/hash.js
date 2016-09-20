import React from 'react';

export default (hash) => {
  const shortened = hash.length > (2 + 9 + 9)
    ? hash.substr(2, 9) + '...' + hash.slice(-9)
    : hash.slice(2);
  return (<abbr title={ hash }>{ shortened }</abbr>);
};
