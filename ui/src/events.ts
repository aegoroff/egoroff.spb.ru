import mitt from 'mitt'

export type Events = {
  pageChanged: number;
  dateSelectionChanged: void;
  tagChanged: string;
};

export const emitter = mitt<Events>();
