export type System = {
  slug: string;
  name: string;
  manufacturer: string;
  generation: number; // console generation, useful for sorting/grouping
  coreBackend: "retroarch" | "pcsx2" | "dolphin";
  releaseYear: number;
};
