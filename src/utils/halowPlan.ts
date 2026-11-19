// 802.11ah (HaLow) channel plans. Frequencies are channel centres in 100 kHz
// units, the unit the firmware uses for wifi_chan_list (AT+CHAN_LIST).

export type Region = 'US' | 'EU' | 'CN' | 'custom';

// first centre per bandwidth (MHz -> 100 kHz units), channels step by their width
type Plan = { name: string, lo: number, hi: number, first: { [bw: number]: number }, count: { [bw: number]: number } };

export const PLANS: { [r in Exclude<Region, 'custom'>]: Plan } = {
  // FCC 902-928 MHz: 8 MHz channels 908/916/924 match the vendor default for 900M modules
  US: { name: 'US (902–928 MHz)', lo: 9020, hi: 9280,
    first: { 1: 9025, 2: 9030, 4: 9060, 8: 9080 }, count: { 1: 26, 2: 13, 4: 6, 8: 3 } },
  // ETSI 863-868 MHz: 1 and 2 MHz only; 864/866 at 2 MHz is the vendor default for 860M modules
  EU: { name: 'EU (863–868 MHz)', lo: 8630, hi: 8680,
    first: { 1: 8635, 2: 8640 }, count: { 1: 5, 2: 2 } },
  // China 755-787 MHz: 8 MHz channels 764/772/780 match the vendor default for 750M modules;
  // narrower channels subdivide the same 760-784 MHz block
  CN: { name: 'China (760–784 MHz)', lo: 7600, hi: 7840,
    first: { 1: 7605, 2: 7610, 4: 7620, 8: 7640 }, count: { 1: 24, 2: 12, 4: 6, 8: 3 } }
};

export const MAX_LIST = 16;   // sys_config.chan_list[16]

/** All channel centres of a region at a bandwidth (empty if that width is not allowed there). */
export function planFreqs(region: Region, bw: number): number[] {
  if (region === 'custom') return [];
  const p = PLANS[region];
  const n = p.count[bw] || 0;
  return Array.from({ length: n }, (_, i) => p.first[bw] + i * bw * 10);
}

/**
 * The list stored on the device: the whole plan, or for plans longer than the
 * firmware's 16 entries, the 16 consecutive channels around the chosen one.
 */
export function deviceList(plan: number[], selected: number): number[] {
  if (plan.length <= MAX_LIST) return plan;
  const i = Math.max(0, plan.indexOf(selected));
  const start = Math.min(Math.max(0, i - MAX_LIST / 2), plan.length - MAX_LIST);
  return plan.slice(start, start + MAX_LIST);
}

/** Which region a stored list was generated from (or 'custom'). */
export function inferRegion(list: number[], bw: number): Region {
  for (const r of Object.keys(PLANS) as Region[]) {
    const plan = planFreqs(r, bw);
    const i = plan.indexOf(list[0]);
    if (i >= 0 && list.length === Math.min(plan.length, MAX_LIST) && list.every((f, k) => plan[i + k] === f)) return r;
  }
  return 'custom';
}

// Tuning range of each module type (MODULE_TYPE_*, sysctrl.h), from sysctrl_freq_type_match()
const MODULE_RANGE: { [type: number]: [number, number] } = {
  1: [7300, 8299],    // 750M
  5: [7900, 8299],    // 810M
  3: [8300, 8899],    // 860M
  2: [8300, 9299],    // 900M
  16: [8400, 8599]    // 850M
};

/** Regions this module can tune to (all of them if the type is unknown). */
export function regionsFor(moduleType: number): Region[] {
  const r = MODULE_RANGE[moduleType];
  const ok = (Object.keys(PLANS) as Exclude<Region, 'custom'>[])
    .filter((k) => !r || (PLANS[k].lo >= r[0] && PLANS[k].hi <= r[1] + 1));
  return [...ok, 'custom'];
}

export const mhz = (f: number) => (f / 10).toFixed(1);
