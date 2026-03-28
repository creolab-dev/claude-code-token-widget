import { describe, it, expect } from 'vitest';
import { getMaxCost, getIntensity } from './heatmap';

describe('getMaxCost', () => {
  it('returns 1 for empty array', () => {
    expect(getMaxCost([])).toBe(1);
  });

  it('returns 0.01 when all costs are zero', () => {
    const entries = [
      { total_cost_usd: 0 },
      { total_cost_usd: 0 },
    ];
    expect(getMaxCost(entries)).toBe(0.01);
  });

  it('returns the maximum cost', () => {
    const entries = [
      { total_cost_usd: 1.5 },
      { total_cost_usd: 3.0 },
      { total_cost_usd: 2.0 },
    ];
    expect(getMaxCost(entries)).toBe(3.0);
  });

  it('returns at least 0.01 for very small costs', () => {
    const entries = [
      { total_cost_usd: 0.001 },
      { total_cost_usd: 0.005 },
    ];
    expect(getMaxCost(entries)).toBe(0.01);
  });

  it('handles single entry', () => {
    expect(getMaxCost([{ total_cost_usd: 5.0 }])).toBe(5.0);
  });

  it('handles negative values by returning 0.01 minimum', () => {
    const entries = [{ total_cost_usd: -1 }];
    expect(getMaxCost(entries)).toBe(0.01);
  });
});

describe('getIntensity', () => {
  it('returns 0 for zero cost', () => {
    expect(getIntensity(0, 10)).toBe(0);
  });

  it('returns 0 for negative cost', () => {
    expect(getIntensity(-1, 10)).toBe(0);
  });

  it('returns 1 for low ratio (0 < ratio <= 0.25)', () => {
    expect(getIntensity(1, 10)).toBe(1);    // ratio = 0.1
    expect(getIntensity(2.5, 10)).toBe(1);  // ratio = 0.25
  });

  it('returns 2 for medium-low ratio (0.25 < ratio <= 0.5)', () => {
    expect(getIntensity(2.6, 10)).toBe(2);  // ratio = 0.26
    expect(getIntensity(5, 10)).toBe(2);    // ratio = 0.5
  });

  it('returns 3 for medium-high ratio (0.5 < ratio <= 0.75)', () => {
    expect(getIntensity(5.1, 10)).toBe(3);  // ratio = 0.51
    expect(getIntensity(7.5, 10)).toBe(3);  // ratio = 0.75
  });

  it('returns 4 for high ratio (ratio > 0.75)', () => {
    expect(getIntensity(7.6, 10)).toBe(4);  // ratio = 0.76
    expect(getIntensity(10, 10)).toBe(4);   // ratio = 1.0
  });

  it('returns 4 when cost equals maxCost', () => {
    expect(getIntensity(5, 5)).toBe(4);     // ratio = 1.0
  });

  it('returns 4 when cost exceeds maxCost', () => {
    expect(getIntensity(15, 10)).toBe(4);   // ratio = 1.5
  });

  it('boundary at exactly 0.25', () => {
    expect(getIntensity(25, 100)).toBe(1);  // ratio = 0.25, not > 0.25
  });

  it('boundary at exactly 0.5', () => {
    expect(getIntensity(50, 100)).toBe(2);  // ratio = 0.5, not > 0.5
  });

  it('boundary at exactly 0.75', () => {
    expect(getIntensity(75, 100)).toBe(3);  // ratio = 0.75, not > 0.75
  });
});
