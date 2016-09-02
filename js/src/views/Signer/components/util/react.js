
import { isValidElement } from 'react';

export function isReactComponent (componentOrElem) {
  return isValidElement(componentOrElem) && typeof componentOrElem.type === 'function';
}
