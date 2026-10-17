const DEFAULT_MAX_EXTRA_COUNT = 1_000;

export function resolveSeedRetention(requestedCount, configuredMaximum) {
  const maxExtraCount =
    configuredMaximum === undefined ? DEFAULT_MAX_EXTRA_COUNT : Number(configuredMaximum);

  if (!Number.isSafeInteger(maxExtraCount) || maxExtraCount < 0) {
    throw new Error('SEED_MAX_EXTRA_COUNT must be a non-negative integer');
  }
  if (!Number.isSafeInteger(requestedCount) || requestedCount < 0) {
    throw new Error('--count must be a non-negative integer');
  }
  if (requestedCount > maxExtraCount) {
    throw new Error(`--count exceeds the configured retention limit of ${maxExtraCount}`);
  }

  return { extraCount: requestedCount, maxExtraCount };
}

export function selectSeedRecordsForCleanup(records, retainCount) {
  if (!Number.isSafeInteger(retainCount) || retainCount < 0) {
    throw new Error('retainCount must be a non-negative integer');
  }

  return [...records]
    .sort((left, right) => {
      const timestampDifference = new Date(right.createdAt).getTime() - new Date(left.createdAt).getTime();
      return timestampDifference || String(right.id).localeCompare(String(left.id));
    })
    .slice(retainCount)
    .map(({ id }) => id);
}
