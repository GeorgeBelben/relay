export type Game = {
  id: string;
  name: string;
  system: string; // slug, references System.slug
  genre: string[];
  releaseYear: number;
  region: "NTSC-U" | "PAL" | "NTSC-J";
  coverArt: string; // placeholder path/URL — swap for real art later
  backdropArt: string; // placeholder path/URL — swap for real art later
  fileSizeMB: number;
  playtimeMinutes: number; // cumulative, 0 if never played
  lastPlayed: string | null; // ISO date, null if never played
  favorite: boolean;
  installed: boolean; // whether ROM is present on disk vs. library-only entry
  players: number; // max local players supported
};
