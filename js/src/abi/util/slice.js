import { padAddress } from './pad';

export function sliceData (_data) {
  if (!_data || !_data.length) {
    return [];
  }

  let data = (_data.substr(0, 2) === '0x') ? _data.substr(2) : _data;

  if (!data.length) {
    data = padAddress('');
  }

  if (data.length % 64) {
    throw new Error(`Invalid data length (not mod 64) passed to sliceData, ${data}, % 64 == ${data.length % 64}`);
  }

  return data.match(/.{1,64}/g);
}
