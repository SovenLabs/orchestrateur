export type CommunicationEdge = {
  from: string;
  to: string;
  count: number;
  lastAt: number;
};

export type CommunicationLogEntry = {
  id: string;
  kind: "message" | "turn" | "event";
  from: string;
  to: string;
  body: string;
  at: number;
};