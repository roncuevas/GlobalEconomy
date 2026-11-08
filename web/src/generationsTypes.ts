export type Strategy = 'saver' | 'consumer' | 'entrepreneur';
export type DeathCause = 'old_age' | 'poverty';

export type GenConfig = { seed: number; founders: number; max_population: number; inheritance_tax: number; capital_return: number; max_ticks?: number };

export type Citizen = {
  id: number; name: string; lineage: number; parent: number | null; generation: number; age: number; alive: boolean;
  died_tick: number | null; cause: DeathCause | null; money: number; delta: number; earned: number; received: number;
  strategy: Strategy; skill: number; children: number[]; x: number; y: number;
};

export type Lineage = { id: number; surname: string; alive: number; ever: number; wealth: number; share: number; extinct_tick: number | null };

export type GenEvent =
  | { kind: 'birth'; tick: number; child: number; parent: number; lineage: number; dowry: number }
  | { kind: 'death'; tick: number; id: number; lineage: number; age: number; cause: DeathCause; estate: number }
  | { kind: 'inheritance'; tick: number; from: number; heirs: number[]; each: number; tax: number; to_state: number }
  | { kind: 'redistribution'; tick: number; amount: number; recipients: number }
  | { kind: 'extinction'; tick: number; lineage: number }
  | { kind: 'collapse'; tick: number };

export type GenMetrics = {
  tick: number; population: number; births: number; deaths: number; gini: number; top_10_share: number; mean_wealth: number;
  median_wealth: number; total_wealth: number; lineages_alive: number; inherited_share: number; max_generation: number; state_collected: number;
};

export type GenSnapshot = {
  tick: number; config: GenConfig; config_hash: string; finished: boolean; citizens: Citizen[]; lineages: Lineage[];
  events: GenEvent[]; metrics: GenMetrics; metric_history: GenMetrics[];
};

export type FamilyView = { person: Citizen; surname: string; parent: Citizen | null; children: Citizen[]; ancestors: Citizen[] };
