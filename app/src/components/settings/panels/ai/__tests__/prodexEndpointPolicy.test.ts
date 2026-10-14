import {describe, expect, it} from 'vitest';
import {validProdexDestination} from '../prodexEndpointPolicy';

describe('Prodex image egress policy', () => {
  it('accepts loopback HTTP and remote HTTPS destinations', () => {
    for (const url of ['http://127.0.0.2:4000/v1', 'http://[::1]:4001/v1', 'http://localhost:4000/v1', 'https://vision.example.com/v1']) {
      expect(validProdexDestination(url)).toBe(true);
    }
  });
  it('rejects remote HTTP, lookalike localhost, embedded credentials and non-HTTP protocols', () => {
    for (const url of ['http://upstream.example.com/v1', 'http://127.0.0.1.bad.example/v1',
      'https://user:password@example.com/v1', 'https://example.com/v1?token=leak',
      'https://example.com/v1#frag', 'file:///secret', '']) {
      expect(validProdexDestination(url)).toBe(false);
    }
  });
});
