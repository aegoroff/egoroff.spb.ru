import mitt from 'mitt'

export type Events = {
  pageChanged: number;
  dateSelectionChanged: void;
  postDeleted: void;
  postCreated: void;
  postUpdated: void;
  downloadCreated: void;
  downloadDeleted: void;
  tagChanged: string;
};

export const emitter = mitt<Events>();
