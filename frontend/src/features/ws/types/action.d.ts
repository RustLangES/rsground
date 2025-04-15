export type Action = {
  type: "insertion";
  pos: number;
  text: string;
  timestamp: number;
} | {
  type: "deletion";
  range_start: number;
  range_end: number;
  timestamp: number;
};
