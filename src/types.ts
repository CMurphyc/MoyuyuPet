export type PetState = "idle" | "working" | "celebrate" | "error" | "sleep";

export type BubbleKind = "idle" | "working" | "celebrate" | "error";

export interface ActivityPayload {
  state: PetState;
  intensity: number;
  token_rate: number;
  source: string;
}

export interface BubblePayload {
  kind: BubbleKind;
  tool?: string | null;
  tokens?: number | null;
  task?: string | null;
}

export interface AdapterStatus {
  source: string;
  status: string;
  message: string;
}

export interface AppSettings {
  paused: boolean;
  autostart: boolean;
  include_task: boolean;
}
