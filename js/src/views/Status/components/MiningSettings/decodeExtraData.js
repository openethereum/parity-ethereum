import rlp from 'rlp';

export function decodeExtraData (str) {
  try {
    // Try decoding as RLP
    const decoded = rlp.decode(str);
    const v = decoded[0];
    decoded[0] = decoded[1];
    decoded[1] = `${v[0]}.${v[1]}.${v[2]}`;
    return decoded.join('/');
  } catch (err) {
    // hex -> str
    return str.match(/.{1,2}/g).map(v => {
      return String.fromCharCode(parseInt(v, 16));
    }).join('');
  }
}
