import type { HeatmapEntry } from '$lib/types';
import { getHeatmapData } from '$lib/tauri/commands';
import { getMaxCost, getIntensity } from '$lib/utils/heatmap';

class HeatmapState {
  entries = $state.raw<HeatmapEntry[]>([]);
  loading = $state(false);

  get maxCost(): number {
    return getMaxCost(this.entries);
  }

  /** 0-4 の強度レベルを返す */
  getIntensity(cost: number): number {
    return getIntensity(cost, this.maxCost);
  }

  async load(days = 365): Promise<void> {
    this.loading = true;
    try {
      this.entries = await getHeatmapData(days);
    } catch (e) {
      console.error('Failed to load heatmap data:', e);
      this.entries = [];
    } finally {
      this.loading = false;
    }
  }
}

export const heatmapState = new HeatmapState();
