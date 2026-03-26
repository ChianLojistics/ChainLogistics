import { describe, it, expect } from 'vitest';
import { shortenPublicKey } from './format';

describe('shortenPublicKey', () => {
  it('should shorten a valid public key correctly', () => {
    const pubKey = 'GAR4LQ4QOQUO3E7FMFJQUX43A2H67ZZO4XU6E7RQ5M6P5XV7K4L3K2V';
    const result = shortenPublicKey(pubKey);
    expect(result).toBe('GAR4…3K2V');
  });

  it('should use custom chars length when provided', () => {
    const pubKey = 'GAR4LQ4QOQUO3E7FMFJQUX43A2H67ZZO4XU6E7RQ5M6P5XV7K4L3K2V';
    const result = shortenPublicKey(pubKey, 6);
    expect(result).toBe('GAR4LQ…4L3K2V');
  });

  it('should return the original key if length is less than or equal to chars * 2', () => {
    const shortKey = 'G123456';
    const result = shortenPublicKey(shortKey, 4);
    expect(result).toBe('G123456');
  });
});
