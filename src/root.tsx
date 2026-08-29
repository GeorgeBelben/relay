import { useState } from "react";
import { Logo } from "./components/logo";
import { useCreateGame, useDeleteGame, useGames, useUpdateGame } from "./hooks/use-games";
import { useCreateRom, useRoms } from "./hooks/use-roms";
import { useCreateSystem, useSystems } from "./hooks/use-systems";

// Minimal, unstyled CRUD stub proving the data layer end to end through the UI
// (create a system -> a rom on it -> a game on that rom, then edit/delete it).
// Not real library UI -- that comes with the ingestion pipeline and library screens.
function SystemsSection() {
  const { data: systems } = useSystems();
  const createSystem = useCreateSystem();
  const [id, setId] = useState("");
  const [name, setName] = useState("");

  return (
    <div>
      <h2>Systems</h2>
      <ul>
        {systems?.map((system) => (
          <li key={system.id}>
            {system.id} — {system.name}
          </li>
        ))}
      </ul>
      <form
        onSubmit={(e) => {
          e.preventDefault();
          if (!id || !name) return;
          createSystem.mutate(
            { id, name, extensions: "[]", retroarchCore: null, standaloneBinary: null },
            { onSuccess: () => { setId(""); setName(""); } },
          );
        }}
      >
        <input placeholder="id (e.g. nes)" value={id} onChange={(e) => setId(e.target.value)} />
        <input placeholder="name" value={name} onChange={(e) => setName(e.target.value)} />
        <button type="submit">Add system</button>
      </form>
    </div>
  );
}

function RomsSection() {
  const { data: systems } = useSystems();
  const { data: roms } = useRoms();
  const createRom = useCreateRom();
  const [systemId, setSystemId] = useState("");
  const [path, setPath] = useState("");

  return (
    <div>
      <h2>Roms</h2>
      <ul>
        {roms?.map((rom) => (
          <li key={rom.id}>
            {rom.path} ({rom.system_id})
          </li>
        ))}
      </ul>
      <form
        onSubmit={(e) => {
          e.preventDefault();
          if (!systemId || !path) return;
          createRom.mutate(
            { systemId, path, crc32: null, sizeBytes: null, discs: null },
            { onSuccess: () => setPath("") },
          );
        }}
      >
        <select aria-label="System for rom" value={systemId} onChange={(e) => setSystemId(e.target.value)}>
          <option value="">Select a system</option>
          {systems?.map((system) => (
            <option key={system.id} value={system.id}>
              {system.name}
            </option>
          ))}
        </select>
        <input placeholder="path" value={path} onChange={(e) => setPath(e.target.value)} />
        <button type="submit">Add rom</button>
      </form>
    </div>
  );
}

function GamesSection() {
  const { data: games, isLoading, error } = useGames();
  const { data: roms } = useRoms();
  const createGame = useCreateGame();
  const updateGame = useUpdateGame();
  const deleteGame = useDeleteGame();
  const [romId, setRomId] = useState("");
  const [title, setTitle] = useState("");

  return (
    <div>
      <h2>Games</h2>
      {isLoading && <p>Loading games...</p>}
      {error && <p>Error: {String(error)}</p>}
      {games && (
        <ul>
          {games.map((game) => (
            <li key={game.id}>
              {game.title}{" "}
              <button
                onClick={() => {
                  const newTitle = window.prompt("New title", game.title);
                  if (newTitle) updateGame.mutate({ id: game.id, title: newTitle });
                }}
              >
                Edit
              </button>{" "}
              <button onClick={() => deleteGame.mutate(game.id)}>Delete</button>
            </li>
          ))}
        </ul>
      )}
      <form
        onSubmit={(e) => {
          e.preventDefault();
          if (!romId || !title) return;
          createGame.mutate({ romId, title }, { onSuccess: () => setTitle("") });
        }}
      >
        <select aria-label="Rom for game" value={romId} onChange={(e) => setRomId(e.target.value)}>
          <option value="">Select a rom</option>
          {roms?.map((rom) => (
            <option key={rom.id} value={rom.id}>
              {rom.path}
            </option>
          ))}
        </select>
        <input placeholder="title" value={title} onChange={(e) => setTitle(e.target.value)} />
        <button type="submit">Add game</button>
      </form>
    </div>
  );
}

export function App() {
  return (
    <div className="h-svh flex items-center justify-center">
      <Logo className="w-32" />
      <div>
        <SystemsSection />
        <RomsSection />
        <GamesSection />
      </div>
    </div>
  );
}
