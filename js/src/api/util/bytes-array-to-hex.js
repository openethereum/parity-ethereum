export default (bytes) =>
  '0x' + bytes.map((b) => ('0' + b.toString(16)).slice(-2)).join('')
