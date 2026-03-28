import { describe, it, expect } from 'vitest';
import { formatTokens, formatCostUsd, formatCostJpy, formatDuration } from './format';

describe('formatTokens', () => {
  it('formats zero', () => {
    expect(formatTokens(0)).toBe('0');
  });

  it('formats small numbers without commas', () => {
    expect(formatTokens(999)).toBe('999');
  });

  it('formats thousands with commas', () => {
    expect(formatTokens(1000)).toBe('1,000');
    expect(formatTokens(1234567)).toBe('1,234,567');
  });

  it('formats large numbers', () => {
    expect(formatTokens(100000000)).toBe('100,000,000');
  });
});

describe('formatCostUsd', () => {
  it('formats zero', () => {
    expect(formatCostUsd(0)).toBe('$0.0000');
  });

  it('formats small cost with 4 decimal places', () => {
    expect(formatCostUsd(0.0001)).toBe('$0.0001');
  });

  it('formats typical cost', () => {
    expect(formatCostUsd(1.5)).toBe('$1.5000');
  });

  it('rounds to 4 decimal places', () => {
    expect(formatCostUsd(0.12345)).toBe('$0.1235');
  });

  it('formats larger values', () => {
    expect(formatCostUsd(100.999)).toBe('$100.9990');
  });
});

describe('formatCostJpy', () => {
  it('formats zero', () => {
    expect(formatCostJpy(0, 150)).toBe('\u00a50');
  });

  it('converts USD to JPY and rounds', () => {
    expect(formatCostJpy(1.0, 150)).toBe('\u00a5150');
  });

  it('uses comma separator for large values', () => {
    expect(formatCostJpy(100, 150)).toBe('\u00a515,000');
  });

  it('rounds to nearest integer', () => {
    expect(formatCostJpy(0.01, 150)).toBe('\u00a52');
  });

  it('handles different rates', () => {
    expect(formatCostJpy(1.0, 100)).toBe('\u00a5100');
    expect(formatCostJpy(1.0, 200)).toBe('\u00a5200');
  });

  it('handles zero rate', () => {
    expect(formatCostJpy(10, 0)).toBe('\u00a50');
  });
});

describe('formatDuration', () => {
  it('formats zero', () => {
    expect(formatDuration(0)).toBe('00:00:00');
  });

  it('formats seconds only', () => {
    expect(formatDuration(5000)).toBe('00:00:05');
  });

  it('formats minutes and seconds', () => {
    expect(formatDuration(90000)).toBe('00:01:30');
  });

  it('formats hours, minutes, seconds', () => {
    expect(formatDuration(3661000)).toBe('01:01:01');
  });

  it('pads single digits', () => {
    expect(formatDuration(1000)).toBe('00:00:01');
  });

  it('handles large durations', () => {
    expect(formatDuration(86400000)).toBe('24:00:00');
  });

  it('truncates sub-second ms', () => {
    expect(formatDuration(1500)).toBe('00:00:01');
    expect(formatDuration(999)).toBe('00:00:00');
  });
});
