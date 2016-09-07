export function numberFromString (val) {
  return parseInt(
    val
      .replace(/m/ig, 'k')
      .replace(/k/ig, '000')
      .replace(/[^0-9]/g, '')
    , 10
  );
}
