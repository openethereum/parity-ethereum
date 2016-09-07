import { numberFromString } from './numberFromString';

describe('NUMBER FROM STRING', () => {
  it('should convert string to number', () => {
    expect(numberFromString('12345'), 12345);
  });

  it('should handle special characters "k" and "m"', () => {
    expect(numberFromString('10kk'), 10000000);
    expect(numberFromString('10K'), 1000);
    expect(numberFromString('10Mmk'), 1000000000000000);
  });

  it('should ignore any non-numeric characters', () => {
    expect(numberFromString('10.000.000'), 10000000);
    expect(numberFromString('10_000_000'), 10000000);
    expect(numberFromString('10_k_k'), 10000000);
    expect(numberFromString('-5'), 5);
  });
});
