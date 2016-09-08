export default (bytes) =>
  '0x' + bytes.map((b) => b.toString(16)).join('')
