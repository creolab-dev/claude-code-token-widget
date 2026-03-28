import { describe, it, expect } from 'vitest';
import { getStatusColor } from './colors';

describe('getStatusColor - dark theme', () => {
  it('returns green for low usage (0%)', () => {
    expect(getStatusColor(0, true)).toBe('#22c55e');
  });

  it('returns green for usage at 50%', () => {
    expect(getStatusColor(50, true)).toBe('#22c55e');
  });

  it('returns yellow for usage above 50%', () => {
    expect(getStatusColor(50.1, true)).toBe('#eab308');
  });

  it('returns yellow for usage at 75%', () => {
    expect(getStatusColor(75, true)).toBe('#eab308');
  });

  it('returns orange for usage above 75%', () => {
    expect(getStatusColor(75.1, true)).toBe('#f97316');
  });

  it('returns orange for usage at 90%', () => {
    expect(getStatusColor(90, true)).toBe('#f97316');
  });

  it('returns red for usage above 90%', () => {
    expect(getStatusColor(90.1, true)).toBe('#ef4444');
  });

  it('returns red for 100%', () => {
    expect(getStatusColor(100, true)).toBe('#ef4444');
  });
});

describe('getStatusColor - light theme', () => {
  it('returns green for low usage (0%)', () => {
    expect(getStatusColor(0, false)).toBe('#16a34a');
  });

  it('returns green for usage at 50%', () => {
    expect(getStatusColor(50, false)).toBe('#16a34a');
  });

  it('returns yellow for usage above 50%', () => {
    expect(getStatusColor(50.1, false)).toBe('#ca8a04');
  });

  it('returns orange for usage above 75%', () => {
    expect(getStatusColor(75.1, false)).toBe('#ea580c');
  });

  it('returns red for usage above 90%', () => {
    expect(getStatusColor(90.1, false)).toBe('#dc2626');
  });

  it('returns red for 100%', () => {
    expect(getStatusColor(100, false)).toBe('#dc2626');
  });
});

describe('getStatusColor - boundary values', () => {
  it('exactly at each boundary - dark', () => {
    // At exactly 50, 75, 90 - should NOT trigger the higher color
    expect(getStatusColor(50, true)).toBe('#22c55e');   // <= 50 is green
    expect(getStatusColor(75, true)).toBe('#eab308');   // <= 75 is yellow
    expect(getStatusColor(90, true)).toBe('#f97316');   // <= 90 is orange
  });

  it('negative percentage returns green', () => {
    expect(getStatusColor(-1, true)).toBe('#22c55e');
    expect(getStatusColor(-1, false)).toBe('#16a34a');
  });
});
