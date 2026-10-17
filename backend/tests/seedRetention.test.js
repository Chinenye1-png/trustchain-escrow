import { describe, expect, it } from '@jest/globals';
import {
  resolveSeedRetention,
  selectSeedRecordsForCleanup,
} from '../../scripts/seedRetention.js';

describe('seed retention controls', () => {
  it('accepts a count within the configured bound', () => {
    expect(resolveSeedRetention(25, '50')).toEqual({ extraCount: 25, maxExtraCount: 50 });
  });

  it('rejects a count above the configured bound', () => {
    expect(() => resolveSeedRetention(51, '50')).toThrow('retention limit');
  });

  it('rejects unsafe retention configuration', () => {
    expect(() => resolveSeedRetention(1, '-1')).toThrow('SEED_MAX_EXTRA_COUNT');
  });

  it('selects only the oldest deterministic records for cleanup', () => {
    const records = [
      { id: 'a', createdAt: '2026-01-01T00:00:00.000Z' },
      { id: 'c', createdAt: '2026-03-01T00:00:00.000Z' },
      { id: 'b', createdAt: '2026-02-01T00:00:00.000Z' },
    ];

    expect(selectSeedRecordsForCleanup(records, 2)).toEqual(['a']);
  });
});
